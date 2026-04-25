import {
  useState,
  useEffect,
  useCallback,
  useRef,
  useMemo,
} from 'react';

// ─── API types ────────────────────────────────────────────────────────────────

type DiffLineKind = 'context' | 'add' | 'remove';

interface DiffLine {
  kind: DiffLineKind;
  content: string;
  old_lineno: number | null;
  new_lineno: number | null;
}

interface DiffHunk {
  old_start: number;
  old_count: number;
  new_start: number;
  new_count: number;
  header: string;
  lines: DiffLine[];
}

interface FileDiff {
  old_path: string;
  new_path: string;
  is_new: boolean;
  is_deleted: boolean;
  is_binary: boolean;
  added: number;
  removed: number;
  hunks: DiffHunk[];
}

interface DiffResponse {
  files: FileDiff[];
  total_added: number;
  total_removed: number;
  truncated: boolean;
  ref_range: string;
}

// ─── Virtualized diff row ─────────────────────────────────────────────────────

type ViewMode = 'unified' | 'split';

/** A flat row in the rendered diff (one or two diff lines for split view). */
interface DiffRow {
  type: 'hunk-header';
  header: string;
  hunkIndex: number;
}

interface DiffContentRow {
  type: 'line';
  // unified: single line
  line?: DiffLine;
  // split: left=old side, right=new side (null = empty placeholder)
  left?: DiffLine | null;
  right?: DiffLine | null;
}

type Row = DiffRow | DiffContentRow;

// ─── Split row builder ────────────────────────────────────────────────────────

function buildSplitRows(lines: DiffLine[]): DiffContentRow[] {
  const rows: DiffContentRow[] = [];
  let removes: DiffLine[] = [];
  let adds: DiffLine[] = [];

  function flush() {
    const n = Math.max(removes.length, adds.length);
    for (let i = 0; i < n; i++) {
      rows.push({
        type: 'line',
        left: removes[i] ?? null,
        right: adds[i] ?? null,
      });
    }
    removes = [];
    adds = [];
  }

  for (const line of lines) {
    if (line.kind === 'context') {
      flush();
      rows.push({ type: 'line', left: line, right: line });
    } else if (line.kind === 'remove') {
      removes.push(line);
    } else {
      adds.push(line);
    }
  }
  flush();
  return rows;
}

// ─── Flat row list builder ────────────────────────────────────────────────────

function buildRows(file: FileDiff, mode: ViewMode): Row[] {
  const rows: Row[] = [];
  for (let hi = 0; hi < file.hunks.length; hi++) {
    const hunk = file.hunks[hi];
    rows.push({ type: 'hunk-header', header: hunk.header, hunkIndex: hi });
    if (mode === 'unified') {
      for (const line of hunk.lines) {
        rows.push({ type: 'line', line });
      }
    } else {
      const splitRows = buildSplitRows(hunk.lines);
      rows.push(...splitRows);
    }
  }
  return rows;
}

// ─── Constants ────────────────────────────────────────────────────────────────

const ROW_HEIGHT = 22; // px — must match CSS .diff-row height
const OVERSCAN = 30; // extra rows above/below viewport

// ─── VirtualDiffList ─────────────────────────────────────────────────────────

interface VirtualDiffListProps {
  rows: Row[];
  mode: ViewMode;
  currentHunkIndex: number | null;
  hunkRefs: React.RefObject<Map<number, number>>; // hunkIndex → rowIndex
}

