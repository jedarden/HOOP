import { useAtom, useAtomValue, useSetAtom } from 'jotai';
import { useState, useEffect, useCallback, useRef } from 'react';
import { wsConnectedAtom, connectionStatusAtom, configStatusAtom, projectCardsAtom, searchPaletteOpenAtom, activeProjectNameAtom, ProjectCardData } from './atoms';
import { useWebSocket } from './useWebSocket';
import OverviewPage from './OverviewPage';
import ProjectDetail from './ProjectDetail';
import FleetMap from './FleetMap';
import BeadList from './BeadList';
import ConversationPane from './ConversationPane';
import CapacityPanel from './CapacityPanel';
import AgentChatPane from './AgentChatPane';
import WorkerTimeline from './WorkerTimeline';
import AuditPanel from './AuditPanel';
import { SearchPalette } from './SearchPalette';
import CrossProjectDashboard from './CrossProjectDashboard';
import PatternsView from './PatternsView';
import ConversationsView from './ConversationsView';
import { DictationWidget } from './components/DictationWidget';
import { ConnectionBanner } from './components/ConnectionBanner';
import { StuckAlertBanner } from './components/StuckAlertBanner';
import { CollisionAlertBanner } from './components/CollisionAlertBanner';
import DraftsTab from './DraftsTab';
import UnknownEventsDiagnostics from './UnknownEventsDiagnostics';

type Route =
  | { view: 'overview' }
  | { view: 'project'; name: string }
  | { view: 'fleet' }
  | { view: 'timeline' }
  | { view: 'audit' }
  | { view: 'dashboard' }
  | { view: 'patterns'; patternId?: string }
  | { view: 'conversations' }
  | { view: 'drafts' }
  | { view: 'diagnostics' };

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
  const withoutPrefix = hash.replace(/^#\/?/, '');
  // Strip any ?filter=... query params embedded in the hash (used by FilesTab).
  const [path] = withoutPrefix.split('?', 2);
  if (!path) return { view: 'overview' };
  if (path === 'fleet') return { view: 'fleet' };
  if (path === 'timeline') return { view: 'timeline' };
  if (path === 'audit') return { view: 'audit' };
  if (path === 'dashboard') return { view: 'dashboard' };
  if (path === 'patterns') return { view: 'patterns' };
  if (path === 'conversations') return { view: 'conversations' };
  if (path === 'drafts') return { view: 'drafts' };
  if (path === 'diagnostics') return { view: 'diagnostics' };
  if (path.startsWith('patterns/')) {
    const patternId = path.slice('patterns/'.length);
    if (patternId) return { view: 'patterns', patternId };
  }
  return { view: 'project', name: path };
}

