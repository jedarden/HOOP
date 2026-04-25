import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import type { Highlighter } from 'shiki';
import type { BlameLine, BlameMap } from './FilesTab';

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
  clj: 'clojure', cljs: 'clojure', cljc: 'clojure', edn: 'clojure',
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

// Extract individual line HTML strings from the Shiki-generated HTML blob.
// Shiki wraps each line in <span class="line">…</span> inside <pre><code>.
function extractShikiLines(html: string): string[] {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');
  const lineEls = doc.querySelectorAll('.line');
  return Array.from(lineEls).map(el => el.innerHTML);
}

interface ServerHighlightResult {
  language: string;
  line_count: number;
  truncated: boolean;
  theme_bg: string;
  theme_fg: string;
  lines: string[];
}

// Build a single HTML string from syntect server response.
// Each entry in lines is already HTML with inline-styled spans; joining with
// '\n' inside <pre> produces correct line breaks.
function buildServerHtml(data: ServerHighlightResult): string {
  const body = data.lines.join('\n');
  return `<pre style="background:${data.theme_bg};color:${data.theme_fg};">${body}</pre>`;
}

export interface CodeViewerProps {
  projectName: string;
  filePath: string;
  fileSize: number;
  theme?: 'light' | 'dark';
  /** Override language detection */
  lang?: string;
  /** Blame attribution map — when present, renders line-by-line with blame gutter */
  blameMap?: BlameMap;
  onStitchClick?: (stitchId: string) => void;
}

type State =
  | { kind: 'idle' }
  | { kind: 'loading' }
  | { kind: 'error'; message: string }
  | { kind: 'ready'; html: string; lang: string; lineCount: number; truncated: boolean; serverMode: boolean };

// ─── Blame cell (shared with ServerCodeViewer, duplicated for isolation) ─────

function BlameCellInline({ blame, onStitchClick }: { blame: BlameLine | undefined; onStitchClick?: (id: string) => void }) {
  if (!blame) {
    return <span className="blame-cell blame-cell--empty" />;
  }
  if (blame.stitch_id) {
    const tooltip = `${blame.stitch_title ?? blame.bead_id ?? blame.stitch_id}\n${blame.summary}\n${blame.author}`;
    return (
      <span
        className="blame-cell blame-cell--stitch"
        title={tooltip}
        onClick={() => onStitchClick?.(blame.stitch_id!)}
        role="button"
        tabIndex={0}
        onKeyDown={e => e.key === 'Enter' && onStitchClick?.(blame.stitch_id!)}
        aria-label={`Stitch: ${blame.stitch_title}`}
      >
        ●
      </span>
    );
  }
  const shortSha = blame.sha.slice(0, 7);
  return (
    <span
      className="blame-cell blame-cell--git"
      title={`${blame.author} · ${shortSha}\n${blame.summary}`}
    >
      ·
    </span>
  );
}

