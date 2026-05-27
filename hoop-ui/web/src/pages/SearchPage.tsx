import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { useAtomValue } from 'jotai';
import {
  projectCardsAtom,
  beadsAtom,
  conversationsAtom,
  type SessionKind,
} from './atoms';

const PAGE_SIZE = 100;

// Search result types
interface SearchResult {
  kind: 'project' | 'bead' | 'conversation';
  id: string;
  title: string;
  snippet: string;
  project?: string;
  href?: string;
  createdAt: string;
  // Bead-specific
  status?: 'open' | 'closed';
  priority?: number;
  issueType?: string;
  // Conversation-specific
  provider?: string;
  sessionKind?: SessionKind;
  complete?: boolean;
  totalTokens?: number;
  messageCount?: number;
  workerName?: string;
  // Project-specific
  label?: string;
  beadCount?: number;
  workerCount?: number;
}

// URL parameter types
interface SearchParams {
  q: string;
  project: string[];
  kind: string[];
  status: string[];
  provider: string[];
  adapter: string[];
  after: string;
  before: string;
  page: number;
}

// Parse URL params from hash
function parseUrlParams(): SearchParams {
  const hash = window.location.hash;
  const queryStart = hash.indexOf('?');
  if (queryStart === -1) {
    return {
      q: '',
      project: [],
      kind: [],
      status: [],
      provider: [],
      adapter: [],
      after: '',
      before: '',
      page: 0,
    };
  }

  const params = new URLSearchParams(hash.slice(queryStart + 1));
  return {
    q: params.get('q') || '',
    project: params.getAll('project'),
    kind: params.getAll('kind'),
    status: params.getAll('status'),
    provider: params.getAll('provider'),
    adapter: params.getAll('adapter'),
    after: params.get('after') || '',
    before: params.get('before') || '',
    page: parseInt(params.get('page') || '0', 10),
  };
}

// Update URL with current params
function updateUrlParams(params: SearchParams) {
  const urlParams = new URLSearchParams();
  if (params.q) urlParams.set('q', params.q);
  params.project.forEach(p => urlParams.append('project', p));
  params.kind.forEach(k => urlParams.append('kind', k));
  params.status.forEach(s => urlParams.append('status', s));
  params.provider.forEach(p => urlParams.append('provider', p));
  params.adapter.forEach(a => urlParams.append('adapter', a));
  if (params.after) urlParams.set('after', params.after);
  if (params.before) urlParams.set('before', params.before);
  urlParams.set('page', params.page.toString());

  const hash = urlParams.toString() ? `#/search?${urlParams.toString()}` : '#/search';
  window.location.hash = hash;
}

// Get snippet with query highlighting
function getSnippet(text: string, query: string): string {
  if (!query) return text.slice(0, 120);

  const lowerText = text.toLowerCase();
  const lowerQuery = query.toLowerCase();
  const idx = lowerText.indexOf(lowerQuery);
  if (idx === -1) return text.slice(0, 120);

  const start = Math.max(0, idx - 40);
  const end = Math.min(text.length, idx + query.length + 40);
  let snippet = text.slice(start, end);
  if (start > 0) snippet = '…' + snippet;
  if (end < text.length) snippet = snippet + '…';
  return snippet;
}

// Format timestamp
function formatTimestamp(ts: string): string {
  try {
    const d = new Date(ts);
    return d.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  } catch {
    return ts;
  }
}

// Format tokens
function formatTokens(tokens: number): string {
  if (tokens < 1000) return tokens.toString();
  return `${(tokens / 1000).toFixed(1)}k`;
}

