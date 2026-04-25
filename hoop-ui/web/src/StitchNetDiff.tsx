import { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import { useAtomValue } from 'jotai';
import { conversationsAtom, Conversation } from './atoms';

// ─── Diff API types ───────────────────────────────────────────────────────────

type DiffLineKind = 'context' | 'add' | 'remove';

interface DiffLine {
  kind: DiffLineKind;
  content: string;
  old_lineno: number | null;
  new_lineno: number | null;
}

interface DiffHunk {
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

type ViewMode = 'unified' | 'split';

// ─── Row types & builders ─────────────────────────────────────────────────────

type HunkHeaderRow = { type: 'hunk-header'; header: string; hunkIndex: number };
type ContentRow = {
  type: 'line';
  line?: DiffLine;
  left?: DiffLine | null;
  right?: DiffLine | null;
};
type Row = HunkHeaderRow | ContentRow;

function buildSplitRows(lines: DiffLine[]): ContentRow[] {
  const rows: ContentRow[] = [];
  let removes: DiffLine[] = [];
  let adds: DiffLine[] = [];

  function flush() {
    const n = Math.max(removes.length, adds.length);
    for (let i = 0; i < n; i++) {
      rows.push({ type: 'line', left: removes[i] ?? null, right: adds[i] ?? null });
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

function buildRows(file: FileDiff, mode: ViewMode): Row[] {
  const rows: Row[] = [];
  for (let hi = 0; hi < file.hunks.length; hi++) {
    const hunk = file.hunks[hi];
    rows.push({ type: 'hunk-header', header: hunk.header, hunkIndex: hi });
    if (mode === 'unified') {
      for (const line of hunk.lines) rows.push({ type: 'line', line });
    } else {
      rows.push(...buildSplitRows(hunk.lines));
    }
  }
  return rows;
}

// ─── Constants ────────────────────────────────────────────────────────────────

const ROW_HEIGHT = 22;
const OVERSCAN = 30;
const MOBILE_THRESHOLD = 768;

// ─── VirtualDiffList ──────────────────────────────────────────────────────────

function VirtualDiffList({
  rows,
  mode,
  currentHunkIndex,
  hunkRefs,
}: {
  rows: Row[];
  mode: ViewMode;
  currentHunkIndex: number | null;
  hunkRefs: React.RefObject<Map<number, number>>;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(600);

  useEffect(() => {
    const map = hunkRefs.current!;
    map.clear();
    rows.forEach((row, i) => {
      if (row.type === 'hunk-header') map.set((row as HunkHeaderRow).hunkIndex, i);
    });
  }, [rows, hunkRefs]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const observer = new ResizeObserver(entries => {
      for (const entry of entries) setContainerHeight(entry.contentRect.height);
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (currentHunkIndex == null) return;
    const rowIndex = hunkRefs.current!.get(currentHunkIndex);
    if (rowIndex == null) return;
    containerRef.current?.scrollTo({
      top: Math.max(0, rowIndex * ROW_HEIGHT - 40),
      behavior: 'smooth',
    });
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
              row.type === 'hunk-header' && (row as HunkHeaderRow).hunkIndex === currentHunkIndex;

            if (row.type === 'hunk-header') {
              return (
                <div
                  key={rowIndex}
                  className={`diff-row diff-hunk-header${isCurrentHunk ? ' current-hunk' : ''}`}
                >
                  {(row as HunkHeaderRow).header}
                </div>
              );
            }

            const r = row as ContentRow;
            if (mode === 'unified') {
              const line = r.line!;
              return (
                <div key={rowIndex} className={`diff-row diff-line diff-line--${line.kind}`}>
                  <span className="diff-lineno diff-lineno-old">{line.old_lineno ?? ''}</span>
                  <span className="diff-lineno diff-lineno-new">{line.new_lineno ?? ''}</span>
                  <span className="diff-line-marker">
                    {line.kind === 'add' ? '+' : line.kind === 'remove' ? '-' : ' '}
                  </span>
                  <span className="diff-line-content">{line.content}</span>
                </div>
              );
            } else {
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

// ─── Narrative bar ────────────────────────────────────────────────────────────
// Phase 2: concatenates stitch titles as a placeholder.
// Phase 5+: replace body with agent-synthesized narrative via the agent API.

function NarrativeBar({
  stitches,
  totalAdded,
  totalRemoved,
}: {
  stitches: Conversation[];
  totalAdded: number;
  totalRemoved: number;
}) {
  const [expanded, setExpanded] = useState(false);

  const titles = stitches.filter(s => s.title).map(s => s.title);
  const preview = titles.length > 0 ? titles.slice(0, 3).join(' · ') : 'No active stitches';
  const hasMore = titles.length > 3;

  return (
    <div className="nd-narrative">
      <div className="nd-narrative-header">
        <span className="nd-narrative-badge">Narrative</span>
        <span className="nd-narrative-text">{preview}</span>
        {hasMore && (
          <button
            className="nd-narrative-expand"
            onClick={() => setExpanded(v => !v)}
          >
            {expanded ? '▲ less' : `+${titles.length - 3} more`}
          </button>
        )}
        <div className="nd-narrative-meta">
          <span className="nd-narrative-count">
            {stitches.length} stitch{stitches.length !== 1 ? 'es' : ''}
          </span>
          {(totalAdded > 0 || totalRemoved > 0) && (
            <>
              <span className="diff-stat-add">+{totalAdded}</span>
              <span className="diff-stat-rem">-{totalRemoved}</span>
            </>
          )}
        </div>
      </div>
      {expanded && titles.length > 3 && (
        <ul className="nd-narrative-list">
          {titles.map((t, i) => (
            <li key={i} className="nd-narrative-item">{t}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

// ─── File tree ────────────────────────────────────────────────────────────────

interface FileNode {
  name: string;
  path: string;
  isDir: boolean;
  children: Map<string, FileNode>;
  file?: FileDiff;
}

function buildFileTree(files: FileDiff[]): FileNode {
  const root: FileNode = { name: '', path: '', isDir: true, children: new Map() };
  for (const file of files) {
    const path = file.new_path || file.old_path;
    const parts = path.split('/');
    let node = root;
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      const isLast = i === parts.length - 1;
      if (!node.children.has(part)) {
        node.children.set(part, {
          name: part,
          path: parts.slice(0, i + 1).join('/'),
          isDir: !isLast,
          children: new Map(),
          file: isLast ? file : undefined,
        });
      }
      node = node.children.get(part)!;
    }
  }
  return root;
}

function FileTreeNode({
  node,
  selectedPath,
  onSelect,
  depth = 0,
}: {
  node: FileNode;
  selectedPath: string | null;
  onSelect: (file: FileDiff) => void;
  depth?: number;
}) {
  const [open, setOpen] = useState(true);

  if (node.name === '') {
    return (
      <>
        {Array.from(node.children.values()).map(child => (
          <FileTreeNode
            key={child.name}
            node={child}
            selectedPath={selectedPath}
            onSelect={onSelect}
            depth={depth}
          />
        ))}
      </>
    );
  }

  if (node.isDir) {
    return (
      <div className="nd-tree-dir">
        <button
          className="nd-tree-dir-btn"
          onClick={() => setOpen(v => !v)}
          style={{ paddingLeft: `${depth * 12 + 8}px` }}
        >
          <span className="nd-tree-chevron">{open ? '▾' : '▸'}</span>
          <span className="nd-tree-dir-name">{node.name}/</span>
        </button>
        {open && Array.from(node.children.values()).map(child => (
          <FileTreeNode
            key={child.name}
            node={child}
            selectedPath={selectedPath}
            onSelect={onSelect}
            depth={depth + 1}
          />
        ))}
      </div>
    );
  }

  if (!node.file) return null;

  const path = node.file.new_path || node.file.old_path;
  const isSelected = path === selectedPath;

  return (
    <button
      className={`nd-tree-file${isSelected ? ' nd-tree-file--selected' : ''}`}
      onClick={() => onSelect(node.file!)}
      style={{ paddingLeft: `${depth * 12 + 8}px` }}
      title={path}
    >
      <span className="nd-tree-file-name">{node.name}</span>
      <span className="nd-tree-file-stats">
        {node.file.added > 0 && <span className="diff-stat-add">+{node.file.added}</span>}
        {node.file.removed > 0 && <span className="diff-stat-rem">-{node.file.removed}</span>}
      </span>
    </button>
  );
}

function CollapsibleFileTree({
  files,
  selectedFile,
  onSelect,
  collapsed,
  onToggleCollapse,
}: {
  files: FileDiff[];
  selectedFile: FileDiff | null;
  onSelect: (file: FileDiff) => void;
  collapsed: boolean;
  onToggleCollapse: () => void;
}) {
  const tree = useMemo(() => buildFileTree(files), [files]);
  const selectedPath = selectedFile ? (selectedFile.new_path || selectedFile.old_path) : null;

  return (
    <div className={`nd-file-tree${collapsed ? ' nd-file-tree--collapsed' : ''}`}>
      <div className="nd-file-tree-header">
        {!collapsed && (
          <span className="nd-file-tree-title">Files ({files.length})</span>
        )}
        <button
          className="nd-file-tree-toggle"
          onClick={onToggleCollapse}
          title={collapsed ? 'Expand file tree' : 'Collapse file tree'}
        >
          {collapsed ? '›' : '‹'}
        </button>
      </div>
      {!collapsed && (
        <div className="nd-file-tree-content">
          <FileTreeNode
            node={tree}
            selectedPath={selectedPath}
            onSelect={onSelect}
          />
        </div>
      )}
    </div>
  );
}

// ─── Ref presets ──────────────────────────────────────────────────────────────

const REF_OPTIONS = [
  { label: 'Working vs HEAD', ref_: 'HEAD', ref2: undefined as string | undefined },
  { label: 'HEAD vs HEAD~1', ref_: 'HEAD~1', ref2: 'HEAD' },
  { label: 'HEAD vs main', ref_: 'main', ref2: 'HEAD' },
];

// ─── Main component ───────────────────────────────────────────────────────────

export interface StitchNetDiffProps {
  projectName: string;
  projectPath: string;
  conversations?: Conversation[];
}

export default function StitchNetDiff({
  projectName,
  projectPath,
  conversations: conversationsProp,
}: StitchNetDiffProps) {
  const globalConversations = useAtomValue(conversationsAtom);
  const conversations = conversationsProp ?? globalConversations;

  // Mobile detection (§21)
  const [isMobile, setIsMobile] = useState(
    typeof window !== 'undefined' && window.innerWidth < MOBILE_THRESHOLD,
  );

  // Diff state
  const [diff, setDiff] = useState<DiffResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedRef, setSelectedRef] = useState(REF_OPTIONS[0]);

  // View state
  const [mode, setMode] = useState<ViewMode>('split');
  const [selectedFile, setSelectedFile] = useState<FileDiff | null>(null);
  const [treeCollapsed, setTreeCollapsed] = useState(false);

  // Hunk navigation
  const [currentHunkIndex, setCurrentHunkIndex] = useState<number | null>(null);
  const hunkRefs = useRef(new Map<number, number>());
  const viewerRef = useRef<HTMLDivElement>(null);

  // Stitch data: active (non-complete) conversations scoped to this project
  const projectStitches = useMemo(
    () => conversations.filter(c => c.cwd.startsWith(projectPath) && !c.complete),
    [conversations, projectPath],
  );

  // Derived: rows for the virtual diff list
  const rows = useMemo((): Row[] => {
    if (!selectedFile || selectedFile.is_binary || selectedFile.hunks.length === 0) return [];
    return buildRows(selectedFile, mode);
  }, [selectedFile, mode]);

  const totalHunks = selectedFile?.hunks.length ?? 0;

  // Fetch diff on mount + when selectedRef changes
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    setCurrentHunkIndex(null);

    const params = new URLSearchParams({ ref: selectedRef.ref_ });
    if (selectedRef.ref2) params.set('ref2', selectedRef.ref2);

    fetch(`/api/projects/${encodeURIComponent(projectName)}/diff?${params}`)
      .then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json() as Promise<DiffResponse>;
      })
      .then(data => {
        if (cancelled) return;
        setDiff(data);
        setSelectedFile(data.files[0] ?? null);
        setLoading(false);
      })
      .catch(err => {
        if (cancelled) return;
        setError(String(err));
        setLoading(false);
      });

    return () => { cancelled = true; };
  }, [projectName, selectedRef]);

  // Mobile resize listener
  useEffect(() => {
    const handler = () => setIsMobile(window.innerWidth < MOBILE_THRESHOLD);
    window.addEventListener('resize', handler);
    return () => window.removeEventListener('resize', handler);
  }, []);

  // Keyboard navigation (n/j next hunk, p/k prev hunk, u unified, s split)
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) return;
      if (!viewerRef.current?.offsetParent) return;

      switch (e.key) {
        case 'n':
        case 'j':
          e.preventDefault();
          setCurrentHunkIndex(i => (i == null ? 0 : Math.min(i + 1, totalHunks - 1)));
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

  // Mobile degradation (§21) — all hooks are above this early return
  if (isMobile) {
    return (
      <div className="nd-mobile-message">
        <div className="nd-mobile-icon">🖥</div>
        <h3>View on desktop</h3>
        <p>The Net-Diff review surface requires a wider screen for side-by-side diff rendering.</p>
      </div>
    );
  }

  const selectedPath = selectedFile ? (selectedFile.new_path || selectedFile.old_path) : null;

  return (
    <div className="nd-root" ref={viewerRef}>
      {/* ── Narrative bar ── */}
      <NarrativeBar
        stitches={projectStitches}
        totalAdded={diff?.total_added ?? 0}
        totalRemoved={diff?.total_removed ?? 0}
      />

      {/* ── Main layout: file tree + diff panel ── */}
      <div className="nd-layout">
        {/* File tree */}
        <CollapsibleFileTree
          files={diff?.files ?? []}
          selectedFile={selectedFile}
          onSelect={file => { setSelectedFile(file); setCurrentHunkIndex(null); }}
          collapsed={treeCollapsed}
          onToggleCollapse={() => setTreeCollapsed(v => !v)}
        />

        {/* Diff panel */}
        <div className="nd-diff-panel">
          {/* Toolbar */}
          <div className="nd-diff-toolbar">
            <div className="nd-diff-toolbar-left">
              <span className="nd-diff-file-path" title={selectedPath ?? ''}>
                {selectedPath ?? 'No file selected'}
              </span>
              {selectedFile && (
                <span className="diff-stats">
                  <span className="diff-stat-add">+{selectedFile.added}</span>
                  <span className="diff-stat-rem">-{selectedFile.removed}</span>
                </span>
              )}
              {diff?.truncated && (
                <span className="diff-truncated" title="Diff truncated">…trunc</span>
              )}
            </div>
            <div className="nd-diff-toolbar-right">
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

              <select
                className="diff-ref-select"
                value={JSON.stringify(selectedRef)}
                onChange={e => {
                  setSelectedRef(JSON.parse(e.target.value));
                }}
              >
                {REF_OPTIONS.map(opt => (
                  <option key={opt.label} value={JSON.stringify(opt)}>{opt.label}</option>
                ))}
              </select>

              {totalHunks > 0 && (
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

          {/* Diff body */}
          <div className="nd-diff-body">
            {loading ? (
              <div className="diff-loading">Loading diff…</div>
            ) : error ? (
              <div className="diff-error">{error}</div>
            ) : !selectedFile ? (
              <div className="diff-empty">No changes found</div>
            ) : selectedFile.is_binary ? (
              <div className="diff-binary-notice">Binary file — diff not shown</div>
            ) : selectedFile.hunks.length === 0 ? (
              <div className="diff-empty-notice">No changes in this file</div>
            ) : (
              <VirtualDiffList
                rows={rows}
                mode={mode}
                currentHunkIndex={currentHunkIndex}
                hunkRefs={hunkRefs}
              />
            )}
          </div>

          {/* Keyboard hint */}
          <div className="diff-keyboard-hint">
            <kbd>n</kbd>/<kbd>j</kbd> next hunk
            <kbd>p</kbd>/<kbd>k</kbd> prev hunk
            <kbd>u</kbd> unified
            <kbd>s</kbd> split
          </div>
        </div>
      </div>
    </div>
  );
}
