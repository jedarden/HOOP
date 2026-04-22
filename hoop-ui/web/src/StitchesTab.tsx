import { useAtomValue } from 'jotai';
import { useState, useMemo } from 'react';
import { stitchesAtom, conversationsAtom, beadsAtom, workersAtom } from './atoms';

type StitchKind = 'operator' | 'dictated' | 'worker' | 'ad-hoc' | 'all';
type StitchStatus = 'active' | 'awaiting_review' | 'done' | 'all';

interface FilterConfig {
  kind: StitchKind;
  status: StitchStatus;
  search: string;
}

function formatTimeAgo(timestamp: string): string {
  const now = new Date();
  const then = new Date(timestamp);
  const seconds = Math.floor((now.getTime() - then.getTime()) / 1000);

  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86400)}d`;
}

function getKindBadge(kind: string): { label: string; className: string } {
  switch (kind) {
    case 'worker':
      return { label: 'Worker', className: 'badge-worker' };
    case 'operator':
      return { label: 'Operator', className: 'badge-operator' };
    case 'dictated':
      return { label: 'Dictated', className: 'badge-dictated' };
    case 'ad-hoc':
      return { label: 'Ad-hoc', className: 'badge-ad-hoc' };
    default:
      return { label: kind, className: 'badge-unknown' };
  }
}

function getStatusBadge(status: string): { label: string; className: string } {
  switch (status) {
    case 'pending':
      return { label: 'In Progress', className: 'status-in-progress' };
    case 'running':
      return { label: 'Running', className: 'status-running' };
    case 'completed':
      return { label: 'Done', className: 'status-done' };
    case 'failed':
      return { label: 'Failed', className: 'status-failed' };
    default:
      return { label: status, className: 'status-unknown' };
  }
}

interface StitchesTabProps {
  projectName: string;
  projectPath: string;
}

export default function StitchesTab({ projectName, projectPath }: StitchesTabProps) {
  const stitches = useAtomValue(stitchesAtom);
  const conversations = useAtomValue(conversationsAtom);
  const beads = useAtomValue(beadsAtom);
  const workers = useAtomValue(workersAtom);

  const [filter, setFilter] = useState<FilterConfig>({
    kind: 'all',
    status: 'all',
    search: '',
  });

  // Combine stitches and conversations for unified view
  const stitchItems = useMemo(() => {
    // For now, use conversations as stitch items
    // This will be updated when we have proper stitch data from the backend
    return conversations.map(conv => ({
      id: conv.id,
      title: conv.title,
      kind: conv.kind,
      status: conv.complete ? 'done' : 'active',
      createdAt: conv.created_at,
      updatedAt: conv.updated_at,
      project: projectName,
      participantCount: 1,
      messageCount: conv.messages.length,
      totalTokens: conv.total_tokens,
      linkedBeads: conv.worker_metadata?.bead ? [conv.worker_metadata.bead] : [],
      workerMetadata: conv.worker_metadata,
    }));
  }, [conversations, projectName]);

  // Filter stitch items
  const filteredItems = useMemo(() => {
    return stitchItems.filter(item => {
      if (filter.kind !== 'all' && item.kind !== filter.kind) return false;
      if (filter.status !== 'all' && item.status !== filter.status) return false;
      if (filter.search) {
        const searchLower = filter.search.toLowerCase();
        return (
          item.title.toLowerCase().includes(searchLower) ||
          item.id.toLowerCase().includes(searchLower)
        );
      }
      return true;
    });
  }, [stitchItems, filter]);

  const activeCount = stitchItems.filter(i => i.status === 'active').length;
  const awaitingReviewCount = 0; // Will be calculated from review-kind beads
  const doneCount = stitchItems.filter(i => i.status === 'done').length;

  return (
    <div className="stitches-tab">
      <div className="stitches-header">
        <div className="stitches-stats">
          <div className="stitch-stat">
            <span className="stat-value">{activeCount}</span>
            <span className="stat-label">Active</span>
          </div>
          <div className="stitch-stat">
            <span className="stat-value">{awaitingReviewCount}</span>
            <span className="stat-label">Awaiting Review</span>
          </div>
          <div className="stitch-stat">
            <span className="stat-value">{doneCount}</span>
            <span className="stat-label">Done</span>
          </div>
        </div>

        <div className="stitches-filters">
          <input
            type="text"
            placeholder="Search stitches..."
            value={filter.search}
            onChange={(e) => setFilter(prev => ({ ...prev, search: e.target.value }))}
            className="stitch-search"
          />

          <select
            value={filter.kind}
            onChange={(e) => setFilter(prev => ({ ...prev, kind: e.target.value as StitchKind }))}
            className="stitch-filter-select"
          >
            <option value="all">All Kinds</option>
            <option value="operator">Operator</option>
            <option value="worker">Worker</option>
            <option value="dictated">Dictated</option>
            <option value="ad-hoc">Ad-hoc</option>
          </select>

          <select
            value={filter.status}
            onChange={(e) => setFilter(prev => ({ ...prev, status: e.target.value as StitchStatus }))}
            className="stitch-filter-select"
          >
            <option value="all">All Status</option>
            <option value="active">Active</option>
            <option value="awaiting_review">Awaiting Review</option>
            <option value="done">Done</option>
          </select>
        </div>
      </div>

      {filteredItems.length === 0 ? (
        <div className="stitches-empty">
          <p>No stitches found</p>
          {filter.search || filter.kind !== 'all' || filter.status !== 'all' ? (
            <button
              className="clear-filters-button"
              onClick={() => setFilter({ kind: 'all', status: 'all', search: '' })}
            >
              Clear filters
            </button>
          ) : (
            <p className="empty-hint">Stitches will appear here as conversations happen</p>
          )}
        </div>
      ) : (
        <div className="stitches-list">
          {filteredItems.map(item => {
            const kindBadge = getKindBadge(item.kind);
            const statusBadge = getStatusBadge(item.status);

            return (
              <div key={item.id} className="stitch-card">
                <div className="stitch-card-header">
                  <h3 className="stitch-title">{item.title}</h3>
                  <div className="stitch-badges">
                    <span className={`badge ${kindBadge.className}`}>{kindBadge.label}</span>
                    <span className={`badge ${statusBadge.className}`}>{statusBadge.label}</span>
                  </div>
                </div>

                <div className="stitch-card-meta">
                  <span className="meta-item">
                    <span className="meta-label">Updated:</span>
                    <span className="meta-value">{formatTimeAgo(item.updatedAt)} ago</span>
                  </span>
                  <span className="meta-item">
                    <span className="meta-label">Messages:</span>
                    <span className="meta-value">{item.messageCount}</span>
                  </span>
                  {item.totalTokens > 0 && (
                    <span className="meta-item">
                      <span className="meta-label">Tokens:</span>
                      <span className="meta-value">{item.totalTokens.toLocaleString()}</span>
                    </span>
                  )}
                  {item.linkedBeads.length > 0 && (
                    <span className="meta-item">
                      <span className="meta-label">Beads:</span>
                      <span className="meta-value">{item.linkedBeads.length}</span>
                    </span>
                  )}
                </div>

                {item.workerMetadata && (
                  <div className="stitch-worker-info">
                    <span className="worker-badge">
                      Worker: {item.workerMetadata.worker}
                    </span>
                    {item.workerMetadata.bead && (
                      <span className="bead-badge">
                        {item.workerMetadata.bead}
                      </span>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
