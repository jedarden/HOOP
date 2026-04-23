import { useAtomValue, useSetAtom } from 'jotai';
import { useState, useMemo, useCallback, useRef, useEffect } from 'react';
import { conversationsAtom, streamingContentAtom, selectedConversationIdAtom } from './atoms';

const PAGE_SIZE = 50;

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
    case 'active':
      return { label: 'Active', className: 'status-stitch-active' };
    case 'awaiting_review':
      return { label: 'Awaiting Review', className: 'status-stitch-review' };
    case 'done':
      return { label: 'Done', className: 'status-stitch-done' };
    default:
      return { label: status, className: 'status-unknown' };
  }
}

interface StitchesTabProps {
  projectName: string;
  projectPath: string;
}

export default function StitchesTab({ projectName, projectPath: _projectPath }: StitchesTabProps) {
  const conversations = useAtomValue(conversationsAtom);
  const streamingContent = useAtomValue(streamingContentAtom);
  const setSelectedConversationId = useSetAtom(selectedConversationIdAtom);

  const [filter, setFilter] = useState<FilterConfig>({
    kind: 'all',
    status: 'all',
    search: '',
  });
  const [page, setPage] = useState(1);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  // Map conversations to stitch items, sorted by last_activity_at (updated_at) DESC
  const stitchItems = useMemo(() => {
    const items = conversations.map(conv => ({
      id: conv.id,
      title: conv.title,
      kind: conv.kind,
      status: conv.complete ? 'done' : 'active',
      createdAt: conv.created_at,
      lastActivityAt: conv.updated_at,
      project: projectName,
      messageCount: conv.messages.length,
      totalTokens: conv.total_tokens,
      linkedBeads: conv.worker_metadata?.bead ? [conv.worker_metadata.bead] : [],
      workerMetadata: conv.worker_metadata,
      isStreaming: streamingContent.has(conv.id),
    }));

    // Reddit-post ranking: most recent activity at top
    return items.sort((a, b) =>
      new Date(b.lastActivityAt).getTime() - new Date(a.lastActivityAt).getTime()
    );
  }, [conversations, streamingContent, projectName]);

  // Reset to first page when filter changes
  useEffect(() => {
    setPage(1);
  }, [filter]);

  const filteredItems = useMemo(() => {
    return stitchItems.filter(item => {
      if (filter.kind !== 'all' && item.kind !== filter.kind) return false;
      if (filter.status !== 'all' && item.status !== filter.status) return false;
      if (filter.search) {
        const q = filter.search.toLowerCase();
        return item.title.toLowerCase().includes(q) || item.id.toLowerCase().includes(q);
      }
      return true;
    });
  }, [stitchItems, filter]);

  const visibleItems = useMemo(() => filteredItems.slice(0, page * PAGE_SIZE), [filteredItems, page]);
  const hasMore = visibleItems.length < filteredItems.length;

  const activeCount = stitchItems.filter(i => i.status === 'active').length;
  const streamingCount = stitchItems.filter(i => i.isStreaming).length;
  const doneCount = stitchItems.filter(i => i.status === 'done').length;

  const handleCardClick = useCallback((id: string) => {
    setSelectedId(prev => {
      const next = prev === id ? null : id;
      if (next) setSelectedConversationId(next);
      return next;
    });
  }, [setSelectedConversationId]);

  // Infinite scroll sentinel
  const sentinelRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!sentinelRef.current || !hasMore) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) setPage(prev => prev + 1);
      },
      { threshold: 0.1 }
    );
    observer.observe(sentinelRef.current);
    return () => observer.disconnect();
  }, [hasMore]);

  return (
    <div className="stitches-tab">
      <div className="stitches-header">
        <div className="stitches-stats">
          <div className="stitch-stat">
            <span className="stat-value">{activeCount}</span>
            <span className="stat-label">Active</span>
          </div>
          {streamingCount > 0 && (
            <div className="stitch-stat stitch-stat--streaming">
              <span className="stat-value">{streamingCount}</span>
              <span className="stat-label">Live</span>
            </div>
          )}
          <div className="stitch-stat">
            <span className="stat-value">0</span>
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
          {visibleItems.map(item => {
            const kindBadge = getKindBadge(item.kind);
            const statusBadge = getStatusBadge(item.status);
            const isSelected = selectedId === item.id;
            const selectedConv = isSelected ? conversations.find(c => c.id === item.id) : undefined;

            return (
              <div
                key={item.id}
                className={[
                  'stitch-card',
                  item.isStreaming ? 'stitch-card--streaming' : '',
                  isSelected ? 'stitch-card--selected' : '',
                ].filter(Boolean).join(' ')}
                onClick={() => handleCardClick(item.id)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => e.key === 'Enter' && handleCardClick(item.id)}
                aria-expanded={isSelected}
              >
                <div className="stitch-card-header">
                  <div className="stitch-title-row">
                    {item.isStreaming && (
                      <span className="stitch-activity-dot" aria-label="Streaming" />
                    )}
                    <h3 className="stitch-title">{item.title}</h3>
                  </div>
                  <div className="stitch-badges">
                    <span className={`badge ${kindBadge.className}`}>{kindBadge.label}</span>
                    <span className={`badge ${statusBadge.className}`}>{statusBadge.label}</span>
                  </div>
                </div>

                <div className="stitch-card-meta">
                  <span className="meta-item stitch-last-activity">
                    <span className="meta-label">Last activity:</span>
                    <span className="meta-value">{formatTimeAgo(item.lastActivityAt)} ago</span>
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
                      <span className="bead-badge">{item.workerMetadata.bead}</span>
                    )}
                  </div>
                )}

                {isSelected && selectedConv && (
                  <div className="stitch-detail" onClick={(e) => e.stopPropagation()}>
                    <div className="stitch-detail-header">
                      <span className="stitch-detail-id">{item.id}</span>
                      <span className="stitch-detail-created">
                        Created {formatTimeAgo(item.createdAt)} ago
                      </span>
                    </div>
                    <div className="stitch-detail-messages">
                      {selectedConv.messages.slice(-5).map((msg, i) => (
                        <div
                          key={i}
                          className={`stitch-detail-message stitch-detail-message--${msg.role}`}
                        >
                          <span className="stitch-detail-role">{msg.role}</span>
                          <span className="stitch-detail-content">
                            {typeof msg.content === 'string'
                              ? msg.content.slice(0, 200) + (msg.content.length > 200 ? '…' : '')
                              : JSON.stringify(msg.content).slice(0, 200)}
                          </span>
                        </div>
                      ))}
                      {selectedConv.messages.length > 5 && (
                        <p className="stitch-detail-truncated">
                          +{selectedConv.messages.length - 5} earlier messages
                        </p>
                      )}
                      {selectedConv.messages.length === 0 && (
                        <p className="stitch-detail-truncated">No messages yet</p>
                      )}
                    </div>
                  </div>
                )}
              </div>
            );
          })}

          {hasMore && (
            <div ref={sentinelRef} className="stitches-load-sentinel">
              <button
                className="stitch-load-more"
                onClick={() => setPage(prev => prev + 1)}
              >
                Load more ({filteredItems.length - visibleItems.length} remaining)
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
