import { useState, useEffect, useRef, useCallback } from 'react';
import type { Highlighter } from 'shiki';

const MAX_CLIENT_BYTES = 50 * 1024;

// File extension → Shiki language id
const EXT_LANG: Record<string, string> = {
  ts: 'typescript', tsx: 'tsx', js: 'javascript', jsx: 'jsx',
  mjs: 'javascript', cjs: 'javascript', mts: 'typescript',
  rs: 'rust', py: 'python', go: 'go',
  java: 'java', kt: 'kotlin', swift: 'swift',
  rb: 'ruby', php: 'php', cs: 'csharp',
  cpp: 'cpp', cc: 'cpp', cxx: 'cpp', c: 'c', h: 'c', hpp: 'cpp',
  sh: 'bash', bash: 'bash', zsh: 'bash', fish: 'fish',
  md: 'markdown', mdx: 'mdx',
  json: 'json', jsonc: 'jsonc', json5: 'json5',
  yaml: 'yaml', yml: 'yaml', toml: 'toml',
  html: 'html', css: 'css', scss: 'scss', sass: 'sass', less: 'less',
  sql: 'sql', graphql: 'graphql',
  xml: 'xml', svg: 'xml',
  svelte: 'svelte', vue: 'vue', astro: 'astro',
  ex: 'elixir', exs: 'elixir', eex: 'html-derivative',
  fs: 'fsharp', fsx: 'fsharp', hs: 'haskell',
  lua: 'lua', r: 'r', dart: 'dart',
  tf: 'hcl', hcl: 'hcl',
  dockerfile: 'dockerfile', makefile: 'makefile',
  proto: 'proto', diff: 'diff', patch: 'diff',
  ini: 'ini', conf: 'ini', env: 'bash',
  lock: 'yaml',
};

const SUPPORTED_LANGS = new Set(Object.values(EXT_LANG));

function detectLang(filePath: string): string {
  const name = filePath.split('/').pop() ?? '';
  const lower = name.toLowerCase();
  // Special filenames
  if (lower === 'dockerfile') return 'dockerfile';
  if (lower === 'makefile' || lower === 'gnumakefile') return 'makefile';
  if (lower === '.env' || lower.startsWith('.env.')) return 'bash';
  const dot = lower.lastIndexOf('.');
  if (dot === -1) return 'text';
  return EXT_LANG[lower.slice(dot + 1)] ?? 'text';
}

// Module-level highlighter cache so we only load Shiki once.
let highlighterPromise: Promise<Highlighter> | null = null;

function getHighlighter(): Promise<Highlighter> {
  if (!highlighterPromise) {
    highlighterPromise = import('shiki').then(({ createHighlighter }) =>
      createHighlighter({
        themes: ['github-light', 'github-dark'],
        langs: Array.from(SUPPORTED_LANGS),
      })
    );
  }
  return highlighterPromise;
}

// Per-file highlight cache: path+lang+theme → html
const htmlCache = new Map<string, string>();

export interface CodeViewerProps {
  projectName: string;
  filePath: string;
  fileSize: number;
  theme?: 'light' | 'dark';
  /** Override language detection */
  lang?: string;
}

type State =
  | { kind: 'idle' }
  | { kind: 'loading' }
  | { kind: 'too_large' }
  | { kind: 'error'; message: string }
  | { kind: 'ready'; html: string; lang: string; lineCount: number };

export function CodeViewer({ projectName, filePath, fileSize, theme = 'light', lang: langOverride }: CodeViewerProps) {
  const [state, setState] = useState<State>({ kind: 'idle' });
  const [selectedLang, setSelectedLang] = useState<string>('');
  const abortRef = useRef<AbortController | null>(null);

  const effectiveLang = selectedLang || langOverride || detectLang(filePath);
  const shikiTheme = theme === 'dark' ? 'github-dark' : 'github-light';
  const cacheKey = `${filePath}|${effectiveLang}|${shikiTheme}`;

  const highlight = useCallback(async (signal: AbortSignal) => {
    if (fileSize > MAX_CLIENT_BYTES) {
      setState({ kind: 'too_large' });
      return;
    }

    // Check in-memory cache first
    const cached = htmlCache.get(cacheKey);
    if (cached !== undefined) {
      const lineCount = (cached.match(/<span class="line"/g) ?? []).length;
      setState({ kind: 'ready', html: cached, lang: effectiveLang, lineCount });
      return;
    }

    setState({ kind: 'loading' });

    try {
      const url = `/api/projects/${encodeURIComponent(projectName)}/files/content?path=${encodeURIComponent(filePath)}&raw=true`;
      const res = await fetch(url, { signal });
      if (!res.ok) {
        if (res.status === 413) {
          setState({ kind: 'too_large' });
          return;
        }
        setState({ kind: 'error', message: `HTTP ${res.status}` });
        return;
      }
      const code = await res.text();
      if (signal.aborted) return;

      const hl = await getHighlighter();
      if (signal.aborted) return;

      const resolvedLang = SUPPORTED_LANGS.has(effectiveLang) ? effectiveLang : 'text';
      const html = hl.codeToHtml(code, { lang: resolvedLang, theme: shikiTheme });

      if (!signal.aborted) {
        htmlCache.set(cacheKey, html);
        const lineCount = (html.match(/<span class="line"/g) ?? []).length;
        setState({ kind: 'ready', html, lang: effectiveLang, lineCount });
      }
    } catch (err) {
      if (!signal.aborted) {
        setState({ kind: 'error', message: String(err) });
      }
    }
  }, [cacheKey, effectiveLang, filePath, fileSize, projectName, shikiTheme]);

  useEffect(() => {
    abortRef.current?.abort();
    const ctrl = new AbortController();
    abortRef.current = ctrl;
    highlight(ctrl.signal);
    return () => ctrl.abort();
  }, [highlight]);

  const langOptions = Array.from(SUPPORTED_LANGS).sort();

  return (
    <div className="code-viewer">
      <div className="code-viewer-toolbar">
        <span className="code-viewer-lang-label">
          {state.kind === 'ready' ? state.lang : effectiveLang}
        </span>
        <select
          className="code-viewer-lang-select"
          value={selectedLang || effectiveLang}
          onChange={e => setSelectedLang(e.target.value)}
          title="Override language"
        >
          {langOptions.map(l => (
            <option key={l} value={l}>{l}</option>
          ))}
          <option value="text">text (plain)</option>
        </select>
        {state.kind === 'ready' && (
          <span className="code-viewer-line-count">{state.lineCount} lines</span>
        )}
      </div>

      <div className="code-viewer-body">
        {state.kind === 'idle' || state.kind === 'loading' ? (
          <div className="code-viewer-loading">Loading…</div>
        ) : state.kind === 'too_large' ? (
          <div className="code-viewer-message">
            File is larger than 50 KB — client-side highlighting is disabled for this file.
          </div>
        ) : state.kind === 'error' ? (
          <div className="code-viewer-message code-viewer-error">{state.message}</div>
        ) : (
          <div
            className="code-viewer-shiki"
            dangerouslySetInnerHTML={{ __html: state.html }}
          />
        )}
      </div>
    </div>
  );
}
