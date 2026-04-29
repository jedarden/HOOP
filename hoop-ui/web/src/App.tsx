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
import RedactionAuditPanel from './RedactionAuditPanel';
import { SearchPalette } from './SearchPalette';
import CrossProjectDashboard from './CrossProjectDashboard';
import PatternsView from './PatternsView';
import ConversationsView from './ConversationsView';
import SearchPage from './SearchPage';
import { DictationWidget } from './components/DictationWidget';
import { ConnectionBanner } from './components/ConnectionBanner';
import { StuckAlertBanner } from './components/StuckAlertBanner';
import { CollisionAlertBanner } from './components/CollisionAlertBanner';
import { WelcomeTour } from './components/WelcomeTour';
import { SettingsMenu } from './components/SettingsMenu';
import { WhatsNewBanner } from './components/OnboardingPromptBanner';
import DraftsTab from './DraftsTab';
import UnknownEventsDiagnostics from './UnknownEventsDiagnostics';
import UnassignedSessions from './UnassignedSessions';

type Route =
  | { view: 'overview' }
  | { view: 'project'; name: string }
  | { view: 'fleet' }
  | { view: 'timeline' }
  | { view: 'audit' }
  | { view: 'redaction-audit' }
  | { view: 'unassigned' }
  | { view: 'dashboard' }
  | { view: 'patterns'; patternId?: string }
  | { view: 'conversations' }
  | { view: 'drafts' }
  | { view: 'diagnostics' }
  | { view: 'search' };

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

