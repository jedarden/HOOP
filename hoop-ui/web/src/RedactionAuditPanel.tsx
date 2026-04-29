import { useState, useEffect, useCallback } from 'react';
import { useAtomValue } from 'jotai';
import { projectCardsAtom, type RedactionAuditRow, type RedactionAuditResponse } from './atoms';

const PAGE_SIZE = 50;

const WHAT_FLAGGED_OPTIONS = [
  { value: '', label: 'All types' },
  { value: 'attachment', label: 'Attachment' },
  { value: 'transcript', label: 'Transcript' },
  { value: 'session_filter', label: 'Session Filter' },
  { value: 'draft', label: 'Draft' },
  { value: 'morning_brief', label: 'Morning Brief' },
];

const ACTION_OPTIONS = [
  { value: '', label: 'All actions' },
  { value: 'flagged_only', label: 'Flagged Only' },
  { value: 'redacted_in_place', label: 'Redacted In Place' },
  { value: 'redacted_and_deleted', label: 'Redacted and Deleted' },
  { value: 'proceeded_anyway', label: 'Proceeded Anyway' },
];

function formatTs(ts: string): string {
  try {
    const d = new Date(ts);
    return d.toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  } catch {
    return ts;
  }
}

function actionLabel(action: string): string {
  const found = ACTION_OPTIONS.find(a => a.value === action);
  return found ? found.label : action;
}

function actionBadgeClass(action: string): string {
  switch (action) {
    case 'flagged_only': return 'redaction-action-flagged';
    case 'redacted_in_place': return 'redaction-action-redacted-in-place';
    case 'redacted_and_deleted': return 'redaction-action-deleted';
    case 'proceeded_anyway': return 'redaction-action-proceeded';
    default: return 'redaction-action-unknown';
  }
}

function MetadataCell({ metadata }: { metadata: Record<string, unknown> | null }) {
  if (!metadata) return <span className="redaction-metadata-empty">—</span>;

  const keys = Object.keys(metadata);
  if (keys.length === 0) return <span className="redaction-metadata-empty">{'{}'}</span>;

  const preview = keys.slice(0, 2).map(k => `${k}: ${JSON.stringify(metadata[k])}`).join(', ');

  return (
    <div className="redaction-metadata-cell">
      <span className="redaction-metadata-preview" title={JSON.stringify(metadata, null, 2)}>
        {preview}{keys.length > 2 ? '…' : ''}
      </span>
    </div>
  );
}

interface Filters {
  project: string;
  pattern: string;
  operator: string;
  whatFlagged: string;
  action: string;
}

