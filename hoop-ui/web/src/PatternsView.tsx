import { useState, useEffect, useCallback } from 'react';
import { ProjectCardData } from './atoms';

// ─── API response types ───────────────────────────────────────────────────────

interface PatternListItem {
  id: string;
  title: string;
  description?: string;
  status: string;
  owner?: string;
  deadline?: string;
  parent_pattern?: string;
  created_at: string;
  updated_at?: string;
  member_count: number;
  closed_member_count: number;
  progress_percent: number;
  total_tokens: number;
}

interface PatternListResponse {
  patterns: PatternListItem[];
}

interface PatternBreadcrumb {
  id: string;
  title: string;
  status: string;
}

interface PatternRow {
  id: string;
  title: string;
  description?: string;
  status: string;
  owner?: string;
  deadline?: string;
  parent_pattern?: string;
  created_at: string;
  updated_at?: string;
}

interface BeadSummary {
  bead_id: string;
  title?: string;
  status?: string;
  relationship: string;
}

interface PatternMemberDetail {
  stitch_id: string;
  project: string;
  kind: string;
  title: string;
  created_at: string;
  last_activity_at: string;
  added_at: string;
  linked_beads: BeadSummary[];
  is_closed: boolean;
  total_tokens: number;
}

interface PatternAggregate {
  total_members: number;
  closed_members: number;
  progress_percent: number;
  total_tokens: number;
  duration_seconds?: number;
}

interface PatternDetailResponse {
  pattern: PatternRow;
  parent_chain: PatternBreadcrumb[];
  members: PatternMemberDetail[];
  aggregate: PatternAggregate;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(0)}k`;
  return String(n);
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m`;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function formatDate(ts: string): string {
  try {
    return new Date(ts).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  } catch {
    return ts;
  }
}

