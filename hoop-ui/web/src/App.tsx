import { useAtom, useAtomValue } from 'jotai';
import { useState, useEffect, useMemo } from 'react';
import { wsConnectedAtom, configStatusAtom, projectCardsAtom, ProjectCardData } from './atoms';
import { useWebSocket } from './useWebSocket';
import ProjectDetail from './ProjectDetail';
import FleetMap from './FleetMap';
import BeadList from './BeadList';
import ConversationPane from './ConversationPane';
import CapacityPanel from './CapacityPanel';

function formatRelativeTime(iso?: string): string {
  if (!iso) return '--';
  const then = new Date(iso).getTime();
  const now = Date.now();
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

function ProjectCard({ card, onClick }: { card: ProjectCardData; onClick: () => void }) {
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
        <span className="pcf-activity">{formatRelativeTime(card.last_activity)}</span>
      </div>

      <div className={`pcf-runtime-bar ${runtimeState}`} />
    </button>
  );
}

export default function App() {
  const [wsConnected] = useAtom(wsConnectedAtom);
  const [configStatus] = useAtom(configStatusAtom);
  const projectCards = useAtomValue(projectCardsAtom);
  const [selectedProject, setSelectedProject] = useState<ProjectCardData | null>(null);

  useWebSocket();

  // Handle hash-based routing
  useEffect(() => {
    const handleHashChange = () => {
      const hash = window.location.hash.slice(1);
      if (hash) {
        const project = projectCards.find(p => p.name === hash);
        if (project) {
          setSelectedProject(project);
        }
      } else {
        setSelectedProject(null);
      }
    };

    window.addEventListener('hashchange', handleHashChange);
    handleHashChange();

    return () => window.removeEventListener('hashchange', handleHashChange);
  }, [projectCards]);

  const navigateToProject = (card: ProjectCardData) => {
    window.location.hash = card.name;
  };

  const fleetSummary = useMemo(() => {
    const totalWorkers = projectCards.reduce((s, c) => s + c.worker_count, 0);
    const totalStitches = projectCards.reduce((s, c) => s + c.active_stitch_count, 0);
    const totalCost = projectCards.reduce((s, c) => s + c.cost_today, 0);
    const totalStuck = projectCards.reduce((s, c) => s + c.stuck_count, 0);
    const degradedCount = projectCards.filter(c => c.degraded).length;
    return { totalWorkers, totalStitches, totalCost, totalStuck, degradedCount };
  }, [projectCards]);

  // If a project is selected, show the project detail view
  if (selectedProject) {
    return (
      <div className="app app-project-detail">
        {configStatus.error && (
          <div className="config-error-banner">
            <div className="banner-content">
              <strong>Configuration Error</strong>
              <span className="banner-message">{configStatus.error.message}</span>
              <span className="banner-location">Line {configStatus.error.line}, Column {configStatus.error.col}</span>
            </div>
          </div>
        )}
        <header className="app-header-mini">
          <div className="header-top">
            <a href="#/" className="back-link" onClick={(e) => { e.preventDefault(); window.location.hash = ''; }}>
              &larr; All Projects
            </a>
            <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
              <span className="indicator-dot" />
              {wsConnected ? 'Connected' : 'Connecting...'}
            </div>
          </div>
        </header>
        <ProjectDetail
          projectName={selectedProject.name}
          projectPath={selectedProject.path}
        />
      </div>
    );
  }

  // Overview page - fleet-of-fleets dashboard
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
        {/* Fleet Summary Strip */}
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

        {/* Project Cards Grid */}
        <section className="projects-section">
          <h2>Fleet</h2>
          {projectCards.length === 0 ? (
            <div className="fleet-empty">No projects registered</div>
          ) : (
            <div className="fleet-cards-grid">
              {projectCards.map(card => (
                <ProjectCard
                  key={card.name}
                  card={card}
                  onClick={() => navigateToProject(card)}
                />
              ))}
            </div>
          )}
        </section>

        <CapacityPanel projectName="" />
        <FleetMap />
        <BeadList />
        <ConversationPane />
      </main>
    </div>
  );
}
