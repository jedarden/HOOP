import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { CodeViewer as ShikiCodeViewer } from './CodeViewer';
import { ImageViewer } from './ImageViewer';

const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp', 'ico']);

function isImagePath(p: string): boolean {
  const ext = p.split('.').pop()?.toLowerCase() ?? '';
  return IMAGE_EXTENSIONS.has(ext);
}

const SHIKI_MAX_BYTES = 50 * 1024;

export interface FilesTabProps {
  projectName: string;
  projectPath: string;
}

type GitStatus = 'clean' | 'modified' | 'added' | 'deleted' | 'untracked' | 'renamed' | 'dirty';

interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  mtime: number;
  git_status: GitStatus;
}

interface GrepMatch {
  line_number: number;
  line: string;
  match_start: number;
  match_end: number;
}

interface FileSearchResult {
  path: string;
  name: string;
  size: number;
  mtime: number;
  git_status: GitStatus;
  grep_match: GrepMatch | null;
}

interface FileFilter {
  ext: string;
  modifiedSince: string;
  grep: string;
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatMtime(unixSecs: number): string {
  if (unixSecs === 0) return '';
  const now = Date.now() / 1000;
  const diff = now - unixSecs;
  if (diff < 60) return 'just now';
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  if (diff < 86400 * 30) return `${Math.floor(diff / 86400)}d ago`;
  const d = new Date(unixSecs * 1000);
  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
}

const GIT_STATUS_BADGE: Record<GitStatus, { label: string; color: string } | null> = {
  clean: null,
  modified: { label: 'M', color: '#f9ab00' },
  added: { label: 'A', color: '#34a853' },
  deleted: { label: 'D', color: '#ea4335' },
  untracked: { label: '?', color: '#9aa0a6' },
  renamed: { label: 'R', color: '#4285f4' },
  dirty: { label: '~', color: '#f9ab00' },
};

// ─── URL filter persistence ───────────────────────────────────────────────────

function readFiltersFromHash(): FileFilter {
  const hash = window.location.hash.replace(/^#\/?/, '');
  const qIdx = hash.indexOf('?');
  if (qIdx === -1) return { ext: '', modifiedSince: '', grep: '' };
  const params = new URLSearchParams(hash.slice(qIdx + 1));
  return {
    ext: params.get('ext') ?? '',
    modifiedSince: params.get('modified_since') ?? '',
    grep: params.get('grep') ?? '',
  };
}

function writeFiltersToHash(filters: FileFilter): void {
  const hash = window.location.hash.replace(/^#\/?/, '');
  const pathPart = hash.split('?')[0];
  const params = new URLSearchParams();
  if (filters.ext) params.set('ext', filters.ext);
  if (filters.modifiedSince) params.set('modified_since', filters.modifiedSince);
  if (filters.grep) params.set('grep', filters.grep);
  const qs = params.toString();
  const newHash = `#/${pathPart}${qs ? `?${qs}` : ''}`;
  window.history.replaceState(null, '', newHash);
}

function hasActiveFilter(f: FileFilter): boolean {
  return Boolean(f.ext || f.modifiedSince || f.grep);
}

// ─── Tree node (unchanged tree view) ─────────────────────────────────────────

interface TreeNodeProps {
  entry: FileEntry;
  depth: number;
  expanded: Set<string>;
  loading: Set<string>;
  childCache: Map<string, FileEntry[]>;
  onToggle: (entry: FileEntry) => void;
  onSelect: (entry: FileEntry) => void;
  selectedPath: string | null;
}

function TreeNode({
  entry,
  depth,
  expanded,
  loading,
  childCache,
  onToggle,
  onSelect,
  selectedPath,
}: TreeNodeProps) {
  const isExpanded = expanded.has(entry.path);
  const isLoading = loading.has(entry.path);
  const isSelected = selectedPath === entry.path;
  const badge = GIT_STATUS_BADGE[entry.git_status];

  const handleClick = () => {
    if (entry.is_dir) {
      onToggle(entry);
    } else {
      onSelect(entry);
    }
  };

  const children = childCache.get(entry.path) ?? [];

  return (
    <div>
      <div
        className={`file-tree-node${isSelected ? ' selected' : ''}`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={handleClick}
        title={entry.path}
      >
        <span className="file-expand-icon">
          {entry.is_dir ? (isLoading ? '⋯' : isExpanded ? '▼' : '▶') : ''}
        </span>
        <span className={`file-icon file-icon-${entry.is_dir ? 'directory' : 'file'}`}>
          {entry.is_dir ? '📁' : '📄'}
        </span>
        <span className="file-name">{entry.name}</span>
        {badge && (
          <span className="file-git-badge" style={{ color: badge.color }} title={entry.git_status}>
            {badge.label}
          </span>
        )}
        <span className="file-meta">
          {formatMtime(entry.mtime)}
          {entry.size > 0 && <span className="file-size">{formatSize(entry.size)}</span>}
        </span>
      </div>

      {entry.is_dir && isExpanded && (
        <div className="file-children">
          {isLoading && children.length === 0 ? (
            <div className="file-tree-loading" style={{ paddingLeft: `${(depth + 1) * 16 + 8}px` }}>
              Loading…
            </div>
          ) : (
            children.map(child => (
              <TreeNode
                key={child.path}
                entry={child}
                depth={depth + 1}
                expanded={expanded}
                loading={loading}
                childCache={childCache}
                onToggle={onToggle}
                onSelect={onSelect}
                selectedPath={selectedPath}
              />
            ))
          )}
        </div>
      )}
    </div>
  );
}

// ─── Search result row ────────────────────────────────────────────────────────

function SearchResultRow({
  result,
  isSelected,
  onSelect,
}: {
  result: FileSearchResult;
  isSelected: boolean;
  onSelect: (r: FileSearchResult) => void;
}) {
  const badge = GIT_STATUS_BADGE[result.git_status];
  const gm = result.grep_match;

  // Highlight the match inside the line.
  let lineNode: React.ReactNode = null;
  if (gm) {
    const { line, match_start, match_end } = gm;
    // Truncate long lines, keeping context around the match.
    const MAX_CHARS = 120;
    let displayLine = line;
    let adjustedStart = match_start;
    let adjustedEnd = match_end;
    if (line.length > MAX_CHARS) {
      const contextBefore = 30;
      const sliceStart = Math.max(0, match_start - contextBefore);
      displayLine = (sliceStart > 0 ? '…' : '') + line.slice(sliceStart, sliceStart + MAX_CHARS);
      adjustedStart = match_start - sliceStart + (sliceStart > 0 ? 1 : 0);
      adjustedEnd = match_end - sliceStart + (sliceStart > 0 ? 1 : 0);
      if (sliceStart + MAX_CHARS < line.length) displayLine += '…';
    }

    lineNode = (
      <span className="file-grep-line">
        <span className="file-grep-lineno">{gm.line_number}</span>
        <span className="file-grep-text">
          {displayLine.slice(0, adjustedStart)}
          <mark className="file-grep-mark">{displayLine.slice(adjustedStart, adjustedEnd)}</mark>
          {displayLine.slice(adjustedEnd)}
        </span>
      </span>
    );
  }

  return (
    <div
      className={`file-search-row${isSelected ? ' selected' : ''}`}
      onClick={() => onSelect(result)}
      title={result.path}
    >
      <div className="file-search-top">
        <span className="file-icon">📄</span>
        <span className="file-search-path">{result.path}</span>
        {badge && (
          <span className="file-git-badge" style={{ color: badge.color }} title={result.git_status}>
            {badge.label}
          </span>
        )}
        <span className="file-meta">
          {formatMtime(result.mtime)}
          {result.size > 0 && <span className="file-size">{formatSize(result.size)}</span>}
        </span>
      </div>
      {lineNode && <div className="file-search-match">{lineNode}</div>}
    </div>
  );
}

// ─── Filter bar ───────────────────────────────────────────────────────────────

interface FilterBarProps {
  filter: FileFilter;
  onChange: (f: FileFilter) => void;
  resultCount: number | null;
  searching: boolean;
}

const MODIFIED_SINCE_SUGGESTIONS = ['HEAD~1', 'HEAD~3', 'HEAD~5', 'HEAD~10', 'HEAD~20', 'main', 'master'];

function FilterBar({ filter, onChange, resultCount, searching }: FilterBarProps) {
  const active = hasActiveFilter(filter);

  return (
    <div className={`files-filter-bar${active ? ' active' : ''}`}>
      <div className="files-filter-inputs">
        <div className="files-filter-field">
          <label className="files-filter-label" htmlFor="ff-ext">ext</label>
          <input
            id="ff-ext"
            className="files-filter-input"
            type="text"
            placeholder="*.rs  or  *.{ts,tsx}"
            value={filter.ext}
            onChange={e => onChange({ ...filter, ext: e.target.value })}
            spellCheck={false}
          />
        </div>

        <div className="files-filter-field">
          <label className="files-filter-label" htmlFor="ff-since">since</label>
          <input
            id="ff-since"
            className="files-filter-input"
            type="text"
            placeholder="HEAD~N or ref"
            list="modified-since-suggestions"
            value={filter.modifiedSince}
            onChange={e => onChange({ ...filter, modifiedSince: e.target.value })}
            spellCheck={false}
          />
          <datalist id="modified-since-suggestions">
            {MODIFIED_SINCE_SUGGESTIONS.map(s => (
              <option key={s} value={s} />
            ))}
          </datalist>
        </div>

        <div className="files-filter-field files-filter-field--grep">
          <label className="files-filter-label" htmlFor="ff-grep">grep</label>
          <input
            id="ff-grep"
            className="files-filter-input"
            type="text"
            placeholder="regex pattern"
            value={filter.grep}
            onChange={e => onChange({ ...filter, grep: e.target.value })}
            spellCheck={false}
          />
        </div>
      </div>

      <div className="files-filter-status">
        {searching && <span className="files-filter-spinner">⋯</span>}
        {!searching && resultCount !== null && (
          <span className="files-filter-count">{resultCount} file{resultCount !== 1 ? 's' : ''}</span>
        )}
        {active && (
          <button
            className="files-filter-clear"
            onClick={() => onChange({ ext: '', modifiedSince: '', grep: '' })}
            title="Clear all filters"
          >
            ✕
          </button>
        )}
      </div>
    </div>
  );
}

// ─── Server-side (syntect) code viewer — fallback for files >50 KB ───────────

interface HighlightResult {
  language: string;
  line_count: number;
  truncated: boolean;
  theme_bg: string;
  theme_fg: string;
  lines: string[];
}

const LINE_HEIGHT = 20; // px — must match .hl-line height in CSS

function ServerCodeViewer({
  projectName,
  path,
  theme,
}: {
  projectName: string;
  path: string;
  theme: string;
}) {
  const [result, setResult] = useState<HighlightResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(400);

  // Observe the container's rendered height so virtual scroll fills correctly.
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver(entries => {
      setContainerHeight(entries[0].contentRect.height);
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    setLoading(true);
    setError(null);
    setResult(null);
    setScrollTop(0);
    const ctrl = new AbortController();
    const params = new URLSearchParams({ path, theme });
    fetch(`/api/projects/${encodeURIComponent(projectName)}/files/content?${params}`, {
      signal: ctrl.signal,
    })
      .then(r => (r.ok ? r.json() : Promise.reject(`HTTP ${r.status}`)))
      .then((data: HighlightResult) => {
        setResult(data);
        setLoading(false);
      })
      .catch((err: unknown) => {
        if (err instanceof Error && err.name === 'AbortError') return;
        setError(String(err));
        setLoading(false);
      });
    return () => ctrl.abort();
  }, [projectName, path, theme]);

  const visibleSlice = useMemo(() => {
    if (!result) return { start: 0, end: 0, paddingTop: 0 };
    const start = Math.max(0, Math.floor(scrollTop / LINE_HEIGHT) - 5);
    const visible = Math.ceil(containerHeight / LINE_HEIGHT) + 10;
    const end = Math.min(start + visible, result.lines.length);
    return { start, end, paddingTop: start * LINE_HEIGHT };
  }, [scrollTop, containerHeight, result]);

  if (loading) {
    return <div className="hl-status">Loading…</div>;
  }
  if (error) {
    return <div className="hl-status hl-status--error">{error}</div>;
  }
  if (!result) return null;

  const totalHeight = result.lines.length * LINE_HEIGHT;
  const { start, end, paddingTop } = visibleSlice;

  return (
    <div className="hl-wrapper">
      <div className="hl-toolbar">
        <span className="hl-lang">{result.language}</span>
        <span className="hl-linecount">
          {result.line_count.toLocaleString()} lines
          {result.truncated && ' (first 50 000 shown)'}
        </span>
      </div>
      <div
        ref={containerRef}
        className="hl-container"
        style={{ backgroundColor: result.theme_bg, color: result.theme_fg }}
        onScroll={e => setScrollTop((e.currentTarget as HTMLDivElement).scrollTop)}
      >
        <div style={{ height: `${totalHeight}px`, position: 'relative' }}>
          <div style={{ position: 'absolute', top: `${paddingTop}px`, width: '100%' }}>
            {result.lines.slice(start, end).map((lineHtml, i) => (
              <div
                key={start + i}
                className="hl-line"
                data-ln={start + i + 1}
              >
                <span className="hl-lineno">{start + i + 1}</span>
                {/* syntect emits only span[style] nodes — no scripts possible */}
                {/* eslint-disable-next-line react/no-danger */}
                <span className="hl-code" dangerouslySetInnerHTML={{ __html: lineHtml }} />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

// ─── Main component ───────────────────────────────────────────────────────────

export default function FilesTab({ projectName, projectPath }: FilesTabProps) {
  // Tree state
  const [rootEntries, setRootEntries] = useState<FileEntry[]>([]);
  const [rootLoading, setRootLoading] = useState(true);
  const [rootError, setRootError] = useState<string | null>(null);
  const [childCache, setChildCache] = useState<Map<string, FileEntry[]>>(new Map());
  const [loading, setLoading] = useState<Set<string>>(new Set());
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [selectedFile, setSelectedFile] = useState<{ path: string; size: number } | null>(null);

  // Filter state — initialised from URL hash
  const [filter, setFilter] = useState<FileFilter>(readFiltersFromHash);

  // Syntax highlight theme — default matches the overall light UI
  const [hlTheme, setHlTheme] = useState<string>('light');

  // Search state
  const [searchResults, setSearchResults] = useState<FileSearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);

  const searchAbortRef = useRef<AbortController | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const activeFilter = hasActiveFilter(filter);

  // ── Sync filter → URL ──────────────────────────────────────────────────────
  useEffect(() => {
    writeFiltersToHash(filter);
  }, [filter]);

  // ── Sync URL hash changes → filter (back/forward nav) ─────────────────────
  useEffect(() => {
    const handler = () => setFilter(readFiltersFromHash());
    window.addEventListener('hashchange', handler);
    return () => window.removeEventListener('hashchange', handler);
  }, []);

  // ── Tree: fetch root ───────────────────────────────────────────────────────
  const fetchDir = useCallback(
    async (relPath: string): Promise<FileEntry[]> => {
      const url = `/api/projects/${encodeURIComponent(projectName)}/files${relPath ? `?path=${encodeURIComponent(relPath)}` : ''}`;
      const res = await fetch(url);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      return res.json();
    },
    [projectName],
  );

  useEffect(() => {
    setRootLoading(true);
    setRootError(null);
    fetchDir('')
      .then(entries => {
        setRootEntries(entries);
        setRootLoading(false);
      })
      .catch(err => {
        setRootError(String(err));
        setRootLoading(false);
      });
  }, [fetchDir]);

  const handleToggle = useCallback(
    async (entry: FileEntry) => {
      if (!entry.is_dir) return;

      if (expanded.has(entry.path)) {
        setExpanded(prev => {
          const next = new Set(prev);
          next.delete(entry.path);
          return next;
        });
        return;
      }

      if (childCache.has(entry.path)) {
        setExpanded(prev => new Set(prev).add(entry.path));
        return;
      }

      setLoading(prev => new Set(prev).add(entry.path));
      setExpanded(prev => new Set(prev).add(entry.path));

      try {
        const children = await fetchDir(entry.path);
        setChildCache(prev => new Map(prev).set(entry.path, children));
      } catch (_err) {
        setChildCache(prev => new Map(prev).set(entry.path, []));
      } finally {
        setLoading(prev => {
          const next = new Set(prev);
          next.delete(entry.path);
          return next;
        });
      }
    },
    [expanded, childCache, fetchDir],
  );

  const handleSelect = useCallback((entry: FileEntry | FileSearchResult) => {
    setSelectedFile(prev =>
      prev?.path === entry.path ? null : { path: entry.path, size: entry.size },
    );
  }, []);

  // ── Search: fetch when filter changes ─────────────────────────────────────
  useEffect(() => {
    if (!activeFilter) {
      setSearchResults([]);
      setSearchError(null);
      setSearching(false);
      return;
    }

    if (debounceRef.current) clearTimeout(debounceRef.current);

    debounceRef.current = setTimeout(async () => {
      if (searchAbortRef.current) searchAbortRef.current.abort();
      const abort = new AbortController();
      searchAbortRef.current = abort;

      setSearching(true);
      setSearchError(null);

      try {
        const params = new URLSearchParams();
        if (filter.ext) params.set('ext', filter.ext);
        if (filter.modifiedSince) params.set('modified_since', filter.modifiedSince);
        if (filter.grep) params.set('grep', filter.grep);

        const url = `/api/projects/${encodeURIComponent(projectName)}/files/search?${params}`;
        const res = await fetch(url, { signal: abort.signal });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data: FileSearchResult[] = await res.json();
        setSearchResults(data);
      } catch (err: unknown) {
        if (err instanceof Error && err.name === 'AbortError') return;
        setSearchError(String(err));
      } finally {
        if (!abort.signal.aborted) setSearching(false);
      }
    }, 300);

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [filter, activeFilter, projectName]);

  const selectedPath = selectedFile?.path ?? null;

  // Map the theme name to Shiki's 'light' | 'dark' binary.
  // Light variants get 'light'; everything else (dark, solarized-dark, etc.) maps to 'dark'.
  const shikiTheme: 'light' | 'dark' = (
    hlTheme === 'light' || hlTheme === 'solarized-light' || hlTheme === 'ocean-light'
  ) ? 'light' : 'dark';

  // ── Render ─────────────────────────────────────────────────────────────────
  return (
    <div className="files-tab">
      <div className="files-header">
        <h4>File Browser</h4>
        <p className="files-path">{projectPath}</p>
      </div>

      <FilterBar
        filter={filter}
        onChange={setFilter}
        resultCount={activeFilter ? searchResults.length : null}
        searching={searching}
      />

      <div className="files-content">
        {activeFilter ? (
          // ── Search / flat results view ──────────────────────────────────
          <div className="file-search-results">
            {searching && searchResults.length === 0 && (
              <div className="file-tree-loading" style={{ padding: '1rem' }}>Searching…</div>
            )}
            {searchError && (
              <div className="file-tree-error" style={{ padding: '1rem' }}>{searchError}</div>
            )}
            {!searching && !searchError && searchResults.length === 0 && (
              <div className="file-search-empty">No files match the current filters.</div>
            )}
            {searchResults.map(r => (
              <SearchResultRow
                key={r.path}
                result={r}
                isSelected={selectedPath === r.path}
                onSelect={handleSelect}
              />
            ))}
          </div>
        ) : (
          // ── Tree view (default) ─────────────────────────────────────────
          <div className="file-tree">
            {rootLoading && (
              <div className="file-tree-loading" style={{ padding: '1rem' }}>Loading…</div>
            )}
            {rootError && (
              <div className="file-tree-error" style={{ padding: '1rem' }}>{rootError}</div>
            )}
            {!rootLoading &&
              !rootError &&
              rootEntries.map(entry => (
                <TreeNode
                  key={entry.path}
                  entry={entry}
                  depth={0}
                  expanded={expanded}
                  loading={loading}
                  childCache={childCache}
                  onToggle={handleToggle}
                  onSelect={handleSelect}
                  selectedPath={selectedPath}
                />
              ))}
          </div>
        )}

        {selectedFile && (
          <div className="file-preview">
            <div className="file-preview-header">
              <span className="file-preview-path">{selectedFile.path}</span>
              <div className="file-preview-controls">
                {!isImagePath(selectedFile.path) && (
                  <select
                    className="hl-theme-select"
                    value={hlTheme}
                    onChange={e => setHlTheme(e.target.value)}
                    title="Highlight theme"
                  >
                    <option value="light">GitHub Light</option>
                    <option value="dark">GitHub Dark</option>
                    <option value="solarized-dark">Solarized Dark</option>
                    <option value="solarized-light">Solarized Light</option>
                    <option value="eighties">Eighties Dark</option>
                    <option value="mocha-dark">Mocha Dark</option>
                    <option value="ocean-light">Ocean Light</option>
                  </select>
                )}
                <button className="file-preview-close" onClick={() => setSelectedFile(null)}>
                  ×
                </button>
              </div>
            </div>
            <div className={`file-preview-body${isImagePath(selectedFile.path) ? ' file-preview-body--image' : ' file-preview-body--code'}`}>
              {isImagePath(selectedFile.path) ? (
                <ImageViewer
                  projectName={projectName}
                  path={selectedFile.path}
                />
              ) : selectedFile.size <= SHIKI_MAX_BYTES ? (
                <ShikiCodeViewer
                  projectName={projectName}
                  filePath={selectedFile.path}
                  fileSize={selectedFile.size}
                  theme={shikiTheme}
                />
              ) : (
                <ServerCodeViewer
                  projectName={projectName}
                  path={selectedFile.path}
                  theme={hlTheme}
                />
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
