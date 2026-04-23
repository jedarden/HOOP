import React, { useState } from 'react';

interface FilesTabProps {
  projectPath: string;
}

interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  size?: number;
  modified?: string;
  children?: FileNode[];
}

export default function FilesTab({ projectPath }: FilesTabProps) {
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());

  // Placeholder file tree - in real implementation, this would come from the backend
  const fileTree: FileNode = {
    name: projectPath.split('/').pop() || 'project',
    path: projectPath,
    type: 'directory',
    children: [
      {
        name: '.beads',
        path: `${projectPath}/.beads`,
        type: 'directory',
        children: [
          { name: 'beads.db', path: `${projectPath}/.beads/beads.db`, type: 'file', size: 102400 },
          { name: 'events.jsonl', path: `${projectPath}/.beads/events.jsonl`, type: 'file', size: 51200 },
        ],
      },
      {
        name: 'src',
        path: `${projectPath}/src`,
        type: 'directory',
        children: [
          { name: 'main.rs', path: `${projectPath}/src/main.rs`, type: 'file', size: 2048 },
          { name: 'lib.rs', path: `${projectPath}/src/lib.rs`, type: 'file', size: 1536 },
        ],
      },
      {
        name: 'Cargo.toml',
        path: `${projectPath}/Cargo.toml`,
        type: 'file',
        size: 512,
      },
      {
        name: 'README.md',
        path: `${projectPath}/README.md`,
        type: 'file',
        size: 3072,
      },
    ],
  };

  function toggleDirectory(path: string) {
    setExpandedDirs(prev => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function renderFileNode(node: FileNode, depth: number = 0): React.ReactElement {
    const isExpanded = expandedDirs.has(node.path);
    const isSelected = selectedPath === node.path;

    return (
      <div key={node.path}>
        <div
          className={`file-tree-node ${isSelected ? 'selected' : ''}`}
          style={{ paddingLeft: `${depth * 16 + 8}px` }}
          onClick={() => {
            if (node.type === 'directory') {
              toggleDirectory(node.path);
            } else {
              setSelectedPath(node.path);
            }
          }}
        >
          {node.type === 'directory' && (
            <span className="file-expand-icon">{isExpanded ? '▼' : '▶'}</span>
          )}
          <span className={`file-icon file-icon-${node.type}`}>
            {node.type === 'directory' ? '📁' : '📄'}
          </span>
          <span className="file-name">{node.name}</span>
          {node.size !== undefined && (
            <span className="file-size">{formatSize(node.size)}</span>
          )}
        </div>
        {node.type === 'directory' && isExpanded && node.children && (
          <div className="file-children">
            {node.children.map(child => renderFileNode(child, depth + 1))}
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="files-tab">
      <div className="files-header">
        <h4>File Browser</h4>
        <p className="files-path">{projectPath}</p>
      </div>

      <div className="files-content">
        <div className="file-tree">
          {renderFileNode(fileTree)}
        </div>

        {selectedPath && (
          <div className="file-preview">
            <div className="file-preview-header">
              <span className="file-preview-path">{selectedPath}</span>
              <button
                className="file-preview-close"
                onClick={() => setSelectedPath(null)}
              >
                ×
              </button>
            </div>
            <div className="file-preview-body">
              <p className="file-preview-placeholder">
                File preview will be implemented in Phase 3
              </p>
              <p className="file-preview-hint">
                This will show syntax-highlighted code with line numbers
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