function formatTimeAgo(ts: string): string {
  const secs = Math.floor((Date.now() - new Date(ts).getTime()) / 1000);
  if (secs < 60) return `${secs}s ago`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m ago`;
  if (secs < 86400) return `${Math.floor(secs / 3600)}h ago`;
  return `${Math.floor(secs / 86400)}d ago`;
}

const STATUS_CONFIG: Record<string, { label: string; className: string }> = {
  planned:   { label: 'Planned',   className: 'pv-status-planned'   },
  active:    { label: 'Active',    className: 'pv-status-active'    },
  blocked:   { label: 'Blocked',   className: 'pv-status-blocked'   },
  done:      { label: 'Done',      className: 'pv-status-done'      },
  abandoned: { label: 'Abandoned', className: 'pv-status-abandoned' },
};

function StatusBadge({ status }: { status: string }) {
  const cfg = STATUS_CONFIG[status] ?? { label: status, className: 'pv-status-unknown' };
  return <span className={`pv-status-badge ${cfg.className}`}>{cfg.label}</span>;
}

const KIND_CONFIG: Record<string, { label: string; className: string }> = {
  worker:   { label: 'Worker',   className: 'badge-worker'   },
  operator: { label: 'Operator', className: 'badge-operator' },
  dictated: { label: 'Dictated', className: 'badge-dictated' },
  'ad-hoc': { label: 'Ad-hoc',  className: 'badge-ad-hoc'   },
};

function KindBadge({ kind }: { kind: string }) {
  const cfg = KIND_CONFIG[kind] ?? { label: kind, className: 'badge-unknown' };
  return <span className={`badge ${cfg.className}`}>{cfg.label}</span>;
}

function ProgressBar({ pct }: { pct: number }) {
  const clamped = Math.min(100, Math.max(0, pct));
  const color = clamped === 100 ? '#34a853' : clamped >= 50 ? '#1976d2' : '#9aa0a6';
  return (
    <div className="pv-progress-track" title={`${clamped.toFixed(0)}%`}>
      <div className="pv-progress-fill" style={{ width: `${clamped}%`, background: color }} />
    </div>
  );
}

function projectColor(name: string, cards: ProjectCardData[]): string {
  const card = cards.find(c => c.name === name);
  return card?.color || '#1976d2';
}

function ProjectBadge({ name, cards }: { name: string; cards: ProjectCardData[] }) {
  const color = projectColor(name, cards);
  return (
    <span
      className="pv-project-badge"
      style={{ background: `${color}18`, color, borderColor: `${color}44` }}
    >
      {name}
    </span>
  );
}

// ─── Pattern list ─────────────────────────────────────────────────────────────

function PatternListView({
  patterns,
  loading,
  error,
  onSelect,
}: {
  patterns: PatternListItem[];
  loading: boolean;
  error: string | null;
  onSelect: (id: string) => void;
}) {
  if (loading) {
    return (
      <div className="pv-loading">
        <div className="cpd-loading-bar" style={{ width: '100%', marginBottom: '1rem' }} />
        Loading patterns…
      </div>
    );
  }

  if (error) {
    return <div className="cpd-error">{error}</div>;
  }

  if (patterns.length === 0) {
    return <div className="pv-empty">No patterns yet. Create a pattern to group related stitches.</div>;
  }

  return (
    <div className="pv-list">
      {patterns.map(p => (
        <button key={p.id} className="pv-list-row" onClick={() => onSelect(p.id)}>
          <div className="pv-list-main">
            <div className="pv-list-title-row">
              <span className="pv-list-title">{p.title}</span>
              <StatusBadge status={p.status} />
            </div>
            {p.description && <p className="pv-list-desc">{p.description}</p>}
            <div className="pv-list-meta">
              {p.owner && <span className="pv-meta-item">Owner: <strong>{p.owner}</strong></span>}
              {p.deadline && <span className="pv-meta-item">Due: <strong>{formatDate(p.deadline)}</strong></span>}
              <span className="pv-meta-item">{formatTimeAgo(p.created_at)}</span>
            </div>
          </div>
          <div className="pv-list-stats">
            <div className="pv-stat-col">
              <span className="pv-stat-val">{p.closed_member_count}/{p.member_count}</span>
              <span className="pv-stat-label">stitches done</span>
            </div>
            <div className="pv-stat-col pv-progress-col">
              <ProgressBar pct={p.progress_percent} />
              <span className="pv-stat-label">{p.progress_percent.toFixed(0)}%</span>
            </div>
            <div className="pv-stat-col">
              <span className="pv-stat-val">{formatTokens(p.total_tokens)}</span>
              <span className="pv-stat-label">tokens</span>
            </div>
          </div>
          <span className="pv-list-arrow">›</span>
        </button>
      ))}
    </div>
  );
}

// ─── Breadcrumb ──────────────────────────────────────────────────────────────

function Breadcrumb({
  chain,
  current,
  onNavigate,
}: {
  chain: PatternBreadcrumb[];
  current: string;
  onNavigate: (id: string) => void;
}) {
  if (chain.length === 0) return null;
  return (
    <nav className="pv-breadcrumb">
      <button className="pv-crumb pv-crumb-link" onClick={() => onNavigate('')}>
        All Patterns
      </button>
      {chain.map(crumb => (
        <span key={crumb.id} className="pv-crumb-group">
          <span className="pv-crumb-sep">›</span>
          <button className="pv-crumb pv-crumb-link" onClick={() => onNavigate(crumb.id)}>
            {crumb.title}
          </button>
        </span>
      ))}
      <span className="pv-crumb-sep">›</span>
      <span className="pv-crumb pv-crumb-current">{current}</span>
    </nav>
  );
}

// ─── Pattern detail ───────────────────────────────────────────────────────────

function PatternDetailView({
  detail,
  loading,
  error,
  projectCards,
  onNavigate,
}: {
  detail: PatternDetailResponse | null;
  loading: boolean;
  error: string | null;
  projectCards: ProjectCardData[];
  onNavigate: (id: string) => void;
}) {
  if (loading) {
    return (
      <div className="pv-loading">
        <div className="cpd-loading-bar" style={{ width: '100%', marginBottom: '1rem' }} />
        Loading pattern…
      </div>
    );
  }

  if (error) {
    return <div className="cpd-error">{error}</div>;
  }

  if (!detail) return null;

  const { pattern, parent_chain, members, aggregate } = detail;

  // Group members by project
  const byProject = new Map<string, PatternMemberDetail[]>();
  for (const m of members) {
    if (!byProject.has(m.project)) byProject.set(m.project, []);
    byProject.get(m.project)!.push(m);
  }

  return (
    <div className="pv-detail">
      <Breadcrumb chain={parent_chain} current={pattern.title} onNavigate={onNavigate} />

      <div className="pv-detail-header">
        <div className="pv-detail-title-row">
          <h2 className="pv-detail-title">{pattern.title}</h2>
          <StatusBadge status={pattern.status} />
        </div>
        {pattern.description && <p className="pv-detail-desc">{pattern.description}</p>}
        <div className="pv-detail-meta">
          {pattern.owner && (
            <span className="pv-meta-item">Owner: <strong>{pattern.owner}</strong></span>
          )}
          {pattern.deadline && (
            <span className="pv-meta-item">Deadline: <strong>{formatDate(pattern.deadline)}</strong></span>
          )}
          <span className="pv-meta-item">Created {formatDate(pattern.created_at)}</span>
        </div>
      </div>

      {/* KPI row */}
      <div className="pv-kpis">
        <div className="pv-kpi">
          <span className="pv-kpi-value">{aggregate.total_members}</span>
          <span className="pv-kpi-label">Total Stitches</span>
        </div>
        <div className="pv-kpi">
          <span className="pv-kpi-value">{aggregate.closed_members}</span>
          <span className="pv-kpi-label">Done</span>
        </div>
        <div className="pv-kpi">
          <div className="pv-kpi-progress">
            <ProgressBar pct={aggregate.progress_percent} />
            <span className="pv-kpi-value">{aggregate.progress_percent.toFixed(0)}%</span>
          </div>
          <span className="pv-kpi-label">Progress</span>
        </div>
        <div className="pv-kpi">
          <span className="pv-kpi-value">{formatTokens(aggregate.total_tokens)}</span>
          <span className="pv-kpi-label">Tokens</span>
        </div>
        {aggregate.duration_seconds != null && (
          <div className="pv-kpi">
            <span className="pv-kpi-value">{formatDuration(aggregate.duration_seconds)}</span>
            <span className="pv-kpi-label">Duration</span>
          </div>
        )}
      </div>

      {/* Members by project */}
      {aggregate.total_members === 0 ? (
        <div className="pv-empty">No member stitches yet.</div>
      ) : (
        <div className="pv-groups">
          {Array.from(byProject.entries()).map(([project, projectMembers]) => (
            <div key={project} className="pv-group">
              <div className="pv-group-header">
                <ProjectBadge name={project} cards={projectCards} />
                <span className="pv-group-count">{projectMembers.length} stitch{projectMembers.length !== 1 ? 'es' : ''}</span>
              </div>
              <div className="pv-member-list">
                {projectMembers.map(m => (
                  <MemberRow key={m.stitch_id} member={m} />
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function MemberRow({ member }: { member: PatternMemberDetail }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className={`pv-member ${member.is_closed ? 'pv-member-closed' : 'pv-member-open'}`}>
      <div className="pv-member-main" onClick={() => setExpanded(e => !e)} role="button" tabIndex={0}
           onKeyDown={e => e.key === 'Enter' && setExpanded(v => !v)}>
        <span className="pv-member-closed-icon">{member.is_closed ? '✓' : '○'}</span>
        <div className="pv-member-body">
          <div className="pv-member-title-row">
            <span className="pv-member-title">{member.title || '(untitled)'}</span>
            <KindBadge kind={member.kind} />
          </div>
          <div className="pv-member-sub">
            <span className="pv-meta-item">{formatTimeAgo(member.last_activity_at)}</span>
            <span className="pv-meta-item">{formatTokens(member.total_tokens)} tokens</span>
          </div>
        </div>
        <span className="pv-member-expand">{expanded ? '▲' : '▼'}</span>
      </div>

      {expanded && (
        <div className="pv-member-detail">
          <div className="pv-member-detail-row">
            <span className="pv-detail-key">Stitch ID</span>
            <span className="pv-detail-val pv-mono">{member.stitch_id}</span>
          </div>
          <div className="pv-member-detail-row">
            <span className="pv-detail-key">Added</span>
            <span className="pv-detail-val">{formatDate(member.added_at)}</span>
          </div>
          <div className="pv-member-detail-row">
            <span className="pv-detail-key">Created</span>
            <span className="pv-detail-val">{formatDate(member.created_at)}</span>
          </div>
          {member.linked_beads.length > 0 && (
            <div className="pv-member-detail-row pv-member-beads-row">
              <span className="pv-detail-key">Beads</span>
              <div className="pv-bead-list">
                {member.linked_beads.map(b => (
                  <div key={b.bead_id} className="pv-bead-item">
                    <span className={`pv-bead-dot ${b.status === 'closed' ? 'pv-bead-closed' : 'pv-bead-open'}`} />
                    <span className="pv-mono pv-bead-id">{b.bead_id}</span>
                    {b.title && <span className="pv-bead-title">{b.title}</span>}
                    <span className="pv-bead-rel">{b.relationship}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// ─── Main export ──────────────────────────────────────────────────────────────

interface PatternsViewProps {
  patternId?: string;
  projectCards: ProjectCardData[];
}

export default function PatternsView({ patternId, projectCards }: PatternsViewProps) {
  const [patterns, setPatterns] = useState<PatternListItem[]>([]);
  const [listLoading, setListLoading] = useState(false);
  const [listError, setListError] = useState<string | null>(null);

  const [detail, setDetail] = useState<PatternDetailResponse | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [loadedId, setLoadedId] = useState<string | null>(null);

  // Load list
  useEffect(() => {
    setListLoading(true);
    setListError(null);
    fetch('/api/patterns')
      .then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json() as Promise<PatternListResponse>;
      })
      .then(d => setPatterns(d.patterns))
      .catch(e => setListError(String(e)))
      .finally(() => setListLoading(false));
  }, []);

  // Load detail when patternId changes
  useEffect(() => {
    if (!patternId) {
      setDetail(null);
      setLoadedId(null);
      return;
    }
    if (patternId === loadedId) return;

    setDetailLoading(true);
    setDetailError(null);
    fetch(`/api/patterns/${encodeURIComponent(patternId)}`)
      .then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json() as Promise<PatternDetailResponse>;
      })
      .then(d => {
        setDetail(d);
        setLoadedId(patternId);
      })
      .catch(e => setDetailError(String(e)))
      .finally(() => setDetailLoading(false));
  }, [patternId, loadedId]);

  const navigate = useCallback((id: string) => {
    if (id) {
      window.location.hash = `#/patterns/${id}`;
    } else {
      window.location.hash = '#/patterns';
    }
  }, []);

  return (
    <div className="pv-page">
      <div className="pv-header">
        <div className="pv-title-row">
          <h2>Patterns</h2>
          {patternId && (
            <button className="pv-back-btn" onClick={() => navigate('')}>
              ← All Patterns
            </button>
          )}
        </div>
        {!patternId && (
          <p className="pv-subtitle">
            Patterns group related stitches toward a shared goal, tracking aggregate progress and cost.
          </p>
        )}
      </div>

      {patternId ? (
        <PatternDetailView
          detail={detail}
          loading={detailLoading}
          error={detailError}
          projectCards={projectCards}
          onNavigate={navigate}
        />
      ) : (
        <PatternListView
          patterns={patterns}
          loading={listLoading}
          error={listError}
          onSelect={navigate}
        />
      )}
    </div>
  );
}