// Get kind badge info
function getKindBadge(kind: SessionKind): { label: string; className: string } {
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

// Checkbox filter component
function CheckboxFilter({
  label,
  options,
  selected,
  onChange,
}: {
  label: string;
  options: { value: string; label: string; count?: number }[];
  selected: string[];
  onChange: (values: string[]) => void;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const toggleOption = (value: string) => {
    if (selected.includes(value)) {
      onChange(selected.filter(v => v !== value));
    } else {
      onChange([...selected, value]);
    }
  };

  const selectedCount = selected.length;

  return (
    <div ref={containerRef} className="search-filter-dropdown">
      <button
        className={`search-filter-button ${selectedCount > 0 ? 'search-filter-button-active' : ''}`}
        onClick={() => setIsOpen(!isOpen)}
        type="button"
      >
        {label}
        {selectedCount > 0 && (
          <span className="search-filter-badge">{selectedCount}</span>
        )}
        <span className="search-filter-arrow">{isOpen ? '▴' : '▾'}</span>
      </button>
      {isOpen && (
        <div className="search-filter-menu">
          <div className="search-filter-header">
            <button
              className="search-filter-action"
              onClick={() => onChange(options.map(o => o.value))}
              type="button"
            >
              All
            </button>
            <button
              className="search-filter-action"
              onClick={() => onChange([])}
              type="button"
            >
              None
            </button>
          </div>
          <div className="search-filter-options">
            {options.map(option => (
              <label key={option.value} className="search-filter-option">
                <input
                  type="checkbox"
                  checked={selected.includes(option.value)}
                  onChange={() => toggleOption(option.value)}
                />
                <span className="search-filter-option-label">{option.label}</span>
                {option.count !== undefined && (
                  <span className="search-filter-option-count">{option.count}</span>
                )}
              </label>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// Main SearchPage component
export default function SearchPage() {
  const projectCards = useAtomValue(projectCardsAtom);
  const beads = useAtomValue(beadsAtom);
  const conversations = useAtomValue(conversationsAtom);

  // URL state
  const [params, setParams] = useState<SearchParams>(parseUrlParams);

  // Derived state for filters
  const availableProjects = useMemo(() => {
    const projectNames = new Set<string>();
    projectCards.forEach(p => projectNames.add(p.name));
    beads.forEach(b => projectNames.add(b.project));
    conversations.forEach(c => {
      const projectKey = c.cwd.split('/').pop() || c.cwd;
      projectNames.add(projectKey);
    });
    return Array.from(projectNames).sort();
  }, [projectCards, beads, conversations]);

  const availableKinds = useMemo(() => {
    const kinds = new Set<SessionKind>();
    conversations.forEach(c => kinds.add(c.kind));
    return Array.from(kinds).sort();
  }, [conversations]);

  const availableProviders = useMemo(() => {
    const providers = new Set<string>();
    conversations.forEach(c => providers.add(c.provider));
    return Array.from(providers).sort();
  }, [conversations]);

  const availableAdapters = useMemo(() => {
    const adapters = new Set<string>();
    conversations.forEach(c => adapters.add(c.provider));
    return Array.from(adapters).sort();
  }, [conversations]);

  // Perform search (client-side)
  const searchResults = useMemo((): SearchResult[] => {
    const query = params.q.toLowerCase().trim();
    const projectFilters = new Set(params.project);
    const kindFilters = new Set(params.kind as SessionKind[]);
    const statusFilters = new Set(params.status);
    const providerFilters = new Set(params.provider);
    const adapterFilters = new Set(params.adapter);

    const after = params.after ? new Date(params.after) : null;
    const before = params.before ? new Date(params.before) : null;

    const results: SearchResult[] = [];

    // Helper to check date range
    const inDateRange = (dateStr: string) => {
      const date = new Date(dateStr);
      if (after && date < after) return false;
      if (before && date > before) return false;
      return true;
    };

    // Helper to check text match
    const matchesText = (text: string) => !query || text.toLowerCase().includes(query);

    // Search project cards
    projectCards.forEach(p => {
      if (projectFilters.size > 0 && !projectFilters.has(p.name)) return;
      if (!matchesText(`${p.name} ${p.label}`)) return;
      const activityDate = p.last_activity || '';
      if (after && activityDate && new Date(activityDate) < after) return;
      if (before && activityDate && new Date(activityDate) > before) return;

      results.push({
        kind: 'project',
        id: p.name,
        title: p.label || p.name,
        snippet: `${p.bead_count} beads · ${p.worker_count} workers`,
        project: p.name,
        href: `#/${p.name}`,
        createdAt: activityDate,
        label: p.label,
        beadCount: p.bead_count,
        workerCount: p.worker_count,
      });
    });

    // Search beads
    beads.forEach(b => {
      if (projectFilters.size > 0 && !projectFilters.has(b.project)) return;
      if (!matchesText(b.title)) return;
      if (!inDateRange(b.created_at)) return;
      if (statusFilters.size > 0 && !statusFilters.has(b.status)) return;

      results.push({
        kind: 'bead',
        id: b.id,
        title: b.title,
        snippet: getSnippet(b.title, params.q),
        project: b.project,
        href: `#/${b.project}`,
        createdAt: b.created_at,
        status: b.status,
        priority: b.priority,
        issueType: b.issue_type,
      });
    });

    // Search conversations
    conversations.forEach(c => {
      const projectKey = c.cwd.split('/').pop() || c.cwd;
      if (projectFilters.size > 0 && !projectFilters.has(projectKey)) return;
      if (!inDateRange(c.created_at)) return;
      if (kindFilters.size > 0 && !kindFilters.has(c.kind)) return;
      if (providerFilters.size > 0 && !providerFilters.has(c.provider)) return;
      if (adapterFilters.size > 0 && !adapterFilters.has(c.provider)) return;

      if (matchesText(c.title)) {
        results.push({
          kind: 'conversation',
          id: c.id,
          title: c.title,
          snippet: getSnippet(c.title, params.q),
          project: projectKey,
          href: `#/fleet`,
          createdAt: c.created_at,
          provider: c.provider,
          sessionKind: c.kind,
          complete: c.complete,
          totalTokens: c.total_tokens,
          messageCount: c.messages.length,
          workerName: c.worker_metadata?.worker,
        });
        return;
      }

      // Search in message bodies if there's a text query
      if (!query) return;

      for (const msg of c.messages) {
        const content = typeof msg.content === 'string' ? msg.content : '';
        if (content.toLowerCase().includes(query)) {
          results.push({
            kind: 'conversation',
            id: c.id,
            title: c.title,
            snippet: getSnippet(content, params.q),
            project: projectKey,
            href: `#/fleet`,
            createdAt: c.created_at,
            provider: c.provider,
            sessionKind: c.kind,
            complete: c.complete,
            totalTokens: c.total_tokens,
            messageCount: c.messages.length,
            workerName: c.worker_metadata?.worker,
          });
          break;
        }
      }
    });

    // Sort by created date (newest first)
    return results.sort((a, b) => {
      const dateA = new Date(a.createdAt).getTime();
      const dateB = new Date(b.createdAt).getTime();
      return dateB - dateA;
    });
  }, [params, projectCards, beads, conversations]);

  // Paginated results
  const totalPages = Math.max(1, Math.ceil(searchResults.length / PAGE_SIZE));
  const paginatedResults = useMemo(() => {
    const start = params.page * PAGE_SIZE;
    return searchResults.slice(start, start + PAGE_SIZE);
  }, [searchResults, params.page]);

  // Update URL when params change
  const updateParams = useCallback((updates: Partial<SearchParams>) => {
    setParams(prev => {
      const newParams = { ...prev, ...updates, page: updates.page ?? 0 };
      updateUrlParams(newParams);
      return newParams;
    });
  }, []);

  // Sync URL params on hash change (back/forward navigation)
  useEffect(() => {
    const handleHashChange = () => {
      const newParams = parseUrlParams();
      setParams(newParams);
    };
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  const hasActiveFilters =
    params.q ||
    params.project.length > 0 ||
    params.kind.length > 0 ||
    params.status.length > 0 ||
    params.provider.length > 0 ||
    params.adapter.length > 0 ||
    params.after ||
    params.before;

  return (
    <div className="search-page">
      <div className="search-page-header">
        <h2 className="search-page-title">Search</h2>
        <div className="search-page-count">
          {searchResults.length} result{searchResults.length !== 1 ? 's' : ''}
          {totalPages > 1 && ` · page ${params.page + 1} of ${totalPages}`}
        </div>
      </div>

      {/* Search input */}
      <div className="search-input-row">
        <svg className="search-input-icon" width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <circle cx="6.5" cy="6.5" r="4.5" stroke="currentColor" strokeWidth="1.5"/>
          <path d="M10 10L14 14" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
        </svg>
        <input
          className="search-input-field"
          type="text"
          placeholder="Search across all projects, beads, and conversations…"
          value={params.q}
          onChange={e => updateParams({ q: e.target.value })}
          autoComplete="off"
          spellCheck={false}
        />
        {hasActiveFilters && (
          <button
            className="search-clear-btn"
            onClick={() => updateParams({
              q: '',
              project: [],
              kind: [],
              status: [],
              provider: [],
              adapter: [],
              after: '',
              before: '',
              page: 0,
            })}
            type="button"
          >
            Clear all
          </button>
        )}
      </div>

      {/* Facet filters */}
      <div className="search-facets">
        <CheckboxFilter
          label="Project"
          options={availableProjects.map(p => ({ value: p, label: p }))}
          selected={params.project}
          onChange={values => updateParams({ project: values })}
        />

        <CheckboxFilter
          label="Kind"
          options={availableKinds.map(k => ({
            value: k,
            label: getKindBadge(k).label,
          }))}
          selected={params.kind}
          onChange={values => updateParams({ kind: values })}
        />

        <CheckboxFilter
          label="Status"
          options={[
            { value: 'open', label: 'Open' },
            { value: 'closed', label: 'Closed' },
          ]}
          selected={params.status}
          onChange={values => updateParams({ status: values })}
        />

        <CheckboxFilter
          label="Provider"
          options={availableProviders.map(p => ({ value: p, label: p }))}
          selected={params.provider}
          onChange={values => updateParams({ provider: values })}
        />

        <CheckboxFilter
          label="Adapter"
          options={availableAdapters.map(a => ({ value: a, label: a }))}
          selected={params.adapter}
          onChange={values => updateParams({ adapter: values })}
        />

        <div className="search-date-filter">
          <label className="search-date-label">After</label>
          <input
            className="search-date-input"
            type="date"
            value={params.after || ''}
            onChange={e => updateParams({ after: e.target.value })}
          />
        </div>

        <div className="search-date-filter">
          <label className="search-date-label">Before</label>
          <input
            className="search-date-input"
            type="date"
            value={params.before || ''}
            onChange={e => updateParams({ before: e.target.value })}
          />
        </div>
      </div>

      {/* Results */}
      <div className="search-results-container">
        {paginatedResults.length === 0 ? (
          <div className="search-empty">
            {hasActiveFilters
              ? 'No results match your filters.'
              : 'Enter a search query or apply filters to find results.'}
          </div>
        ) : (
          <>
            <div className="search-results-list">
              {paginatedResults.map((result, idx) => (
                <a
                  key={`${result.kind}-${result.id}-${idx}`}
                  href={result.href}
                  className={`search-result-item search-result-${result.kind}`}
                  onClick={(e) => {
                    e.preventDefault();
                    window.location.hash = result.href!.slice(1);
                  }}
                >
                  <span className={`search-result-badge search-result-badge-${result.kind}`}>
                    {result.kind}
                  </span>
                  <div className="search-result-content">
                    <h3 className="search-result-title">{result.title}</h3>
                    <p className="search-result-snippet">{result.snippet}</p>
                    <div className="search-result-meta">
                      {result.project && (
                        <span className="search-result-project" title={result.project}>
                          {result.project}
                        </span>
                      )}
                      {result.kind === 'bead' && result.status && (
                        <span className={`search-result-status search-result-status-${result.status}`}>
                          {result.status}
                        </span>
                      )}
                      {result.kind === 'bead' && result.issueType && (
                        <span className="search-result-issue-type">{result.issueType}</span>
                      )}
                      {result.kind === 'bead' && result.priority !== undefined && (
                        <span className="search-result-priority">P{result.priority}</span>
                      )}
                      {result.kind === 'conversation' && result.sessionKind && (
                        <span className={`badge badge-sm ${getKindBadge(result.sessionKind).className}`}>
                          {getKindBadge(result.sessionKind).label}
                        </span>
                      )}
                      {result.kind === 'conversation' && result.provider && (
                        <span className="search-result-provider">{result.provider}</span>
                      )}
                      {result.kind === 'conversation' && result.workerName && (
                        <span className="search-result-worker">{result.workerName}</span>
                      )}
                      {result.kind === 'conversation' && result.totalTokens !== undefined && (
                        <span className="search-result-tokens">
                          {formatTokens(result.totalTokens)} tokens
                        </span>
                      )}
                      {result.kind === 'project' && result.beadCount !== undefined && (
                        <span className="search-result-bead-count">
                          {result.beadCount} beads
                        </span>
                      )}
                      <time className="search-result-date" dateTime={result.createdAt}>
                        {formatTimestamp(result.createdAt)}
                      </time>
                    </div>
                  </div>
                </a>
              ))}
            </div>

            {/* Pagination */}
            {totalPages > 1 && (
              <div className="search-pagination">
                <button
                  className="search-page-btn"
                  disabled={params.page === 0}
                  onClick={() => updateParams({ page: params.page - 1 })}
                  type="button"
                >
                  ← Previous
                </button>
                <div className="search-page-numbers">
                  {Array.from({ length: Math.min(7, totalPages) }, (_, i) => {
                    let pageNum;
                    if (totalPages <= 7) {
                      pageNum = i;
                    } else if (params.page < 3) {
                      pageNum = i;
                    } else if (params.page > totalPages - 4) {
                      pageNum = totalPages - 7 + i;
                    } else {
                      pageNum = params.page - 3 + i;
                    }

                    return (
                      <button
                        key={pageNum}
                        className={`search-page-number ${pageNum === params.page ? 'search-page-number-current' : ''}`}
                        onClick={() => updateParams({ page: pageNum })}
                        type="button"
                      >
                        {pageNum + 1}
                      </button>
                    );
                  })}
                </div>
                <button
                  className="search-page-btn"
                  disabled={params.page >= totalPages - 1}
                  onClick={() => updateParams({ page: params.page + 1 })}
                  type="button"
                >
                  Next →
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
