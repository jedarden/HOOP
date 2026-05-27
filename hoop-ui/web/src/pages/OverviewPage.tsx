import { useAtom, useAtomValue, useSetAtom } from 'jotai';
import { useMemo, useEffect, memo, useCallback } from 'react';
import {
  wsConnectedAtom,
  configStatusAtom,
  projectCardsAtom,
  projectsReceivedAtom,
  currentTimeAtom,
  stuckWorkersPanelOpenAtom,
  ProjectCardData,
} from './atoms';
import { SettingsMenu } from './components/SettingsMenu';
import { WhatsNewBanner } from './components/OnboardingPromptBanner';
import StuckWorkersPanel from './StuckWorkersPanel';

function formatRelativeTime(iso?: string, now?: number): string {
  if (!iso) return '--';
  const then = new Date(iso).getTime();
  const n = now ?? Date.now();
  const diffSec = Math.floor((n - then) / 1000);
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

// Isolated time display — subscribes to currentTimeAtom independently so
// the parent ProjectCard's memo is not defeated by the 30s tick.
const RelativeTime = memo(function RelativeTime({ iso }: { iso?: string }) {
  const now = useAtomValue(currentTimeAtom);
  return <>{formatRelativeTime(iso, now)}</>;
});

const ProjectCard = memo(function ProjectCard({ card, onClick, onStuckClick }: { card: ProjectCardData; onClick: () => void; onStuckClick: (e: React.MouseEvent) => void }) {
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
          <div
            className="pcf-stat pcf-stat-warn pcf-stat-clickable"
            onClick={(e) => {
              e.stopPropagation();
              onStuckClick(e);
            }}
            title={`View ${card.stuck_count} stuck worker${card.stuck_count > 1 ? 's' : ''}`}
          >
            <span className="pcf-stat-value">{card.stuck_count}</span>
            <span className="pcf-stat-label">stuck</span>
          </div>
        )}
      </div>

      <div className="pcf-footer">
        <span className="pcf-beads">{card.bead_count} beads</span>
        <span className="pcf-activity"><RelativeTime iso={card.last_activity} /></span>
      </div>

      <div
        className={`pcf-runtime-bar ${runtimeState}`}
        style={runtimeState === 'healthy' && card.color ? { background: card.color } : undefined}
      />
    </button>
  );
});

const TOUR_PROJECT_NAME = '__hoop_tour__';