export default function App() {
  const [wsConnected] = useAtom(wsConnectedAtom);
  const [connectionStatus] = useAtom(connectionStatusAtom);
  const [configStatus] = useAtom(configStatusAtom);
  const projectCards = useAtomValue(projectCardsAtom);
  const [route, setRoute] = useState<Route>(() => parseHash(window.location.hash));
  const setSearchOpen = useSetAtom(searchPaletteOpenAtom);
  const setActiveProject = useSetAtom(activeProjectNameAtom);
  const [showRestoreToast, setShowRestoreToast] = useState(false);
  const prevConnectionStatusRef = useRef(connectionStatus);

  useWebSocket();

  // Show "restoring state" toast when transitioning from disconnected to connected
  useEffect(() => {
    if (prevConnectionStatusRef.current === 'disconnected' && connectionStatus === 'connected') {
      setShowRestoreToast(true);
      setTimeout(() => setShowRestoreToast(false), 3000);
    }
    prevConnectionStatusRef.current = connectionStatus;
  }, [connectionStatus]);

  // cmd-K (or ctrl-K) opens the search palette
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setSearchOpen(open => !open);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [setSearchOpen]);

  // Hash-based routing
  useEffect(() => {
    const handleHashChange = () => {
      setRoute(parseHash(window.location.hash));
    };
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  // Notify daemon of active project and update dictation context
  useEffect(() => {
    const project = route.view === 'project' ? route.name : '';
    setActiveProject(project);
    fetch('/api/ui/active-project', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ project }),
    }).catch(() => { /* best-effort */ });
  }, [route, setActiveProject]);

  const navigateToProject = useCallback((card: ProjectCardData) => {
    window.location.hash = `#/${card.name}`;
  }, []);

  const navigateToProjectByName = useCallback((name: string) => {
    window.location.hash = `#/${name}`;
  }, []);

  // Patterns view — list and detail
  if (route.view === 'patterns') {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/dashboard" className="header-nav-link">Dashboard</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/audit" className="header-nav-link">Audit</a>
              </div>
              <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
                <span className="indicator-dot" />
                {wsConnected ? 'Connected' : 'Connecting...'}
              </div>
            </div>
          </header>
          <main>
            <PatternsView
              patternId={route.patternId}
              projectCards={projectCards}
            />
          </main>
        </div>
        <SearchPalette />
        <DictationWidget />
        <StuckAlertBanner />
        <CollisionAlertBanner />
      </>
    );
  }

  // Cross-project dashboard view
  if (route.view === 'dashboard') {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/patterns" className="header-nav-link">Patterns</a>
                <a href="#/conversations" className="header-nav-link">Conversations</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/timeline" className="header-nav-link">Timeline</a>
                <a href="#/audit" className="header-nav-link">Audit</a>
              </div>
              <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
                <span className="indicator-dot" />
                {wsConnected ? 'Connected' : 'Connecting...'}
              </div>
            </div>
          </header>
          <main>
            <CrossProjectDashboard
              projectCards={projectCards}
              onNavigateProject={navigateToProjectByName}
            />
          </main>
        </div>
        <SearchPalette />
        <DictationWidget />
        <StuckAlertBanner />
        <CollisionAlertBanner />
      </>
    );
  }

  // Cross-project conversations view
  if (route.view === 'conversations') {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/patterns" className="header-nav-link">Patterns</a>
                <a href="#/dashboard" className="header-nav-link">Dashboard</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/timeline" className="header-nav-link">Timeline</a>
                <a href="#/audit" className="header-nav-link">Audit</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
              </div>
              <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
                <span className="indicator-dot" />
                {wsConnected ? 'Connected' : 'Connecting...'}
              </div>
            </div>
          </header>
          <main>
            <ConversationsView />
          </main>
        </div>
        <SearchPalette />
        <DictationWidget />
      </>
    );
  }

  // Drafts view — agent-created stitch preview queue (hoop-ttb.6.6)
  if (route.view === 'drafts') {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/dashboard" className="header-nav-link">Dashboard</a>
                <a href="#/patterns" className="header-nav-link">Patterns</a>
                <a href="#/conversations" className="header-nav-link">Conversations</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/timeline" className="header-nav-link">Timeline</a>
                <a href="#/audit" className="header-nav-link">Audit</a>
              </div>
              <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
                <span className="indicator-dot" />
                {wsConnected ? 'Connected' : 'Connecting...'}
              </div>
            </div>
          </header>
          <main>
            <DraftsTab />
          </main>
        </div>
        <SearchPalette />
        <DictationWidget />
        <StuckAlertBanner />
        <CollisionAlertBanner />
      </>
    );
  }

  // Diagnostics view — unknown events, never-silent-drop invariant
  if (route.view === 'diagnostics') {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/dashboard" className="header-nav-link">Dashboard</a>
                <a href="#/patterns" className="header-nav-link">Patterns</a>
                <a href="#/conversations" className="header-nav-link">Conversations</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/timeline" className="header-nav-link">Timeline</a>
                <a href="#/audit" className="header-nav-link">Audit</a>
              </div>
              <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
                <span className="indicator-dot" />
                {wsConnected ? 'Connected' : 'Connecting...'}
              </div>
            </div>
          </header>
          <main>
            <UnknownEventsDiagnostics />
          </main>
        </div>
        <SearchPalette />
        <DictationWidget />
        <StuckAlertBanner />
        <CollisionAlertBanner />
      </>
    );
  }

  // Overview — home route
  if (route.view === 'overview') {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <OverviewPage onNavigateProject={navigateToProject} />
        <SearchPalette />
        <DictationWidget />
      </>
    );
  }

  // Timeline view — per-worker Gantt (hoop-ttb.2.16)
  if (route.view === 'timeline') {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/audit" className="header-nav-link">Audit Log &rarr;</a>
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
        <SearchPalette />
        <DictationWidget />
      </>
    );
  }

  // Audit log view (hoop-ttb.2.18)
  if (route.view === 'audit') {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
              </div>
              <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
                <span className="indicator-dot" />
                {wsConnected ? 'Connected' : 'Connecting...'}
              </div>
            </div>
          </header>
          <main>
            <AuditPanel />
          </main>
        </div>
        <SearchPalette />
        <DictationWidget />
      </>
    );
  }

  // Fleet view — live worker layout (hoop-ttb.3.7)
  if (route.view === 'fleet') {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/timeline" className="header-nav-link">Worker Timeline &rarr;</a>
                <a href="#/audit" className="header-nav-link">Audit Log &rarr;</a>
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
        <SearchPalette />
        <DictationWidget />
      </>
    );
  }

  // Project detail view
  const card = projectCards.find(p => p.name === route.name);
  if (!card) {
    return (
      <>
        <ConnectionBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
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
        <SearchPalette />
        <DictationWidget />
      </>
    );
  }

  return (
    <>
      <ConnectionBanner />
      {showRestoreToast && (
        <div className="restore-toast" role="status">
          Restoring state...
        </div>
      )}
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
      <SearchPalette />
      <DictationWidget />
    </>
  );
}