function VirtualDiffList({ rows, mode, currentHunkIndex, hunkRefs }: VirtualDiffListProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(600);

  // Build a map: hunkIndex → row index (for keyboard navigation)
  useEffect(() => {
    const map = hunkRefs.current!;
    map.clear();
    rows.forEach((row, i) => {
      if (row.type === 'hunk-header') {
        map.set(row.hunkIndex, i);
      }
    });
  }, [rows, hunkRefs]);

  // Track container height via ResizeObserver
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const observer = new ResizeObserver(entries => {
      for (const entry of entries) {
        setContainerHeight(entry.contentRect.height);
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // Scroll to current hunk when it changes
  useEffect(() => {
    if (currentHunkIndex == null) return;
    const rowIndex = hunkRefs.current!.get(currentHunkIndex);
    if (rowIndex == null) return;
    const targetScrollTop = rowIndex * ROW_HEIGHT - 40;
    containerRef.current?.scrollTo({ top: Math.max(0, targetScrollTop), behavior: 'smooth' });
  }, [currentHunkIndex, hunkRefs]);

  const totalHeight = rows.length * ROW_HEIGHT;
  const startIndex = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const endIndex = Math.min(
    rows.length - 1,
    Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + OVERSCAN,
  );

  const visibleRows = rows.slice(startIndex, endIndex + 1);

  return (
    <div
      ref={containerRef}
      className="diff-virtual-container"
      onScroll={e => setScrollTop((e.currentTarget as HTMLDivElement).scrollTop)}
    >
      <div className="diff-virtual-total" style={{ height: totalHeight }}>
        <div
          className="diff-virtual-window"
          style={{ transform: `translateY(${startIndex * ROW_HEIGHT}px)` }}
        >
          {visibleRows.map((row, idx) => {
            const rowIndex = startIndex + idx;
            const isCurrentHunk =
              row.type === 'hunk-header' && row.hunkIndex === currentHunkIndex;
            if (row.type === 'hunk-header') {
              return (
                <div
                  key={rowIndex}
                  className={`diff-row diff-hunk-header${isCurrentHunk ? ' current-hunk' : ''}`}
                >
                  {row.header}
                </div>
              );
            }
            // Content row
            if (mode === 'unified') {
              const line = (row as DiffContentRow).line!;
              return (
                <div key={rowIndex} className={`diff-row diff-line diff-line--${line.kind}`}>
                  <span className="diff-lineno diff-lineno-old">
                    {line.old_lineno ?? ''}
                  </span>
                  <span className="diff-lineno diff-lineno-new">
                    {line.new_lineno ?? ''}
                  </span>
                  <span className="diff-line-marker">
                    {line.kind === 'add' ? '+' : line.kind === 'remove' ? '-' : ' '}
                  </span>
                  <span className="diff-line-content">{line.content}</span>
                </div>
              );
            } else {
              const r = row as DiffContentRow;
              const left = r.left ?? null;
              const right = r.right ?? null;
              return (
                <div key={rowIndex} className="diff-row diff-split-row">
                  <div
                    className={`diff-split-cell diff-split-cell--left${left ? ` diff-line--${left.kind}` : ' diff-split-empty'}`}
                  >
                    {left && (
                      <>
                        <span className="diff-lineno">{left.old_lineno ?? ''}</span>
                        <span className="diff-line-marker">
                          {left.kind === 'remove' ? '-' : ' '}
                        </span>
                        <span className="diff-line-content">{left.content}</span>
                      </>
                    )}
                  </div>
                  <div
                    className={`diff-split-cell diff-split-cell--right${right ? ` diff-line--${right.kind}` : ' diff-split-empty'}`}
                  >
                    {right && (
                      <>
                        <span className="diff-lineno">{right.new_lineno ?? ''}</span>
                        <span className="diff-line-marker">
                          {right.kind === 'add' ? '+' : ' '}
                        </span>
                        <span className="diff-line-content">{right.content}</span>
                      </>
                    )}
                  </div>
                </div>
              );
            }
          })}
        </div>
      </div>
    </div>
  );
}

// ─── FileDiffPanel ────────────────────────────────────────────────────────────

interface FileDiffPanelProps {
  file: FileDiff;
  mode: ViewMode;
  currentHunkIndex: number | null;
  hunkRefs: React.RefObject<Map<number, number>>;
}

function FileDiffPanel({ file, mode, currentHunkIndex, hunkRefs }: FileDiffPanelProps) {
  const rows = useMemo(() => buildRows(file, mode), [file, mode]);

  if (file.is_binary) {
    return <div className="diff-binary-notice">Binary file — diff not shown</div>;
  }
  if (file.hunks.length === 0) {
    return <div className="diff-empty-notice">No changes</div>;
  }

  return (
    <VirtualDiffList
      rows={rows}
      mode={mode}
      currentHunkIndex={currentHunkIndex}
      hunkRefs={hunkRefs}
    />
  );
}

// ─── Three-way mode ───────────────────────────────────────────────────────────

interface ThreeWayPanelProps {
  projectName: string;
  filePath: string | undefined;
  mergeBaseSha: string;
  mode: ViewMode;
}

function ThreeWayPanel({ projectName, filePath, mergeBaseSha, mode }: ThreeWayPanelProps) {
  const [committedDiff, setCommittedDiff] = useState<DiffResponse | null>(null);
  const [uncommittedDiff, setUncommittedDiff] = useState<DiffResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Hunk navigation state (shared across both panels for simplicity)
  const committedHunkRefs = useRef(new Map<number, number>());
  const uncommittedHunkRefs = useRef(new Map<number, number>());
  const [committedHunkIdx, setCommittedHunkIdx] = useState<number | null>(null);
  const [uncommittedHunkIdx, setUncommittedHunkIdx] = useState<number | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    const build = (ref: string, ref2?: string) => {
      const params = new URLSearchParams({ ref });
      if (ref2) params.set('ref2', ref2);
      if (filePath) params.set('file', filePath);
      return fetch(`/api/projects/${encodeURIComponent(projectName)}/diff?${params}`).then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json() as Promise<DiffResponse>;
      });
    };

    Promise.all([
      build(mergeBaseSha, 'HEAD'),
      build('HEAD'),
    ])
      .then(([committed, uncommitted]) => {
        if (cancelled) return;
        setCommittedDiff(committed);
        setUncommittedDiff(uncommitted);
        setLoading(false);
      })
      .catch(err => {
        if (cancelled) return;
        setError(String(err));
        setLoading(false);
      });

    return () => { cancelled = true; };
  }, [projectName, filePath, mergeBaseSha]);

  if (loading) return <div className="diff-loading">Loading three-way diff…</div>;
  if (error) return <div className="diff-error">{error}</div>;

  const committedFile = committedDiff?.files.find(
    f => !filePath || f.new_path === filePath || f.old_path === filePath,
  ) ?? committedDiff?.files[0];
  const uncommittedFile = uncommittedDiff?.files.find(
    f => !filePath || f.new_path === filePath || f.old_path === filePath,
  ) ?? uncommittedDiff?.files[0];

  const totalHunks = Math.max(
    committedFile?.hunks.length ?? 0,
    uncommittedFile?.hunks.length ?? 0,
  );

  return (
    <div className="diff-three-way">
      <div className="diff-three-way-labels">
        <div className="diff-three-way-label">
          <span className="diff-panel-badge diff-panel-badge--committed">committed</span>
          {' '}merge-base → HEAD
          {committedFile && (
            <span className="diff-stats">
              <span className="diff-stat-add">+{committedFile.added}</span>
              <span className="diff-stat-rem">-{committedFile.removed}</span>
            </span>
          )}
        </div>
        <div className="diff-three-way-label">
          <span className="diff-panel-badge diff-panel-badge--uncommitted">uncommitted</span>
          {' '}HEAD → working tree
          {uncommittedFile && (
            <span className="diff-stats">
              <span className="diff-stat-add">+{uncommittedFile.added}</span>
              <span className="diff-stat-rem">-{uncommittedFile.removed}</span>
            </span>
          )}
        </div>
      </div>
      {totalHunks > 0 && (
        <div className="diff-three-way-nav">
          <button
            className="diff-nav-btn"
            disabled={committedHunkIdx == null && uncommittedHunkIdx == null}
            onClick={() => {
              setCommittedHunkIdx(i => (i != null && i > 0 ? i - 1 : i));
              setUncommittedHunkIdx(i => (i != null && i > 0 ? i - 1 : i));
            }}
          >
            ← prev hunk
          </button>
          <button
            className="diff-nav-btn"
            onClick={() => {
              const maxC = (committedFile?.hunks.length ?? 0) - 1;
              const maxU = (uncommittedFile?.hunks.length ?? 0) - 1;
              setCommittedHunkIdx(i => (i == null ? 0 : Math.min(i + 1, maxC)));
              setUncommittedHunkIdx(i => (i == null ? 0 : Math.min(i + 1, maxU)));
            }}
          >
            next hunk →
          </button>
        </div>
      )}
      <div className="diff-three-way-panels">
        <div className="diff-three-way-panel">
          {committedFile ? (
            <FileDiffPanel
              file={committedFile}
              mode={mode}
              currentHunkIndex={committedHunkIdx}
              hunkRefs={committedHunkRefs}
            />
          ) : (
            <div className="diff-empty-notice">No committed changes</div>
          )}
        </div>
        <div className="diff-three-way-panel">
          {uncommittedFile ? (
            <FileDiffPanel
              file={uncommittedFile}
              mode={mode}
              currentHunkIndex={uncommittedHunkIdx}
              hunkRefs={uncommittedHunkRefs}
            />
          ) : (
            <div className="diff-empty-notice">No uncommitted changes</div>
          )}
        </div>
      </div>
    </div>
  );
}

