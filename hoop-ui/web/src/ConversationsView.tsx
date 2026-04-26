import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { useAtomValue } from 'jotai';
import { projectCardsAtom } from './atoms';

// Types matching the backend API
interface ConversationSummary {
  id: string;
  session_id: string;
  provider: string;
  kind: string;
  project: string;
  cwd: string;
  title: string;
  message_count: number;
  total_tokens: number;
  created_at: string;
  updated_at: string;
  complete: boolean;
  worker_metadata?: WorkerMetadata;
}

interface WorkerMetadata {
  worker: string;
  bead: string;
  strand?: string | null;
}

interface ConversationsResponse {
  conversations: ConversationSummary[];
  next_cursor: string | null;
  has_more: boolean;
}

interface ConversationsQueryParams {
  cursor?: string;
  limit?: number;
  project?: string;
  provider?: string;
  kind?: string;
  fleet?: boolean;
  search?: string;
  after?: string;
  before?: string;
  sort?: string;
  order?: string;
}

// URL parameter parsing helpers
function parseUrlParams(): ConversationsQueryParams {
  const params = new URLSearchParams(window.location.hash.split('?')[1] || '');
  const result: ConversationsQueryParams = {};

  if (params.has('cursor')) result.cursor = params.get('cursor')!;
  if (params.has('limit')) result.limit = parseInt(params.get('limit')!, 10);
  if (params.has('project')) result.project = params.get('project')!;
  if (params.has('provider')) result.provider = params.get('provider')!;
  if (params.has('kind')) result.kind = params.get('kind')!;
  if (params.has('fleet')) result.fleet = params.get('fleet') === 'true';
  if (params.has('search')) result.search = params.get('search')!;
  if (params.has('after')) result.after = params.get('after')!;
  if (params.has('before')) result.before = params.get('before')!;
  if (params.has('sort')) result.sort = params.get('sort')!;
  if (params.has('order')) result.order = params.get('order')!;

  return result;
}

function updateUrlParams(params: ConversationsQueryParams) {
  const urlParams = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    if (value !== undefined && value !== null && value !== '') {
      urlParams.set(key, String(value));
    }
  });

  const hash = '#/conversations';
  const queryString = urlParams.toString();
  window.location.hash = queryString ? `${hash}?${queryString}` : hash;
}

// Format helpers
function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);

  if (diffMins < 1) return 'just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  return date.toLocaleDateString();
}

function formatTokens(tokens: number): string {
  if (tokens < 1000) return tokens.toString();
  return `${(tokens / 1000).toFixed(1)}k`;
}

function getKindBadge(kind: string): { label: string; className: string } {
  switch (kind) {
    case 'worker':
      return { label: 'Fleet', className: 'badge-fleet' };
    case 'operator':
      return { label: 'Operator', className: 'badge-operator' };
    case 'dictated':
      return { label: 'Dictated', className: 'badge-dictated' };
    case 'ad-hoc':
      return { label: 'Ad-hoc', className: 'badge-ad-hoc' };
    default:
      return { label: kind, className: 'badge-ad-hoc' };
  }
}

// Virtualized list implementation
interface VirtualizedListProps {
  items: ConversationSummary[];
  itemHeight: number;
  containerHeight: number;
  renderItem: (item: ConversationSummary, index: number) => React.ReactNode;
  overscan?: number;
}

