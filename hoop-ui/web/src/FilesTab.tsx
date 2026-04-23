import { useState, useEffect, useCallback } from 'react';

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

export default function FilesTab({ projectName, projectPath }: FilesTabProps) {
  const [rootEntries, setRootEntries] = useState<FileEntry[]>([]);
  const [rootLoading, setRootLoading] = useState(true);
  const [rootError, setRootError] = useState<string | null>(null);

  // Entries fetched for each expanded directory path
  const [childCache, setChildCache] = useState<Map<string, FileEntry[]>>(new Map());
  // Directories currently being expanded (loading children)
  const [loading, setLoading] = useState<Set<string>>(new Set());
  // Directories that have been expanded (show children)
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  const fetchDir = useCallback(
    async (relPath: string): Promise<FileEntry[]> => {
      const url = `/api/projects/${encodeURIComponent(projectName)}/files${relPath ? `?path=${encodeURIComponent(relPath)}` : ''}`;
      const res = await fetch(url);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      return res.json();
    },
    [projectName],
  );

  // Load root on mount
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

      // Already cached — just expand
      if (childCache.has(entry.path)) {
        setExpanded(prev => new Set(prev).add(entry.path));
        return;
      }

      // Fetch children
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

  const handleSelect = useCallback((entry: FileEntry) => {
    setSelectedPath(prev => (prev === entry.path ? null : entry.path));
  }, []);

  return (
    <div className="files-tab">
      <div className="files-header">
        <h4>File Browser</h4>
        <p className="files-path">{projectPath}</p>
      </div>

      <div className="files-content">
        <div className="file-tree">
          {rootLoading && (
            <div className="file-tree-loading" style={{ padding: '1rem' }}>
              Loading…
            </div>
          )}
          {rootError && (
            <div className="file-tree-error" style={{ padding: '1rem' }}>
              {rootError}
            </div>
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

        {selectedPath && (
          <div className="file-preview">
            <div className="file-preview-header">
              <span className="file-preview-path">{selectedPath}</span>
              <button className="file-preview-close" onClick={() => setSelectedPath(null)}>
                ×
              </button>
            </div>
            <div className="file-preview-body">
              <p className="file-preview-placeholder">File preview coming in Phase 3</p>
              <p className="file-preview-hint">
                Syntax-highlighted code with line numbers will appear here
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