// ─── Main DiffViewer ──────────────────────────────────────────────────────────

export interface DiffViewerProps {
  projectName: string;
  /** If set, only this file's diff is shown. Otherwise shows all changed files. */
  filePath?: string;
}

const REF_PRESETS = [
  { label: 'Working vs HEAD', ref_: 'HEAD', ref2: undefined },
  { label: 'HEAD vs HEAD~1', ref_: 'HEAD~1', ref2: 'HEAD' },
  { label: 'HEAD vs HEAD~3', ref_: 'HEAD~3', ref2: 'HEAD' },
];

export default function DiffViewer({ projectName, filePath }: DiffViewerProps) {
  const [mode, setMode] = useState<ViewMode>('unified');
  const [threeWay, setThreeWay] = useState(false);
  const [mergeBaseSha, setMergeBaseSha] = useState<string | null>(null);
  const [mergeBaseLoading, setMergeBaseLoading] = useState(false);

  // Two-way state
  const [selectedRef, setSelectedRef] = useState<{ ref_: string; ref2?: string }>(REF_PRESETS[0]);
  const [customRef, setCustomRef] = useState('');
  const [diff, setDiff] = useState<DiffResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // File selection within multi-file diffs
  const [selectedFileIdx, setSelectedFileIdx] = useState(0);

  // Hunk navigation
  const [currentHunkIndex, setCurrentHunkIndex] = useState<number | null>(null);
  const hunkRefs = useRef(new Map<number, number>());
  const viewerRef = useRef<HTMLDivElement>(null);

  // ── Fetch two-way diff ────────────────────────────────────────────────────

  useEffect(() => {
    if (threeWay) return;
    let cancelled = false;
    setLoading(true);
    setError(null);
    setCurrentHunkIndex(null);

    const params = new URLSearchParams({ ref: selectedRef.ref_ });
    if (selectedRef.ref2) params.set('ref2', selectedRef.ref2);
    if (filePath) params.set('file', filePath);

    fetch(`/api/projects/${encodeURIComponent(projectName)}/diff?${params}`)
      .then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json() as Promise<DiffResponse>;
      })
      .then(data => {
        if (cancelled) return;
        setDiff(data);
        setSelectedFileIdx(0);
        setLoading(false);
      })
      .catch(err => {
        if (cancelled) return;
        setError(String(err));
        setLoading(false);
      });

    return () => { cancelled = true; };
  }, [projectName, filePath, selectedRef, threeWay]);

  // ── Fetch merge base ──────────────────────────────────────────────────────

  const fetchMergeBase = useCallback(async () => {
    setMergeBaseLoading(true);
    try {
      const r = await fetch(
        `/api/projects/${encodeURIComponent(projectName)}/diff/merge-base?upstream=main`,
      );
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      const data = await r.json();
      setMergeBaseSha(data.sha ?? null);
    } catch (e) {
      setMergeBaseSha(null);
    } finally {
      setMergeBaseLoading(false);
    }
  }, [projectName]);

  const handleToggleThreeWay = useCallback(() => {
    if (!threeWay && mergeBaseSha === null && !mergeBaseLoading) {
      fetchMergeBase();
    }
    setThreeWay(v => !v);
  }, [threeWay, mergeBaseSha, mergeBaseLoading, fetchMergeBase]);

  // ── Keyboard navigation ───────────────────────────────────────────────────

  const totalHunks = useMemo(() => {
    if (!diff || threeWay) return 0;
    const file = diff.files[selectedFileIdx];
    return file?.hunks.length ?? 0;
  }, [diff, selectedFileIdx, threeWay]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) {
        return;
      }
      // Only handle when the viewer is in the DOM and visible
      if (!viewerRef.current?.offsetParent) return;

      switch (e.key) {
        case 'n':
        case 'j':
          e.preventDefault();
          setCurrentHunkIndex(i =>
            i == null ? 0 : Math.min(i + 1, totalHunks - 1),
          );
          break;
        case 'p':
        case 'k':
          e.preventDefault();
          setCurrentHunkIndex(i => (i == null || i === 0 ? 0 : i - 1));
          break;
        case 'u':
          e.preventDefault();
          setMode('unified');
          break;
        case 's':
          e.preventDefault();
          setMode('split');
          break;
      }
    },
    [totalHunks],
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  // ── Render helpers ────────────────────────────────────────────────────────

  const currentFile = diff?.files[selectedFileIdx];

  const statsNode =
    !threeWay && diff ? (
      <span className="diff-stats">
        <span className="diff-stat-add">+{diff.total_added}</span>
        <span className="diff-stat-rem">-{diff.total_removed}</span>
        {diff.truncated && <span className="diff-truncated" title="Diff truncated">…trunc</span>}
      </span>
    ) : null;

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div className="diff-viewer" ref={viewerRef}>
      {/* ── Toolbar ── */}
      <div className="diff-toolbar">
        <div className="diff-toolbar-left">
          <span className="diff-title">
            {filePath ? (
              <span className="diff-file-title">{filePath}</span>
            ) : (
              <span className="diff-file-title">All changes</span>
            )}
            {statsNode}
          </span>
        </div>
        <div className="diff-toolbar-right">
          {/* View mode */}
          <div className="diff-view-toggle" role="group" aria-label="View mode">
            <button
              className={`diff-toggle-btn${mode === 'unified' ? ' active' : ''}`}
              onClick={() => setMode('unified')}
              title="Unified view (u)"
            >
              Unified
            </button>
            <button
              className={`diff-toggle-btn${mode === 'split' ? ' active' : ''}`}
              onClick={() => setMode('split')}
              title="Split view (s)"
            >
              Split
            </button>
          </div>

          {/* Three-way toggle */}
          <button
            className={`diff-three-way-btn${threeWay ? ' active' : ''}`}
            onClick={handleToggleThreeWay}
            title="Three-way view: merge-base ↔ HEAD ↔ working tree"
          >
            {mergeBaseLoading ? '…' : '3-way'}
          </button>

          {/* Ref selector (two-way only) */}
          {!threeWay && (
            <div className="diff-ref-selector">
              <select
                className="diff-ref-select"
                value={JSON.stringify(selectedRef)}
                onChange={e => {
                  const v = e.target.value;
                  if (v === '__custom__') return;
                  setSelectedRef(JSON.parse(v));
                  setCustomRef('');
                }}
              >
                {REF_PRESETS.map(p => (
                  <option key={p.label} value={JSON.stringify(p)}>
                    {p.label}
                  </option>
                ))}
                {customRef && (
                  <option value={JSON.stringify({ ref_: customRef })}>
                    vs {customRef}
                  </option>
                )}
                <option value="__custom__">Custom ref…</option>
              </select>
            </div>
          )}

          {/* Hunk navigation */}
          {!threeWay && totalHunks > 0 && (
            <div className="diff-hunk-nav">
              <button
                className="diff-nav-btn"
                onClick={() => setCurrentHunkIndex(i => (i == null || i === 0 ? 0 : i - 1))}
                disabled={currentHunkIndex == null || currentHunkIndex === 0}
                title="Prev hunk (p / k)"
              >
                ‹ prev
              </button>
              <span className="diff-hunk-counter">
                {currentHunkIndex == null ? '—' : `${currentHunkIndex + 1} / ${totalHunks}`}
              </span>
              <button
                className="diff-nav-btn"
                onClick={() =>
                  setCurrentHunkIndex(i => Math.min((i ?? -1) + 1, totalHunks - 1))
                }
                disabled={currentHunkIndex != null && currentHunkIndex >= totalHunks - 1}
                title="Next hunk (n / j)"
              >
                next ›
              </button>
            </div>
          )}
        </div>
      </div>

      {/* ── Custom ref input ── */}
      {!threeWay && (
        <div className="diff-custom-ref-row">
          <label className="diff-custom-ref-label">Compare vs:</label>
          <input
            className="diff-custom-ref-input"
            type="text"
            placeholder="HEAD~5, main, abc1234…"
            value={customRef}
            onChange={e => setCustomRef(e.target.value)}
            onKeyDown={e => {
              if (e.key === 'Enter' && customRef.trim()) {
                setSelectedRef({ ref_: customRef.trim() });
              }
            }}
            spellCheck={false}
          />
          {customRef && (
            <button
              className="diff-custom-ref-apply"
              onClick={() => {
                if (customRef.trim()) setSelectedRef({ ref_: customRef.trim() });
              }}
            >
              Go
            </button>
          )}
        </div>
      )}

      {/* ── File tabs (multi-file diffs) ── */}
      {!threeWay && diff && diff.files.length > 1 && (
        <div className="diff-file-tabs">
          {diff.files.map((f, idx) => (
            <button
              key={idx}
              className={`diff-file-tab${selectedFileIdx === idx ? ' active' : ''}`}
              onClick={() => {
                setSelectedFileIdx(idx);
                setCurrentHunkIndex(null);
              }}
              title={f.new_path || f.old_path}
            >
              <span className="diff-file-tab-name">
                {(f.new_path || f.old_path).split('/').pop()}
              </span>
              <span className="diff-file-tab-stats">
                {f.added > 0 && <span className="diff-stat-add">+{f.added}</span>}
                {f.removed > 0 && <span className="diff-stat-rem">-{f.removed}</span>}
              </span>
            </button>
          ))}
        </div>
      )}

      {/* ── Diff body ── */}
      <div className="diff-body">
        {threeWay ? (
          mergeBaseSha ? (
            <ThreeWayPanel
              projectName={projectName}
              filePath={filePath}
              mergeBaseSha={mergeBaseSha}
              mode={mode}
            />
          ) : mergeBaseLoading ? (
            <div className="diff-loading">Computing merge base…</div>
          ) : (
            <div className="diff-error">
              Could not compute merge base (try selecting a different upstream branch).
            </div>
          )
        ) : loading ? (
          <div className="diff-loading">Loading diff…</div>
        ) : error ? (
          <div className="diff-error">{error}</div>
        ) : !diff || diff.files.length === 0 ? (
          <div className="diff-empty">No changes found for <code>{selectedRef.ref_}</code></div>
        ) : currentFile ? (
          <FileDiffPanel
            file={currentFile}
            mode={mode}
            currentHunkIndex={currentHunkIndex}
            hunkRefs={hunkRefs}
          />
        ) : null}
      </div>

      {/* ── Keyboard hint ── */}
      <div className="diff-keyboard-hint">
        <kbd>n</kbd>/<kbd>j</kbd> next hunk
        <kbd>p</kbd>/<kbd>k</kbd> prev hunk
        <kbd>u</kbd> unified
        <kbd>s</kbd> split
      </div>
    </div>
  );
}
