import { useState, useEffect, useCallback, useRef } from 'react';
import { useAtomValue } from 'jotai';
import { projectCardsAtom, type AuditRow, type AuditResponse, type HashChainVerifyResponse } from './atoms';

const PAGE_SIZE = 50;

const ACTION_KINDS = [
  { value: '', label: 'All actions' },
  { value: 'bead_created', label: 'Bead Created' },
  { value: 'stitch_created', label: 'Stitch Created' },
  { value: 'config_changed', label: 'Config Changed' },
  { value: 'config_reloaded', label: 'Config Reloaded' },
  { value: 'config_reload_rejected', label: 'Config Reload Rejected' },
  { value: 'project_added', label: 'Project Added' },
  { value: 'project_removed', label: 'Project Removed' },
  { value: 'draft_created', label: 'Draft Created' },
  { value: 'draft_approved', label: 'Draft Approved' },
  { value: 'draft_edited', label: 'Draft Edited' },
  { value: 'draft_rejected', label: 'Draft Rejected' },
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

function kindLabel(kind: string): string {
  const found = ACTION_KINDS.find(k => k.value === kind);
  return found ? found.label : kind;
}

function resultBadgeClass(result: string): string {
  switch (result) {
    case 'success': return 'audit-result-success';
    case 'failure': return 'audit-result-failure';
    case 'partial': return 'audit-result-partial';
    default: return 'audit-result-unknown';
  }
}

function ArgsCell({ args }: { args: Record<string, unknown> | null }) {
  const [expanded, setExpanded] = useState(false);
  if (!args) return <span className="audit-args-empty">—</span>;

  const keys = Object.keys(args);
  if (keys.length === 0) return <span className="audit-args-empty">{'{}'}</span>;

  const preview = keys.slice(0, 2).map(k => `${k}: ${JSON.stringify(args[k])}`).join(', ');
  const full = JSON.stringify(args, null, 2);

  return (
    <div className="audit-args-cell">
      {expanded ? (
        <>
          <pre className="audit-args-full">{full}</pre>
          <button className="audit-args-toggle" onClick={() => setExpanded(false)}>collapse</button>
        </>
      ) : (
        <>
          <span className="audit-args-preview" title={full}>{preview}{keys.length > 2 ? '…' : ''}</span>
          <button className="audit-args-toggle" onClick={() => setExpanded(true)}>expand</button>
        </>
      )}
    </div>
  );
}

function HashChainIndicator({ valid, message, row_count }: HashChainVerifyResponse) {
  return (
    <div className={`audit-hash-chain ${valid ? 'chain-valid' : 'chain-invalid'}`} title={message}>
      <span className="chain-icon">{valid ? '✓' : '✗'}</span>
      <span className="chain-label">
        {valid ? 'Hash chain intact' : 'Hash chain BROKEN'}
      </span>
      <span className="chain-count">{row_count} rows</span>
      {!valid && <span className="chain-message">{message}</span>}
    </div>
  );
}

interface Filters {
  project: string;
  actor: string;
  kind: string;
}

export default function AuditPanel() {
  const projectCards = useAtomValue(projectCardsAtom);
  const [rows, setRows] = useState<AuditRow[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<Filters>({ project: '', actor: '', kind: '' });
  const [pendingFilters, setPendingFilters] = useState<Filters>({ project: '', actor: '', kind: '' });
  const [chainStatus, setChainStatus] = useState<HashChainVerifyResponse | null>(null);
  const [verifying, setVerifying] = useState(false);
  const actorInputRef = useRef<HTMLInputElement>(null);

  const fetchAudit = useCallback(async (f: Filters, pg: number) => {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams();
      params.set('limit', PAGE_SIZE.toString());
      params.set('offset', (pg * PAGE_SIZE).toString());
      if (f.project) params.set('project', f.project);
      if (f.kind) params.set('kind', f.kind);

      const res = await fetch(`/api/audit?${params}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}: ${await res.text()}`);
      const data: AuditResponse = await res.json();

      // Client-side actor filter (API doesn't support it)
      const filtered = f.actor
        ? data.audit_rows.filter(r => r.actor.toLowerCase().includes(f.actor.toLowerCase()))
        : data.audit_rows;

      setRows(filtered);
      setTotalCount(f.actor ? filtered.length : data.total_count);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAudit(filters, page);
  }, [filters, page, fetchAudit]);

  const verifyChain = useCallback(async () => {
    setVerifying(true);
    try {
      const res = await fetch('/api/audit/verify');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data: HashChainVerifyResponse = await res.json();
      setChainStatus(data);
    } catch (e) {
      setChainStatus({
        valid: false,
        message: e instanceof Error ? e.message : String(e),
        row_count: 0,
      });
    } finally {
      setVerifying(false);
    }
  }, []);

  // Verify hash chain on mount
  useEffect(() => {
    verifyChain();
  }, [verifyChain]);

  const applyFilters = useCallback(() => {
    setFilters(pendingFilters);
    setPage(0);
  }, [pendingFilters]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') applyFilters();
  }, [applyFilters]);

  const totalPages = Math.max(1, Math.ceil(totalCount / PAGE_SIZE));
  const projectNames = projectCards.map(c => c.name);

  return (
    <div className="audit-panel">
      <div className="audit-panel-header">
        <div className="audit-title-row">
          <h2 className="audit-title">Audit Log</h2>
          <div className="audit-header-actions">
            {chainStatus ? (
              <HashChainIndicator {...chainStatus} />
            ) : (
              <div className="audit-hash-chain chain-checking">
                <span className="chain-icon">⋯</span>
                <span className="chain-label">{verifying ? 'Verifying…' : 'Hash chain'}</span>
              </div>
            )}
            <button
              className="audit-refresh-btn"
              onClick={() => { verifyChain(); fetchAudit(filters, page); }}
              disabled={loading || verifying}
              title="Refresh audit log and re-verify hash chain"
            >
              Refresh
            </button>
          </div>
        </div>

        {/* Filter bar */}
        <div className="audit-filters">
          <div className="audit-filter-group">
            <label className="audit-filter-label" htmlFor="audit-filter-project">Project</label>
            <select
              id="audit-filter-project"
              className="audit-filter-select"
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

          <div className="audit-filter-group">
            <label className="audit-filter-label" htmlFor="audit-filter-kind">Action</label>
            <select
              id="audit-filter-kind"
              className="audit-filter-select"
              value={pendingFilters.kind}
              onChange={e => setPendingFilters(prev => ({ ...prev, kind: e.target.value }))}
              onKeyDown={handleKeyDown}
            >
              {ACTION_KINDS.map(k => (
                <option key={k.value} value={k.value}>{k.label}</option>
              ))}
            </select>
          </div>

          <div className="audit-filter-group">
            <label className="audit-filter-label" htmlFor="audit-filter-actor">Actor</label>
            <input
              id="audit-filter-actor"
              ref={actorInputRef}
              className="audit-filter-input"
              type="text"
              placeholder="Filter by actor…"
              value={pendingFilters.actor}
              onChange={e => setPendingFilters(prev => ({ ...prev, actor: e.target.value }))}
              onKeyDown={handleKeyDown}
            />
          </div>

          <button className="audit-filter-apply" onClick={applyFilters}>
            Apply
          </button>
          {(filters.project || filters.actor || filters.kind) && (
            <button
              className="audit-filter-clear"
              onClick={() => {
                const empty = { project: '', actor: '', kind: '' };
                setPendingFilters(empty);
                setFilters(empty);
                setPage(0);
              }}
            >
              Clear
            </button>
          )}
        </div>
      </div>

      {/* Table */}
      <div className="audit-table-container">
        {loading ? (
          <div className="audit-loading">
            <div className="audit-loading-spinner" />
            <span>Loading audit log…</span>
          </div>
        ) : error ? (
          <div className="audit-error">
            <strong>Error:</strong> {error}
          </div>
        ) : rows.length === 0 ? (
          <div className="audit-empty">
            <p>No audit entries found.</p>
            <p className="audit-empty-hint">
              The audit log records writes (bead creates, config changes, etc.).
              Phase 1 has no writes, so this table is empty — it will populate once actions are taken.
            </p>
          </div>
        ) : (
          <table className="audit-table">
            <thead>
              <tr>
                <th className="audit-th audit-th-ts">Timestamp</th>
                <th className="audit-th audit-th-actor">Actor</th>
                <th className="audit-th audit-th-action">Action</th>
                <th className="audit-th audit-th-target">Target</th>
                <th className="audit-th audit-th-args">Args</th>
                <th className="audit-th audit-th-result">Result</th>
              </tr>
            </thead>
            <tbody>
              {rows.map(row => (
                <tr key={row.id} className="audit-row">
                  <td className="audit-td audit-td-ts">
                    <time dateTime={row.ts} title={row.ts}>{formatTs(row.ts)}</time>
                  </td>
                  <td className="audit-td audit-td-actor">
                    <span className="audit-actor">{row.actor}</span>
                  </td>
                  <td className="audit-td audit-td-action">
                    <span className="audit-kind-badge">{kindLabel(row.type)}</span>
                    {row.project && (
                      <span className="audit-project-tag">{row.project}</span>
                    )}
                  </td>
                  <td className="audit-td audit-td-target">
                    <span className="audit-target" title={row.target}>{row.target}</span>
                  </td>
                  <td className="audit-td audit-td-args">
                    <ArgsCell args={row.args} />
                  </td>
                  <td className="audit-td audit-td-result">
                    <span className={`audit-result-badge ${resultBadgeClass(row.result)}`}>
                      {row.result}
                    </span>
                    {row.error && (
                      <span className="audit-error-text" title={row.error}>⚠</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Pagination */}
      {!loading && !error && rows.length > 0 && (
        <div className="audit-pagination">
          <button
            className="audit-page-btn"
            disabled={page === 0}
            onClick={() => setPage(p => Math.max(0, p - 1))}
          >
            &larr; Prev
          </button>
          <span className="audit-page-info">
            Page {page + 1} of {totalPages} &mdash; {totalCount} total
          </span>
          <button
            className="audit-page-btn"
            disabled={page >= totalPages - 1}
            onClick={() => setPage(p => Math.min(totalPages - 1, p + 1))}
          >
            Next &rarr;
          </button>
        </div>
      )}

      <div className="audit-footer-note">
        Read-only view. Audit entries are written by the daemon on each action and protected by a SHA-256 hash chain (§13).
      </div>
    </div>
  );
}
