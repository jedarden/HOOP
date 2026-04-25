import { useAtom, useAtomValue } from 'jotai';
import { useState, useEffect, useCallback } from 'react';
import { wsConnectedAtom, configStatusAtom, projectCardsAtom, ProjectCardData } from './atoms';
import { useWebSocket } from './useWebSocket';
import OverviewPage from './OverviewPage';
import ProjectDetail from './ProjectDetail';
import FleetMap from './FleetMap';
import BeadList from './BeadList';
import ConversationPane from './ConversationPane';
import CapacityPanel from './CapacityPanel';
import AgentChatPane from './AgentChatPane';
import WorkerTimeline from './WorkerTimeline';

type Route =
  | { view: 'overview' }
  | { view: 'project'; name: string }
  | { view: 'fleet' }
  | { view: 'timeline' };

function ConfigBanner({ error }: { error: { message: string; line: number; col: number; field?: string; expected?: string; got?: string } }) {
  return (
    <div className="config-error-banner" role="alert">
      <div className="banner-content">
        <strong>Configuration Error</strong>
        <span className="banner-message">{error.message}</span>
        {error.field && <span className="banner-field">Field: {error.field}</span>}
        {error.expected && <span className="banner-expected">Expected: {error.expected}</span>}
        {error.got && <span className="banner-got">Got: {error.got}</span>}
        {error.line > 0 && <span className="banner-location">Line {error.line}, Column {error.col}</span>}
      </div>
    </div>
  );
}

function parseHash(hash: string): Route {
  const path = hash.replace(/^#\/?/, '');
  if (!path) return { view: 'overview' };
  if (path === 'fleet') return { view: 'fleet' };
  if (path === 'timeline') return { view: 'timeline' };
  return { view: 'project', name: path };
}

export default function App() {
  const [wsConnected] = useAtom(wsConnectedAtom);
  const [configStatus] = useAtom(configStatusAtom);
  const projectCards = useAtomValue(projectCardsAtom);
  const [route, setRoute] = useState<Route>(() => parseHash(window.location.hash));

  useWebSocket();

  // Hash-based routing
  useEffect(() => {
    const handleHashChange = () => {
      setRoute(parseHash(window.location.hash));
    };
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  // Notify daemon of active project
  useEffect(() => {
    const project = route.view === 'project' ? route.name : '';
    fetch('/api/ui/active-project', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ project }),
    }).catch(() => { /* best-effort */ });
  }, [route]);

  const navigateToProject = useCallback((card: ProjectCardData) => {
    window.location.hash = `#/${card.name}`;
  }, []);

  // Overview — home route
  if (route.view === 'overview') {
    return <OverviewPage onNavigateProject={navigateToProject} />;
  }

  // Timeline view — per-worker Gantt (hoop-ttb.2.16)
  if (route.view === 'timeline') {
    return (
      <div className="app app-project-detail">
        {configStatus.error && <ConfigBanner error={configStatus.error} />}
        <header className="app-header-mini">
          <div className="header-top">
            <div className="header-nav">
              <a href="#/" className="back-link">&larr; All Projects</a>
              <a href="#/fleet" className="back-link">Fleet</a>
            </div>
            <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
              <span className="indicator-dot" />
              {wsConnected ? 'Connected' : 'Connecting...'}
            </div>
          </div>
        </header>
        <main>
          <WorkerTimeline />
        </main>
      </div>
    );
  }

  // Fleet view — live worker layout (hoop-ttb.3.7)
  if (route.view === 'fleet') {
    return (
      <div className="app app-project-detail">
        {configStatus.error && <ConfigBanner error={configStatus.error} />}
        <header className="app-header-mini">
          <div className="header-top">
            <div className="header-nav">
              <a href="#/" className="back-link">&larr; All Projects</a>
              <a href="#/timeline" className="header-nav-link">Worker Timeline &rarr;</a>
            </div>
            <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
              <span className="indicator-dot" />
              {wsConnected ? 'Connected' : 'Connecting...'}
            </div>
          </div>
        </header>
        <main>
          <FleetMap />
          <BeadList />
          <ConversationPane />
          <AgentChatPane />
          <CapacityPanel projectName="" />
        </main>
      </div>
    );
  }

  // Project detail view
  const card = projectCards.find(p => p.name === route.name);
  if (!card) {
    return (
      <div className="app">
        <header className="app-header-mini">
          <div className="header-top">
            <a href="#/" className="back-link">&larr; All Projects</a>
            <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
              <span className="indicator-dot" />
              {wsConnected ? 'Connected' : 'Connecting...'}
            </div>
          </div>
        </header>
        <main>
          <div className="fleet-empty">Project "{route.name}" not found</div>
        </main>
      </div>
    );
  }

  return (
    <div className="app app-project-detail">
      {configStatus.error && <ConfigBanner error={configStatus.error} />}
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
        projectName={card.name}
        projectPath={card.path}
      />
    </div>
  );
}