export function CodeViewer({ projectName, filePath, fileSize, theme = 'light', lang: langOverride, blameMap, onStitchClick }: CodeViewerProps) {
  const [state, setState] = useState<State>({ kind: 'idle' });
  const [selectedLang, setSelectedLang] = useState<string>('');
  const abortRef = useRef<AbortController | null>(null);

  const isLargeFile = fileSize > MAX_CLIENT_BYTES;
  const effectiveLang = selectedLang || langOverride || detectLang(filePath);
  const shikiTheme = theme === 'dark' ? 'github-dark' : 'github-light';
  const cacheKey = `${filePath}|${effectiveLang}|${shikiTheme}`;

  const highlight = useCallback(async (signal: AbortSignal) => {
    // For small files, serve from in-memory cache without a loading flash.
    if (!isLargeFile) {
      const cached = htmlCache.get(cacheKey);
      if (cached !== undefined) {
        const lineCount = (cached.match(/<span class="line"/g) ?? []).length;
        setState({ kind: 'ready', html: cached, lang: effectiveLang, lineCount, truncated: false, serverMode: false });
        return;
      }
    }

    setState({ kind: 'loading' });

    if (isLargeFile) {
      // Large file: delegate to server-side syntect highlighting.
      // The server reads the file, highlights up to 50k lines with the
      // requested theme, and returns a HighlightResult JSON payload.
      const serverTheme = theme === 'dark' ? 'dark' : 'light';
      const url =
        `/api/projects/${encodeURIComponent(projectName)}/files/content` +
        `?path=${encodeURIComponent(filePath)}&theme=${serverTheme}`;
      try {
        const res = await fetch(url, { signal });
        if (!res.ok) {
          if (!signal.aborted) setState({ kind: 'error', message: `HTTP ${res.status}` });
          return;
        }
        const data: ServerHighlightResult = await res.json();
        if (signal.aborted) return;

        const html = buildServerHtml(data);
        setState({
          kind: 'ready',
          html,
          lang: data.language,
          lineCount: data.line_count,
          truncated: data.truncated,
          serverMode: true,
        });
      } catch (err) {
        if (!signal.aborted) setState({ kind: 'error', message: String(err) });
      }
      return;
    }

    // Small file: fetch raw content and highlight client-side with Shiki.
    try {
      const url = `/api/projects/${encodeURIComponent(projectName)}/files/content?path=${encodeURIComponent(filePath)}&raw=true`;
      const res = await fetch(url, { signal });
      if (!res.ok) {
        if (!signal.aborted) setState({ kind: 'error', message: `HTTP ${res.status}` });
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
        setState({ kind: 'ready', html, lang: effectiveLang, lineCount, truncated: false, serverMode: false });
      }
    } catch (err) {
      if (!signal.aborted) setState({ kind: 'error', message: String(err) });
    }
  }, [cacheKey, effectiveLang, filePath, isLargeFile, projectName, shikiTheme, theme]);

  useEffect(() => {
    abortRef.current?.abort();
    const ctrl = new AbortController();
    abortRef.current = ctrl;
    highlight(ctrl.signal);
    return () => ctrl.abort();
  }, [highlight]);

  // When blame mode is active, extract per-line HTML from the Shiki blob.
  // Server-highlighted content uses a different structure; blame is Shiki-only.
  const lineHtmls = useMemo<string[] | null>(() => {
    if (!blameMap || state.kind !== 'ready' || state.serverMode) return null;
    return extractShikiLines(state.html);
  }, [blameMap, state]);

  const langOptions = Array.from(SUPPORTED_LANGS).sort();

  return (
    <div className="code-viewer">
      <div className="code-viewer-toolbar">
        <span className="code-viewer-lang-label">
          {state.kind === 'ready' ? state.lang : effectiveLang}
        </span>
        {/* Language select is only meaningful for client-side Shiki (small files). */}
        {!isLargeFile && (
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
        )}
        {state.kind === 'ready' && (
          <span className="code-viewer-line-count">
            {state.truncated
              ? `first 50k of ${state.lineCount.toLocaleString()} lines`
              : `${state.lineCount.toLocaleString()} lines`}
          </span>
        )}
      </div>

      <div className="code-viewer-body">
        {state.kind === 'idle' || state.kind === 'loading' ? (
          <div className="code-viewer-loading">Loading…</div>
        ) : state.kind === 'error' ? (
          <div className="code-viewer-message code-viewer-error">{state.message}</div>
        ) : lineHtmls !== null ? (
          // ── Blame mode: line-by-line with gutter ───────────────────────
          <div className="code-viewer-blame-lines">
            {lineHtmls.map((lineHtml, i) => {
              const lineNo = i + 1;
              const blame = blameMap!.get(lineNo);
              return (
                <div key={lineNo} className="code-viewer-blame-line">
                  <BlameCellInline blame={blame} onStitchClick={onStitchClick} />
                  <span className="code-viewer-blame-lineno">{lineNo}</span>
                  {/* Shiki emits only styled spans — no scripts possible */}
                  {/* eslint-disable-next-line react/no-danger */}
                  <span className="code-viewer-blame-code" dangerouslySetInnerHTML={{ __html: lineHtml }} />
                </div>
              );
            })}
          </div>
        ) : (
          // ── Normal mode: single Shiki HTML blob ────────────────────────
          <div
            className="code-viewer-shiki"
            dangerouslySetInnerHTML={{ __html: state.html }}
          />
        )}
      </div>
    </div>
  );
}
