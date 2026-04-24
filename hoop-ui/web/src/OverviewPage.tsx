import { useAtomValue } from 'jotai';
import { useMemo, useState, useEffect, memo } from 'react';
import {
  wsConnectedAtom,
  configStatusAtom,
  projectCardsAtom,
  ProjectCardData,
} from './atoms';

function formatRelativeTime(iso?: string, _now?: number): string {
  if (!iso) return '--';
  const then = new Date(iso).getTime();
  const now = _now ?? Date.now();
  const diffSec = Math.floor((now - then) / 1000);
  if (diffSec < 60) return `${diffSec}s ago`;
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
  return `${Math.floor(diffSec / 86400)}d ago`;
}

function formatCost(usd: number): string {
  if (usd === 0) return '$0';
  if (usd < 0.01) return '<$0.01';
  return `$${usd.toFixed(2)}`;
}

const ProjectCard = memo(function ProjectCard({ card, now, onClick }: { card: ProjectCardData; now: number; onClick: () => void }) {
  const runtimeState = card.runtime_state ?? 'unknown';
  const isDegraded = card.degraded;
  const hasError = isDegraded || runtimeState === 'failed' || runtimeState === 'error';

  return (
    <button
      className={`project-card-fleet ${hasError ? 'project-card-degraded' : ''}`}
      onClick={onClick}
      aria-label={`${card.label || card.name} — ${card.worker_count} workers, ${card.active_stitch_count} stitches, ${formatCost(card.cost_today)} today`}
      style={card.color ? { '--project-accent': card.color } as React.CSSProperties : undefined}
    >
      <div className="pcf-header">
        <div className="pcf-title-row">
          {card.color && <span className="pcf-color-dot" style={{ background: card.color }} />}
          <span className="pcf-label">{card.label || card.name}</span>
          {hasError && (
            <span className="pcf-error-badge" title={card.runtime_error || `Runtime ${runtimeState}`}>
              !
            </span>
          )}
        </div>
        <span className="pcf-arrow">&rarr;</span>
      </div>

      {hasError && card.runtime_error && (
        <div className="pcf-error-message" role="alert">{card.runtime_error}</div>
      )}
      {hasError && !card.runtime_error && (
        <div className="pcf-error-message" role="alert">Runtime {runtimeState}</div>
      )}

      <div className="pcf-stats">
        <div className="pcf-stat">
          <span className="pcf-stat-value">{card.worker_count}</span>
          <span className="pcf-stat-label">workers</span>
        </div>
        <div className="pcf-stat">
          <span className="pcf-stat-value">{card.active_stitch_count}</span>
          <span className="pcf-stat-label">stitches</span>
        </div>
        <div className="pcf-stat">
          <span className="pcf-stat-value">{formatCost(card.cost_today)}</span>
          <span className="pcf-stat-label">today</span>
        </div>
        {card.stuck_count > 0 && (
          <div className="pcf-stat pcf-stat-warn">
            <span className="pcf-stat-value">{card.stuck_count}</span>
            <span className="pcf-stat-label">stuck</span>
          </div>
        )}
      </div>

      <div className="pcf-footer">
        <span className="pcf-beads">{card.bead_count} beads</span>
        <span className="pcf-activity">{formatRelativeTime(card.last_activity, now)}</span>
      </div>

      <div
        className={`pcf-runtime-bar ${runtimeState}`}
        style={runtimeState === 'healthy' && card.color ? { background: card.color } : undefined}
      />
    </button>
  );
});

export default function OverviewPage({ onNavigateProject }: { onNavigateProject: (card: ProjectCardData) => void }) {
  const wsConnected = useAtomValue(wsConnectedAtom);
  const configStatus = useAtomValue(configStatusAtom);
  const projectCards = useAtomValue(projectCardsAtom);

  // Tick every 30s to refresh relative time displays
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 30_000);
    return () => clearInterval(id);
  }, []);

  const fleetSummary = useMemo(() => {
    const totalWorkers = projectCards.reduce((s, c) => s + c.worker_count, 0);
    const totalStitches = projectCards.reduce((s, c) => s + c.active_stitch_count, 0);
    const totalCost = projectCards.reduce((s, c) => s + c.cost_today, 0);
    const totalStuck = projectCards.reduce((s, c) => s + c.stuck_count, 0);
    const degradedCount = projectCards.filter(c => c.degraded).length;
    return { totalWorkers, totalStitches, totalCost, totalStuck, degradedCount };
  }, [projectCards]);

  // Degraded/error projects sorted to top for visibility
  const sortedCards = useMemo(() => {
    const healthy = projectCards.filter(c => !c.degraded && c.runtime_state !== 'failed' && c.runtime_state !== 'error');
    const degraded = projectCards.filter(c => c.degraded || c.runtime_state === 'failed' || c.runtime_state === 'error');
    return [...degraded, ...healthy];
  }, [projectCards]);

  return (
    <div className="app">
      {configStatus.error && (
        <div className="config-error-banner">
          <div className="banner-content">
            <strong>Configuration Error</strong>
            <span className="banner-message">{configStatus.error.message}</span>
            <span className="banner-location">Line {configStatus.error.line}, Column {configStatus.error.col}</span>
          </div>
        </div>
      )}
      <header>
        <div className="header-top">
          <h1>HOOP</h1>
          <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
            <span className="indicator-dot" />
            {wsConnected ? 'Connected' : 'Connecting...'}
          </div>
        </div>
        <p>The operator's pane of glass and conversational handle.</p>
      </header>

      <main>
        {/* Cross-project summary strip */}
        <section className="fleet-summary-strip">
          <div className="fss-item">
            <span className="fss-value">{projectCards.length}</span>
            <span className="fss-label">projects</span>
          </div>
          <div className="fss-item">
            <span className="fss-value">{fleetSummary.totalWorkers}</span>
            <span className="fss-label">workers</span>
          </div>
          <div className="fss-item">
            <span className="fss-value">{fleetSummary.totalStitches}</span>
            <span className="fss-label">active stitches</span>
          </div>
          <div className="fss-item">
            <span className="fss-value">{formatCost(fleetSummary.totalCost)}</span>
            <span className="fss-label">spend today</span>
          </div>
          {fleetSummary.totalStuck > 0 && (
            <div className="fss-item fss-warn">
              <span className="fss-value">{fleetSummary.totalStuck}</span>
              <span className="fss-label">stuck</span>
            </div>
          )}
          {fleetSummary.degradedCount > 0 && (
            <div className="fss-item fss-error">
              <span className="fss-value">{fleetSummary.degradedCount}</span>
              <span className="fss-label">degraded</span>
            </div>
          )}
        </section>

        {/* Project cards grid */}
        <section className="projects-section">
          <div className="section-header-row">
            <h2>Fleet</h2>
            <a href="#/fleet" className="section-header-link">Live worker map &rarr;</a>
          </div>
          {projectCards.length === 0 ? (
            wsConnected ? (
              <div className="fleet-empty">No projects registered</div>
            ) : (
              <div className="fleet-loading">
                <div className="fleet-loading-spinner" />
                <span>Loading projects…</span>
              </div>
            )
          ) : (
            <div className="fleet-cards-grid">
              {sortedCards.map(card => (
                <ProjectCard
                  key={card.name}
                  card={card}
                  now={now}
                  onClick={() => onNavigateProject(card)}
                />
              ))}
            </div>
          )}
        </section>
      </main>
    </div>
  );
}