function VirtualizedList({
  items,
  itemHeight,
  containerHeight,
  renderItem,
  overscan = 3,
}: VirtualizedListProps) {
  const [scrollTop, setScrollTop] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);

  const totalHeight = items.length * itemHeight;
  const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
  const endIndex = Math.min(
    items.length - 1,
    Math.floor((scrollTop + containerHeight) / itemHeight) + overscan
  );

  const visibleItems = items.slice(startIndex, endIndex + 1);

  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  }, []);

  return (
    <div
      ref={containerRef}
      className="conversations-virtual-list"
      style={{ height: `${containerHeight}px`, overflowY: 'auto' }}
      onScroll={handleScroll}
    >
      <div style={{ height: `${totalHeight}px`, position: 'relative' }}>
        <div
          style={{
            transform: `translateY(${startIndex * itemHeight}px)`,
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
          }}
        >
          {visibleItems.map((item, index) => (
            <div
              key={item.id}
              style={{ height: `${itemHeight}px` }}
              className="conversations-list-item-wrapper"
            >
              {renderItem(item, startIndex + index)}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

// Conversation row component
function ConversationRow({
  conversation,
  onClick,
  index,
}: {
  conversation: ConversationSummary;
  onClick: () => void;
  index: number;
}) {
  const badge = getKindBadge(conversation.kind);

  return (
    <div
      className="conversations-list-item"
      onClick={onClick}
      style={{ position: 'absolute', top: 0, left: 0, right: 0, bottom: 0 }}
    >
      <div className="conversations-item-header">
        <span className={`badge ${badge.className} badge-sm`}>{badge.label}</span>
        <span className="conversations-item-time">
          {formatTimestamp(conversation.updated_at)}
        </span>
        <span className="conversations-item-provider">{conversation.provider}</span>
      </div>
      <h4 className="conversations-item-title">{conversation.title || conversation.cwd}</h4>
      <div className="conversations-item-meta">
        <span className="conversations-item-project" title={conversation.cwd}>
          {conversation.project}
        </span>
        <span className="conversations-item-tokens">
          {formatTokens(conversation.total_tokens)} tokens
        </span>
        <span className="conversations-item-messages">
          {conversation.message_count} messages
        </span>
      </div>
      {conversation.worker_metadata && (
        <div className="conversations-item-worker">
          <span className="conversations-item-worker-label">worker:</span>
          <span className="conversations-item-worker-name">{conversation.worker_metadata.worker}</span>
          {conversation.worker_metadata.bead && (
            <span className="conversations-item-bead">{conversation.worker_metadata.bead}</span>
          )}
        </div>
      )}
      {!conversation.complete && (
        <div className="conversations-item-status">
          <span className="status-dot live" />
          <span>live</span>
        </div>
      )}
      <div className="conversations-item-index">#{index + 1}</div>
    </div>
  );
}

// Filters component
function ConversationsFilters({
  params,
  projects,
  onChange,
}: {
  params: ConversationsQueryParams;
  projects: { name: string; label: string }[];
  onChange: (updates: Partial<ConversationsQueryParams>) => void;
}) {
  return (
    <div className="conversations-filters">
      <div className="conversations-filter-group">
        <label>Project</label>
        <select
          value={params.project || ''}
          onChange={(e) => onChange({ project: e.target.value || undefined })}
        >
          <option value="">All Projects</option>
          {projects.map((p) => (
            <option key={p.name} value={p.name}>
              {p.label || p.name}
            </option>
          ))}
        </select>
      </div>

      <div className="conversations-filter-group">
        <label>Provider</label>
        <select
          value={params.provider || ''}
          onChange={(e) => onChange({ provider: e.target.value || undefined })}
        >
          <option value="">All Providers</option>
          <option value="claude">Claude</option>
          <option value="codex">Codex</option>
          <option value="gemini">Gemini</option>
          <option value="opencode">OpenCode</option>
          <option value="aider">Aider</option>
        </select>
      </div>

      <div className="conversations-filter-group">
        <label>Kind</label>
        <select
          value={params.kind || ''}
          onChange={(e) => onChange({ kind: e.target.value || undefined })}
        >
          <option value="">All Kinds</option>
          <option value="worker">Fleet</option>
          <option value="operator">Operator</option>
          <option value="ad-hoc">Ad-hoc</option>
          <option value="dictated">Dictated</option>
        </select>
      </div>

      <div className="conversations-filter-group">
        <label>Fleet vs Ad-hoc</label>
        <select
          value={params.fleet === undefined ? '' : params.fleet ? 'fleet' : 'ad-hoc'}
          onChange={(e) => {
            const value = e.target.value;
            onChange({
              fleet: value === '' ? undefined : value === 'fleet',
            });
          }}
        >
          <option value="">All</option>
          <option value="fleet">Fleet Only</option>
          <option value="ad-hoc">Ad-hoc Only</option>
        </select>
      </div>

      <div className="conversations-filter-group">
        <label>Search</label>
        <input
          type="text"
          placeholder="Search title or path..."
          value={params.search || ''}
          onChange={(e) => onChange({ search: e.target.value || undefined })}
        />
      </div>

      <div className="conversations-filter-group">
        <label>After</label>
        <input
          type="datetime-local"
          value={params.after?.replace('Z', '') || ''}
          onChange={(e) => onChange({ after: e.target.value ? e.target.value + 'Z' : undefined })}
        />
      </div>

      <div className="conversations-filter-group">
        <label>Before</label>
        <input
          type="datetime-local"
          value={params.before?.replace('Z', '') || ''}
          onChange={(e) => onChange({ before: e.target.value ? e.target.value + 'Z' : undefined })}
        />
      </div>

      <div className="conversations-filter-group">
        <label>Sort</label>
        <select
          value={params.sort || 'updated_at'}
          onChange={(e) => onChange({ sort: e.target.value || undefined })}
        >
          <option value="updated_at">Updated</option>
          <option value="created_at">Created</option>
          <option value="title">Title</option>
        </select>
      </div>

      <div className="conversations-filter-group">
        <label>Order</label>
        <select
          value={params.order || 'desc'}
          onChange={(e) => onChange({ order: e.target.value || undefined })}
        >
          <option value="desc">Descending</option>
          <option value="asc">Ascending</option>
        </select>
      </div>

      <button
        className="conversations-filter-clear"
        onClick={() =>
          onChange({
            project: undefined,
            provider: undefined,
            kind: undefined,
            fleet: undefined,
            search: undefined,
            after: undefined,
            before: undefined,
            sort: undefined,
            order: undefined,
          })
        }
      >
        Clear Filters
      </button>
    </div>
  );
}

// Main ConversationsView component
export default function ConversationsView() {
  const projectCards = useAtomValue(projectCardsAtom);

  // Parse URL params on mount and sync with state
  const [urlParams, setUrlParams] = useState<ConversationsQueryParams>(parseUrlParams);
  const [conversations, setConversations] = useState<ConversationSummary[]>([]);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [totalCount, setTotalCount] = useState(0);
  const [selectedConversation, setSelectedConversation] = useState<ConversationSummary | null>(null);

  // Update URL when params change
  const updateParams = useCallback((updates: Partial<ConversationsQueryParams>) => {
    setUrlParams((prev) => {
      const newParams = { ...prev, ...updates, cursor: undefined }; // Reset cursor on filter change
      updateUrlParams(newParams);
      return newParams;
    });
    setSelectedConversation(null);
  }, []);

  // Fetch conversations
  const fetchConversations = useCallback(async (cursor?: string) => {
    setLoading(true);
    setError(null);

    try {
      const queryParams: ConversationsQueryParams = {
        ...urlParams,
        limit: 50,
        cursor,
      };

      const searchParams = new URLSearchParams();
      Object.entries(queryParams).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== '') {
          searchParams.set(key, String(value));
        }
      });

      const response = await fetch(`/api/conversations?${searchParams.toString()}`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data: ConversationsResponse = await response.json();

      if (cursor) {
        setConversations((prev) => [...prev, ...data.conversations]);
      } else {
        setConversations(data.conversations);
        setTotalCount(data.conversations.length);
      }

      setNextCursor(data.next_cursor);
      setHasMore(data.has_more);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, [urlParams]);

  // Initial fetch and refetch on filter change
  useEffect(() => {
    fetchConversations();
  }, [fetchConversations]);

  // Load more handler
  const loadMore = useCallback(() => {
    if (hasMore && nextCursor && !loading) {
      fetchConversations(nextCursor);
    }
  }, [hasMore, nextCursor, loading, fetchConversations]);

  // Sync URL params on hash change (back/forward navigation)
  useEffect(() => {
    const handleHashChange = () => {
      const newParams = parseUrlParams();
      setUrlParams(newParams);
    };
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  // Virtualized list configuration
  const itemHeight = 100; // Estimated height per conversation row
  const containerHeight = 600; // Fixed viewport height

  const projects = useMemo(
    () =>
      projectCards.map((p) => ({
        name: p.name,
        label: p.label,
      })),
    [projectCards]
  );

  return (
    <div className="conversations-view">
      <div className="conversations-header">
        <h2>Conversations</h2>
        <div className="conversations-count">
          {loading && !conversations.length ? 'Loading...' : `${totalCount} conversations`}
          {conversations.length > 0 && hasMore && ` (showing ${conversations.length})`}
        </div>
      </div>

      <ConversationsFilters params={urlParams} projects={projects} onChange={updateParams} />

      {error && <div className="conversations-error">{error}</div>}

      <div className="conversations-content">
        <div className="conversations-list-panel">
          {conversations.length === 0 && !loading ? (
            <div className="conversations-empty">No conversations found matching your filters.</div>
          ) : (
            <VirtualizedList
              items={conversations}
              itemHeight={itemHeight}
              containerHeight={containerHeight}
              renderItem={(conversation, index) => (
                <ConversationRow
                  key={conversation.id}
                  conversation={conversation}
                  index={index}
                  onClick={() => setSelectedConversation(conversation)}
                />
              )}
            />
          )}

          {hasMore && (
            <div className="conversations-load-more">
              <button
                onClick={loadMore}
                disabled={loading}
                className="conversations-load-more-button"
              >
                {loading ? 'Loading...' : 'Load More'}
              </button>
            </div>
          )}
        </div>

        <div className="conversations-detail-panel">
          {selectedConversation ? (
            <ConversationDetail conversation={selectedConversation} />
          ) : (
            <div className="conversations-detail-empty">
              <p>Select a conversation to view details</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// Conversation detail panel
function ConversationDetail({ conversation }: { conversation: ConversationSummary }) {
  const badge = getKindBadge(conversation.kind);

  return (
    <div className="conversations-detail">
      <div className="conversations-detail-header">
        <h3>{conversation.title || conversation.cwd}</h3>
        <span className={`badge ${badge.className}`}>{badge.label}</span>
      </div>

      <div className="conversations-detail-meta">
        <div className="conversations-detail-row">
          <span className="conversations-detail-label">Provider:</span>
          <span className="conversations-detail-value">{conversation.provider}</span>
        </div>

        <div className="conversations-detail-row">
          <span className="conversations-detail-label">Project:</span>
          <span className="conversations-detail-value">{conversation.project}</span>
        </div>

        <div className="conversations-detail-row">
          <span className="conversations-detail-label">Path:</span>
          <span className="conversations-detail-value conversations-detail-path">{conversation.cwd}</span>
        </div>

        <div className="conversations-detail-row">
          <span className="conversations-detail-label">Tokens:</span>
          <span className="conversations-detail-value">{formatTokens(conversation.total_tokens)}</span>
        </div>

        <div className="conversations-detail-row">
          <span className="conversations-detail-label">Messages:</span>
          <span className="conversations-detail-value">{conversation.message_count}</span>
        </div>

        <div className="conversations-detail-row">
          <span className="conversations-detail-label">Created:</span>
          <span className="conversations-detail-value">{new Date(conversation.created_at).toLocaleString()}</span>
        </div>

        <div className="conversations-detail-row">
          <span className="conversations-detail-label">Updated:</span>
          <span className="conversations-detail-value">{new Date(conversation.updated_at).toLocaleString()}</span>
        </div>

        {conversation.worker_metadata && (
          <>
            <div className="conversations-detail-row">
              <span className="conversations-detail-label">Worker:</span>
              <span className="conversations-detail-value">{conversation.worker_metadata.worker}</span>
            </div>

            <div className="conversations-detail-row">
              <span className="conversations-detail-label">Bead:</span>
              <span className="conversations-detail-value">{conversation.worker_metadata.bead}</span>
            </div>

            {conversation.worker_metadata.strand && (
              <div className="conversations-detail-row">
                <span className="conversations-detail-label">Strand:</span>
                <span className="conversations-detail-value">{conversation.worker_metadata.strand}</span>
              </div>
            )}
          </>
        )}

        {!conversation.complete && (
          <div className="conversations-detail-status">
            <span className="status-dot live" />
            <span>Session in progress</span>
          </div>
        )}
      </div>

      <div className="conversations-detail-actions">
        <a
          href={`#/fleet`}
          className="conversations-detail-link"
        >
          View in Fleet
        </a>
      </div>
    </div>
  );
}