function RestartRequiredBanner({ restart_required }: { restart_required: { keys: string[]; message: string } }) {
  return (
    <div className="config-restart-banner" role="alert">
      <div className="banner-content">
        <strong>⚠️ Restart Required</strong>
        <span className="banner-message">{restart_required.message}</span>
        <span className="banner-keys">Keys: {restart_required.keys.join(', ')}</span>
        <span className="banner-action">Run: systemctl --user restart hoop</span>
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
  if (path === 'redaction-audit') return { view: 'redaction-audit' };
  if (path === 'unassigned') return { view: 'unassigned' };
  if (path === 'dashboard') return { view: 'dashboard' };
  if (path === 'patterns') return { view: 'patterns' };
  if (path === 'conversations') return { view: 'conversations' };
  if (path === 'drafts') return { view: 'drafts' };
  if (path === 'diagnostics') return { view: 'diagnostics' };
  if (path === 'search') return { view: 'search' };
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

  // Handle welcome tour starter prompt actions
  useEffect(() => {
    const handleStartDictation = () => {
      // Navigate to the first available project, then trigger dictation
      const firstProject = projectCards[0];
      if (firstProject) {
        window.location.hash = `#/${firstProject.name}`;
        // Delay dictation trigger to allow navigation to complete
        setTimeout(() => {
          const hotkeyEvent = new KeyboardEvent('keydown', {
            key: 'd',
            metaKey: true,
            ctrlKey: false,
            shiftKey: true,
            altKey: false,
            bubbles: true,
          });
          window.dispatchEvent(hotkeyEvent);
        }, 500);
      } else {
        // No projects available, navigate to fleet view
        window.location.hash = '#/fleet';
      }
    };

    const handleRegisterProject = () => {
      // Show instructions for registering a project via CLI
      alert(
        'To register a new project:\n\n' +
        '1. Open your terminal\n' +
        '2. Navigate to your project directory\n' +
        '3. Run: hoop-cli project register\n\n' +
        'Or scan a directory for all projects:\n' +
        '  hoop-cli project scan <directory>\n\n' +
        'The project will appear in HOOP automatically.'
      );
    };

    const handleOpenAgentChat = () => {
      // Navigate to fleet view which includes AgentChatPane and show guidance
      window.location.hash = '#/fleet';
      // Show a toast message pointing to the chat interface
      setTimeout(() => {
        const toast = document.createElement('div');
        toast.className = 'restore-toast agent-chat-hint';
        toast.textContent = '💬 Agent chat is available in the right panel — start typing to ask questions!';
        toast.style.cssText = 'background: #3b82f6; color: white; padding: 12px 20px; border-radius: 8px; margin: 16px; box-shadow: 0 4px 12px rgba(0,0,0,0.15);';
        document.querySelector('.app-project-detail main')?.appendChild(toast);
        setTimeout(() => toast.remove(), 5000);
      }, 300);
    };

    const handleEnableTour = async () => {
      try {
        const response = await fetch('/api/tour/enable', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({}),
        });
        if (response.ok) {
          // Navigate to the tour project
          window.location.hash = '#/__hoop_tour__';
        } else {
          alert('Failed to enable tour project. Please try again.');
        }
      } catch (error) {
        console.error('Error enabling tour:', error);
        alert('Failed to enable tour project. Please try again.');
      }
    };

    window.addEventListener('hoop-start-dictation', handleStartDictation);
    window.addEventListener('hoop-register-project', handleRegisterProject);
    window.addEventListener('hoop-open-agent-chat', handleOpenAgentChat);
    window.addEventListener('hoop-enable-tour', handleEnableTour);

    return () => {
      window.removeEventListener('hoop-start-dictation', handleStartDictation);
      window.removeEventListener('hoop-register-project', handleRegisterProject);
      window.removeEventListener('hoop-open-agent-chat', handleOpenAgentChat);
      window.removeEventListener('hoop-enable-tour', handleEnableTour);
    };
  }, [projectCards]);

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
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/dashboard" className="header-nav-link">Dashboard</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/audit" className="header-nav-link">Audit</a>
                <a href="#/redaction-audit" className="header-nav-link">Redaction Audit</a>
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
        <SettingsMenu />
        <WelcomeTour />
      </>
    );
  }

  // Cross-project dashboard view
  if (route.view === 'dashboard') {
    return (
      <>
        <ConnectionBanner />
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
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
                <a href="#/redaction-audit" className="header-nav-link">Redaction Audit</a>
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
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/patterns" className="header-nav-link">Patterns</a>
                <a href="#/dashboard" className="header-nav-link">Dashboard</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/timeline" className="header-nav-link">Timeline</a>
                <a href="#/audit" className="header-nav-link">Audit</a>
                <a href="#/redaction-audit" className="header-nav-link">Redaction Audit</a>
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
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
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
                <a href="#/redaction-audit" className="header-nav-link">Redaction Audit</a>
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
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/dashboard" className="header-nav-link">Dashboard</a>
                <a href="#/patterns" className="header-nav-link">Patterns</a>
                <a href="#/conversations" className="header-nav-link">Conversations</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/unassigned" className="header-nav-link">Unassigned</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/timeline" className="header-nav-link">Timeline</a>
                <a href="#/audit" className="header-nav-link">Audit</a>
                <a href="#/redaction-audit" className="header-nav-link">Redaction Audit</a>
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

  // Search view — full-text search with faceted filters
  if (route.view === 'search') {
    return (
      <>
        <ConnectionBanner />
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/dashboard" className="header-nav-link">Dashboard</a>
                <a href="#/patterns" className="header-nav-link">Patterns</a>
                <a href="#/conversations" className="header-nav-link">Conversations</a>
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
            <SearchPage />
          </main>
        </div>
        <SearchPalette />
        <DictationWidget />
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
        <StuckAlertBanner />
        <CollisionAlertBanner />
        <SettingsMenu />
        <WelcomeTour />
      </>
    );
  }

  // Timeline view — per-worker Gantt (hoop-ttb.2.16)
  if (route.view === 'timeline') {
    return (
      <>
        <ConnectionBanner />
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/audit" className="header-nav-link">Audit Log &rarr;</a>
                <a href="#/redaction-audit" className="header-nav-link">Redaction Audit &rarr;</a>
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
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/redaction-audit" className="header-nav-link">Redaction Audit &rarr;</a>
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

  // Redaction audit log view (hoop-ttb.15.5)
  if (route.view === 'redaction-audit') {
    return (
      <>
        <ConnectionBanner />
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/fleet" className="header-nav-link">Fleet</a>
                <a href="#/audit" className="header-nav-link">General Audit &rarr;</a>
              </div>
              <div className={`connection-indicator ${wsConnected ? 'connected' : 'disconnected'}`}>
                <span className="indicator-dot" />
                {wsConnected ? 'Connected' : 'Connecting...'}
              </div>
            </div>
          </header>
          <main>
            <RedactionAuditPanel />
          </main>
        </div>
        <SearchPalette />
        <DictationWidget />
      </>
    );
  }

  // Unassigned sessions view (§5.4)
  if (route.view === 'unassigned') {
    return (
      <>
        <ConnectionBanner />
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/dashboard" className="header-nav-link">Dashboard</a>
                <a href="#/patterns" className="header-nav-link">Patterns</a>
                <a href="#/conversations" className="header-nav-link">Conversations</a>
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
            <UnassignedSessions />
          </main>
        </div>
        <SearchPalette />
        <DictationWidget />
        <StuckAlertBanner />
        <CollisionAlertBanner />
      </>
    );
  }

  // Fleet view — live worker layout (hoop-ttb.3.7)
  if (route.view === 'fleet') {
    return (
      <>
        <ConnectionBanner />
        <WhatsNewBanner />
        {showRestoreToast && (
          <div className="restore-toast" role="status">
            Restoring state...
          </div>
        )}
        <div className="app app-project-detail">
          {configStatus.error && <ConfigBanner error={configStatus.error} />}
          {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
          <header className="app-header-mini">
            <div className="header-top">
              <div className="header-nav">
                <a href="#/" className="back-link">&larr; All Projects</a>
                <a href="#/drafts" className="header-nav-link">Drafts</a>
                <a href="#/timeline" className="header-nav-link">Worker Timeline &rarr;</a>
                <a href="#/audit" className="header-nav-link">Audit Log &rarr;</a>
                <a href="#/redaction-audit" className="header-nav-link">Redaction Audit &rarr;</a>
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
      <WhatsNewBanner />
      {showRestoreToast && (
        <div className="restore-toast" role="status">
          Restoring state...
        </div>
      )}
      <div className="app app-project-detail">
        {configStatus.error && <ConfigBanner error={configStatus.error} />}
        {configStatus.restart_required && <RestartRequiredBanner restart_required={configStatus.restart_required} />}
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