export default function OverviewPage({ onNavigateProject }: { onNavigateProject: (card: ProjectCardData) => void }) {
  const wsConnected = useAtomValue(wsConnectedAtom);
  const configStatus = useAtomValue(configStatusAtom);
  const projectCards = useAtomValue(projectCardsAtom);
  const projectsReceived = useAtomValue(projectsReceivedAtom);
  const setCurrentTime = useSetAtom(currentTimeAtom);
  const [stuckPanelOpen, setStuckPanelOpen] = useAtom(stuckWorkersPanelOpenAtom);

  // Tick every 30s to refresh relative time displays
  useEffect(() => {
    const id = setInterval(() => setCurrentTime(Date.now()), 30_000);
    return () => clearInterval(id);
  }, [setCurrentTime]);

  // Filter out the tour project for fleet summary (it's a demo, not a real project)
  const realProjectCards = useMemo(() =>
    projectCards.filter(c => c.name !== TOUR_PROJECT_NAME),
    [projectCards]
  );

  const fleetSummary = useMemo(() => {
    const totalWorkers = realProjectCards.reduce((s, c) => s + c.worker_count, 0);
    const totalStitches = realProjectCards.reduce((s, c) => s + c.active_stitch_count, 0);
    const totalCost = realProjectCards.reduce((s, c) => s + c.cost_today, 0);
    const totalStuck = realProjectCards.reduce((s, c) => s + c.stuck_count, 0);
    const degradedCount = realProjectCards.filter(c => c.degraded).length;
    return { totalWorkers, totalStitches, totalCost, totalStuck, degradedCount };
  }, [realProjectCards]);

  // Degraded/error projects sorted to top for visibility
  const sortedCards = useMemo(() => {
    const healthy = projectCards.filter(c => !c.degraded && c.runtime_state !== 'failed' && c.runtime_state !== 'error');
    const degraded = projectCards.filter(c => c.degraded || c.runtime_state === 'failed' || c.runtime_state === 'error');
    return [...degraded, ...healthy];
  }, [projectCards]);

  const handleStuckClick = useCallback((card: ProjectCardData) => (e: React.MouseEvent) => {
    e.stopPropagation();
    // Toggle the panel: if already open for this project, close it; otherwise open it
    setStuckPanelOpen(prev => prev === card.name ? null : card.name);
  }, [setStuckPanelOpen]);

  const closeStuckPanel = useCallback(() => {
    setStuckPanelOpen(null);
  }, [setStuckPanelOpen]);

  // Get the project name for the stuck panel
  const stuckProjectName = stuckPanelOpen ?? undefined;

  return (
    <div className="app">
      <WhatsNewBanner />
      {configStatus.error && (
        <div className="config-error-banner" role="alert">
          <div className="banner-content">
            <strong>Configuration Error</strong>
            <span className="banner-message">{configStatus.error.message}</span>
            {configStatus.error.field && <span className="banner-field">Field: {configStatus.error.field}</span>}
            {configStatus.error.expected && <span className="banner-expected">Expected: {configStatus.error.expected}</span>}
            {configStatus.error.got && <span className="banner-got">Got: {configStatus.error.got}</span>}
            {configStatus.error.line > 0 && <span className="banner-location">Line {configStatus.error.line}, Column {configStatus.error.col}</span>}
          </div>
        </div>
      )}
      <header>
        <div className="header-top">
          <h1>HOOP</h1>
          <div className="header-right">
            <SettingsMenu />
            <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
              <span className="indicator-dot" />
              {wsConnected ? 'Connected' : 'Connecting...'}
            </div>
          </div>
        </div>
        <p>The operator's pane of glass and conversational handle.</p>
      </header>

      <main>
        {/* Cross-project summary strip */}
        <section className="fleet-summary-strip">
          <div className="fss-item">
            <span className="fss-value">{realProjectCards.length}</span>
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
            <button
              className="fss-item fss-warn fss-clickable"
              onClick={() => setStuckPanelOpen('')}
              title={`View ${fleetSummary.totalStuck} stuck worker${fleetSummary.totalStuck > 1 ? 's' : ''} across all projects`}
            >
              <span className="fss-value">{fleetSummary.totalStuck}</span>
              <span className="fss-label">stuck</span>
            </button>
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
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              <a href="#/search" className="section-header-link">Search &rarr;</a>
              <a href="#/dashboard" className="section-header-link">Dashboard &rarr;</a>
              <a href="#/fleet" className="section-header-link">Live map &rarr;</a>
              <a href="#/timeline" className="section-header-link">Timeline &rarr;</a>
              <a href="#/diagnostics" className="section-header-link">Diagnostics &rarr;</a>
            </div>
          </div>
          {!projectsReceived ? (
            <div className="fleet-loading">
              <div className="fleet-loading-spinner" />
              <span>Loading projects…</span>
            </div>
          ) : projectCards.length === 0 ? (
            <div className="fleet-empty">No projects registered</div>
          ) : (
            <div className="fleet-cards-grid">
              {sortedCards.map(card => (
                <ProjectCard
                  key={card.name}
                  card={card}
                  onClick={() => onNavigateProject(card)}
                  onStuckClick={handleStuckClick(card)}
                />
              ))}
            </div>
          )}
        </section>
      </main>

      {/* Stuck workers panel (right sidebar) */}
      {stuckPanelOpen !== null && (
        <aside className="right-panel stuck-workers-panel-wrapper">
          <div className="right-panel-header">
            <h2>
              {stuckProjectName ? `${stuckProjectName} Stuck Workers` : 'Fleet Stuck Workers'}
            </h2>
            <button
              className="right-panel-close"
              onClick={closeStuckPanel}
              aria-label="Close stuck workers panel"
            >
              ×
            </button>
          </div>
          <StuckWorkersPanel projectName={stuckProjectName ?? ''} projectPath="" />
        </aside>
      )}
    </div>
  );
}
