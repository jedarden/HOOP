import { useState, useEffect, useCallback } from 'react';
import {
  CrossProjectDashboardData,
  ProjectCardData,
} from './atoms';

interface CrossProjectDashboardProps {
  projectCards: ProjectCardData[];
  onNavigateProject: (name: string) => void;
}

type TimeRange = 'today' | 'week' | 'month';

const RANGE_LABELS: Record<TimeRange, string> = {
  today: 'Today',
  week: 'Last 7 days',
  month: 'This month',
};

function formatCost(usd: number): string {
  if (usd === 0) return '$0.00';
  if (usd < 0.01) return '<$0.01';
  return `$${usd.toFixed(2)}`;
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m`;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function Bar({ value, max, color }: { value: number; max: number; color?: string }) {
  const pct = max > 0 ? (value / max) * 100 : 0;
  return (
    <div className="cpd-bar-track">
      <div
        className="cpd-bar-fill"
        style={{ width: `${pct}%`, background: color ?? 'var(--accent, #3b82f6)' }}
      />
    </div>
  );
}

export default function CrossProjectDashboard({
  projectCards,
  onNavigateProject,
}: CrossProjectDashboardProps) {
  const [range, setRange] = useState<TimeRange>('today');
  const [data, setData] = useState<CrossProjectDashboardData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchDashboard = useCallback((r: TimeRange) => {
    setLoading(true);
    setError(null);
    fetch(`/api/dashboard/cross-project?range=${r}`)
      .then(res => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json() as Promise<CrossProjectDashboardData>;
      })
      .then(d => {
        setData(d);
        setLoading(false);
      })
      .catch(e => {
        setError(String(e));
        setLoading(false);
      });
  }, []);

  useEffect(() => {
    fetchDashboard(range);
    const id = setInterval(() => fetchDashboard(range), 30_000);
    return () => clearInterval(id);
  }, [range, fetchDashboard]);

  const handleRangeChange = (r: TimeRange) => {
    setRange(r);
  };

  const maxProjectSpend = data
    ? Math.max(...data.spend_by_project.map(p => p.cost_usd), 0.001)
    : 0;
  const maxAdapterSpend = data
    ? Math.max(...data.spend_by_adapter.map(a => a.cost_usd), 0.001)
    : 0;

  // Derive per-project worker counts from projectCards for supplemental display
  // (the API also returns workers_by_project but projectCards is live)
  const workersByProject = projectCards
    .filter(c => c.worker_count > 0)
    .sort((a, b) => b.worker_count - a.worker_count);

  const totalWorkersLive = projectCards.reduce((s, c) => s + c.worker_count, 0);

  // Map project name -> color from projectCards
  const projectColor = (name: string): string | undefined =>
    projectCards.find(c => c.name === name)?.color;

  return (
    <div className="cpd-page">
      <div className="cpd-header">
        <div className="cpd-title-row">
          <h2>Cross-Project Dashboard</h2>
          <div className="cpd-range-tabs" role="tablist" aria-label="Time range">
            {(['today', 'week', 'month'] as TimeRange[]).map(r => (
              <button
                key={r}
                role="tab"
                aria-selected={range === r}
                className={`cpd-range-tab ${range === r ? 'cpd-range-tab-active' : ''}`}
                onClick={() => handleRangeChange(r)}
              >
                {RANGE_LABELS[r]}
              </button>
            ))}
          </div>
        </div>
        {loading && <div className="cpd-loading-bar" aria-label="Loading…" />}
        {error && <div className="cpd-error" role="alert">Error loading dashboard: {error}</div>}
      </div>

      {data && (
        <div className="cpd-body">
          {/* Top-level KPIs */}
          <section className="cpd-kpis">
            <div className="cpd-kpi">
              <span className="cpd-kpi-value">{formatCost(data.total_spend_usd)}</span>
              <span className="cpd-kpi-label">Total spend · {data.range_label}</span>
            </div>
            <div className="cpd-kpi">
              <span className="cpd-kpi-value">{totalWorkersLive}</span>
              <span className="cpd-kpi-label">Workers running</span>
            </div>
            <div className="cpd-kpi">
              <span className="cpd-kpi-value">{data.longest_running_stitches.length}</span>
              <span className="cpd-kpi-label">Active stitches</span>
            </div>
            <div className="cpd-kpi">
              <span className="cpd-kpi-value">{data.spend_by_project.length}</span>
              <span className="cpd-kpi-label">Active projects</span>
            </div>
          </section>

          <div className="cpd-grid">
            {/* Spend by project */}
            <section className="cpd-section cpd-section-wide">
              <h3>Spend by project · {data.range_label}</h3>
              {data.spend_by_project.length === 0 ? (
                <p className="cpd-empty">No spend data for this period</p>
              ) : (
                <div className="cpd-list">
                  {data.spend_by_project.map(p => (
                    <button
                      key={p.project}
                      className="cpd-list-row cpd-clickable"
                      onClick={() => onNavigateProject(p.project)}
                      title={`View ${p.project} detail`}
                    >
                      <div className="cpd-list-label">
                        {projectColor(p.project) && (
                          <span
                            className="cpd-color-dot"
                            style={{ background: projectColor(p.project) }}
                          />
                        )}
                        <span className="cpd-list-name">{p.project}</span>
                        <span className="cpd-list-link-hint">→</span>
                      </div>
                      <div className="cpd-list-right">
                        <Bar value={p.cost_usd} max={maxProjectSpend} color={projectColor(p.project)} />
                        <span className="cpd-list-value">{formatCost(p.cost_usd)}</span>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </section>

            {/* Spend by adapter */}
            <section className="cpd-section">
              <h3>Spend by adapter · {data.range_label}</h3>
              {data.spend_by_adapter.length === 0 ? (
                <p className="cpd-empty">No spend data for this period</p>
              ) : (
                <div className="cpd-list">
                  {data.spend_by_adapter.map(a => (
                    <div key={a.adapter} className="cpd-list-row">
                      <div className="cpd-list-label">
                        <span className="cpd-adapter-badge">{a.adapter}</span>
                      </div>
                      <div className="cpd-list-right">
                        <Bar value={a.cost_usd} max={maxAdapterSpend} />
                        <span className="cpd-list-value">{formatCost(a.cost_usd)}</span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </section>

            {/* Workers running */}
            <section className="cpd-section">
              <h3>Workers running</h3>
              {workersByProject.length === 0 ? (
                <p className="cpd-empty">No active workers</p>
              ) : (
                <div className="cpd-list">
                  {workersByProject.map(c => (
                    <button
                      key={c.name}
                      className="cpd-list-row cpd-clickable"
                      onClick={() => onNavigateProject(c.name)}
                      title={`View ${c.name} detail`}
                    >
                      <div className="cpd-list-label">
                        {c.color && (
                          <span className="cpd-color-dot" style={{ background: c.color }} />
                        )}
                        <span className="cpd-list-name">{c.label || c.name}</span>
                        <span className="cpd-list-link-hint">→</span>
                      </div>
                      <div className="cpd-list-right">
                        <span className="cpd-worker-count">{c.worker_count}</span>
                        <span className="cpd-list-label-sm">workers</span>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </section>

            {/* Longest-running stitches */}
            <section className="cpd-section cpd-section-wide">
              <h3>Longest-running stitches</h3>
              {data.longest_running_stitches.length === 0 ? (
                <p className="cpd-empty">No active stitches</p>
              ) : (
                <div className="cpd-stitches-list">
                  {data.longest_running_stitches.map(s => (
                    <button
                      key={s.bead_id}
                      className="cpd-stitch-row cpd-clickable"
                      onClick={() => onNavigateProject(s.project)}
                      title={`View ${s.project} detail`}
                    >
                      <div className="cpd-stitch-duration">
                        <span className="cpd-stitch-age">{formatDuration(s.duration_secs)}</span>
                      </div>
                      <div className="cpd-stitch-info">
                        <span className="cpd-stitch-title">{s.title}</span>
                        <div className="cpd-stitch-meta">
                          <span className="cpd-stitch-project">
                            {projectColor(s.project) && (
                              <span
                                className="cpd-color-dot cpd-color-dot-sm"
                                style={{ background: projectColor(s.project) }}
                              />
                            )}
                            {s.project}
                          </span>
                          <span className="cpd-stitch-id">{s.bead_id}</span>
                          <span className="cpd-list-link-hint">→</span>
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </section>
          </div>
        </div>
      )}
    </div>
  );
}