export default function RedactionAuditPanel() {
  const projectCards = useAtomValue(projectCardsAtom);
  const [rows, setRows] = useState<RedactionAuditRow[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<Filters>({
    project: '',
    pattern: '',
    operator: '',
    whatFlagged: '',
    action: '',
  });
  const [pendingFilters, setPendingFilters] = useState<Filters>({
    project: '',
    pattern: '',
    operator: '',
    whatFlagged: '',
    action: '',
  });

  const fetchRedactionAudit = useCallback(async (f: Filters, pg: number) => {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams();
      params.set('limit', PAGE_SIZE.toString());
      params.set('offset', (pg * PAGE_SIZE).toString());
      if (f.project) params.set('project', f.project);
      if (f.pattern) params.set('pattern', f.pattern);
      if (f.operator) params.set('operator', f.operator);
      if (f.whatFlagged) params.set('what_flagged', f.whatFlagged);
      if (f.action) params.set('action', f.action);

      const res = await fetch(`/api/redaction-audit?${params}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}: ${await res.text()}`);
      const data: RedactionAuditResponse = await res.json();

      setRows(data.audit_rows);
      setTotalCount(data.total_count);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchRedactionAudit(filters, page);
  }, [filters, page, fetchRedactionAudit]);

  const applyFilters = useCallback(() => {
    setFilters(pendingFilters);
    setPage(0);
  }, [pendingFilters]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') applyFilters();
  }, [applyFilters]);

  const clearFilters = useCallback(() => {
    const empty = {
      project: '',
      pattern: '',
      operator: '',
      whatFlagged: '',
      action: '',
    };
    setPendingFilters(empty);
    setFilters(empty);
    setPage(0);
  }, []);

  const totalPages = Math.max(1, Math.ceil(totalCount / PAGE_SIZE));
  const projectNames = projectCards.map(c => c.name);

  return (
    <div className="redaction-audit-panel">
      <div className="redaction-audit-panel-header">
        <div className="redaction-audit-title-row">
          <h2 className="redaction-audit-title">Redaction Audit Log</h2>
          <div className="redaction-audit-header-actions">
            <button
              className="redaction-audit-refresh-btn"
              onClick={() => fetchRedactionAudit(filters, page)}
              disabled={loading}
              title="Refresh redaction audit log"
            >
              Refresh
            </button>
          </div>
        </div>

        {/* Filter bar */}
        <div className="redaction-audit-filters">
          <div className="redaction-audit-filter-group">
            <label className="redaction-audit-filter-label" htmlFor="redaction-filter-project">Project</label>
            <select
              id="redaction-filter-project"
              className="redaction-audit-filter-select"
              value={pendingFilters.project}
              onChange={e => setPendingFilters(prev => ({ ...prev, project: e.target.value }))}
              onKeyDown={handleKeyDown}
            >
              <option value="">All projects</option>
              {projectNames.map(name => (
                <option key={name} value={name}>{name}</option>
              ))}
            </select>
          </div>

          <div className="redaction-audit-filter-group">
            <label className="redaction-audit-filter-label" htmlFor="redaction-filter-what">Type</label>
            <select
              id="redaction-filter-what"
              className="redaction-audit-filter-select"
              value={pendingFilters.whatFlagged}
              onChange={e => setPendingFilters(prev => ({ ...prev, whatFlagged: e.target.value }))}
              onKeyDown={handleKeyDown}
            >
              {WHAT_FLAGGED_OPTIONS.map(o => (
                <option key={o.value} value={o.value}>{o.label}</option>
              ))}
            </select>
          </div>

          <div className="redaction-audit-filter-group">
            <label className="redaction-audit-filter-label" htmlFor="redaction-filter-action">Action</label>
            <select
              id="redaction-filter-action"
              className="redaction-audit-filter-select"
              value={pendingFilters.action}
              onChange={e => setPendingFilters(prev => ({ ...prev, action: e.target.value }))}
              onKeyDown={handleKeyDown}
            >
              {ACTION_OPTIONS.map(o => (
                <option key={o.value} value={o.value}>{o.label}</option>
              ))}
            </select>
          </div>

          <div className="redaction-audit-filter-group">
            <label className="redaction-audit-filter-label" htmlFor="redaction-filter-pattern">Pattern</label>
            <input
              id="redaction-filter-pattern"
              className="redaction-audit-filter-input"
              type="text"
              placeholder="Filter by pattern…"
              value={pendingFilters.pattern}
              onChange={e => setPendingFilters(prev => ({ ...prev, pattern: e.target.value }))}
              onKeyDown={handleKeyDown}
            />
          </div>

          <div className="redaction-audit-filter-group">
            <label className="redaction-audit-filter-label" htmlFor="redaction-filter-operator">Operator</label>
            <input
              id="redaction-filter-operator"
              className="redaction-audit-filter-input"
              type="text"
              placeholder="Filter by operator…"
              value={pendingFilters.operator}
              onChange={e => setPendingFilters(prev => ({ ...prev, operator: e.target.value }))}
              onKeyDown={handleKeyDown}
            />
          </div>

          <button className="redaction-audit-filter-apply" onClick={applyFilters}>
            Apply
          </button>
          {(filters.project || filters.pattern || filters.operator || filters.whatFlagged || filters.action) && (
            <button className="redaction-audit-filter-clear" onClick={clearFilters}>
              Clear
            </button>
          )}
        </div>
      </div>

      {/* Table */}
      <div className="redaction-audit-table-container">
        {loading ? (
          <div className="redaction-audit-loading">
            <div className="redaction-audit-loading-spinner" />
            <span>Loading redaction audit log…</span>
          </div>
        ) : error ? (
          <div className="redaction-audit-error">
            <strong>Error:</strong> {error}
          </div>
        ) : rows.length === 0 ? (
          <div className="redaction-audit-empty">
            <p>No redaction audit entries found.</p>
            <p className="redaction-audit-empty-hint">
              Redaction audit entries are written when secrets are detected in transcripts,
              attachments, drafts, or other content.
            </p>
          </div>
        ) : (
          <table className="redaction-audit-table">
            <thead>
              <tr>
                <th className="redaction-audit-th redaction-audit-th-ts">Timestamp</th>
                <th className="redaction-audit-th redaction-audit-th-what">Type</th>
                <th className="redaction-audit-th redaction-audit-th-pattern">Pattern</th>
                <th className="redaction-audit-th redaction-audit-th-action">Action</th>
                <th className="redaction-audit-th redaction-audit-th-operator">Operator</th>
                <th className="redaction-audit-th redaction-audit-th-source">Source</th>
                <th className="redaction-audit-th redaction-audit-th-metadata">Metadata</th>
              </tr>
            </thead>
            <tbody>
              {rows.map(row => (
                <tr key={row.id} className="redaction-audit-row">
                  <td className="redaction-audit-td redaction-audit-td-ts">
                    <time dateTime={row.ts} title={row.ts}>{formatTs(row.ts)}</time>
                  </td>
                  <td className="redaction-audit-td redaction-audit-td-what">
                    <span className="redaction-what-badge">{row.what_flagged}</span>
                    {row.project && (
                      <span className="redaction-project-tag">{row.project}</span>
                    )}
                  </td>
                  <td className="redaction-audit-td redaction-audit-td-pattern">
                    <span className="redaction-pattern-name">{row.pattern_name}</span>
                  </td>
                  <td className="redaction-audit-td redaction-audit-td-action">
                    <span className={`redaction-action-badge ${actionBadgeClass(row.action)}`}>
                      {actionLabel(row.action)}
                    </span>
                  </td>
                  <td className="redaction-audit-td redaction-audit-td-operator">
                    <span className="redaction-operator">{row.operator}</span>
                  </td>
                  <td className="redaction-audit-td redaction-audit-td-source">
                    <span className="redaction-source-ref" title={row.source_ref}>
                      {row.source_ref.length > 30 ? row.source_ref.slice(0, 30) + '…' : row.source_ref}
                    </span>
                  </td>
                  <td className="redaction-audit-td redaction-audit-td-metadata">
                    <MetadataCell metadata={row.metadata} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Pagination */}
      {!loading && !error && rows.length > 0 && (
        <div className="redaction-audit-pagination">
          <button
            className="redaction-audit-page-btn"
            disabled={page === 0}
            onClick={() => setPage(p => Math.max(0, p - 1))}
          >
            &larr; Prev
          </button>
          <span className="redaction-audit-page-info">
            Page {page + 1} of {totalPages} &mdash; {totalCount} total
          </span>
          <button
            className="redaction-audit-page-btn"
            disabled={page >= totalPages - 1}
            onClick={() => setPage(p => Math.min(totalPages - 1, p + 1))}
          >
            Next &rarr;
          </button>
        </div>
      )}

      <div className="redaction-audit-footer-note">
        Read-only view. Redaction audit entries track all secret detection events per §18.5.
        Retention follows the backup policy.
      </div>
    </div>
  );
}
