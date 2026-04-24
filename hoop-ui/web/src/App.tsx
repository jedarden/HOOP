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

type Route =
  | { view: 'overview' }
  | { view: 'project'; name: string }
  | { view: 'fleet' };

function parseHash(hash: string): Route {
  const path = hash.replace(/^#\/?/, '');
  if (!path) return { view: 'overview' };
  if (path === 'fleet') return { view: 'fleet' };
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

  // Fleet view — live worker layout (hoop-ttb.3.7)
  if (route.view === 'fleet') {
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
            <a href="#/" className="back-link">&larr; All Projects</a>
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
        projectName={card.name}
        projectPath={card.path}
      />
    </div>
  );
}
