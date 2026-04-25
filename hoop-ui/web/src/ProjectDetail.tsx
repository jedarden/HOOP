import { useAtomValue } from 'jotai';
import { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import { beadsAtom, workersAtom, conversationsAtom, WorkerData } from './atoms';
import FleetMap from './FleetMap';
import ConversationPane from './ConversationPane';
import BeadGraph from './BeadGraph';
import CostPanel from './CostPanel';
import CapacityPanel from './CapacityPanel';
import StitchesTab from './StitchesTab';
import FilesTab from './FilesTab';
import DebugPanel from './DebugPanel';
import DiffViewer from './DiffViewer';
import StitchNetDiff from './StitchNetDiff';

export type TabId = 'stitches' | 'fleet' | 'graph' | 'conversations' | 'cost' | 'capacity' | 'files' | 'debug' | 'diff' | 'net-diff';

interface Tab {
  id: TabId;
  label: string;
  description: string;
  keyboardShortcut: string;
}

const TABS: Tab[] = [
  { id: 'stitches', label: 'Stitches', description: 'Conversations and work items', keyboardShortcut: '1' },
  { id: 'fleet', label: 'Fleet Map', description: 'Worker status and liveness', keyboardShortcut: '2' },
  { id: 'graph', label: 'Bead Graph', description: 'Dependency graph visualization', keyboardShortcut: '3' },
  { id: 'conversations', label: 'Conversations', description: 'Full conversation transcripts', keyboardShortcut: '4' },
  { id: 'cost', label: 'Cost', description: 'Usage and cost breakdown', keyboardShortcut: '5' },
  { id: 'capacity', label: 'Capacity', description: 'Rate limit and capacity status', keyboardShortcut: '6' },
  { id: 'debug', label: 'Debug', description: 'Per-bead execution step-through', keyboardShortcut: '7' },
  { id: 'files', label: 'Files', description: 'Project file browser', keyboardShortcut: '8' },
  { id: 'diff', label: 'Diff', description: 'Side-by-side git diff view', keyboardShortcut: '9' },
  { id: 'net-diff', label: 'Net-Diff', description: 'PR-like stitch net-diff review', keyboardShortcut: '0' },
];

export interface ProjectDetailProps {
  projectName: string;
  projectPath: string;
}

export default function ProjectDetail({ projectName, projectPath }: ProjectDetailProps) {
  const [activeTab, setActiveTab] = useState<TabId>('stitches');
  const tabsRef = useRef<HTMLDivElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  // Move focus to tab panel on keyboard-driven tab switch
  useEffect(() => {
    requestAnimationFrame(() => {
      if (panelRef.current) {
        panelRef.current.focus();
      }
    });
  }, [activeTab]);

  // Read all global data
  const allBeads = useAtomValue(beadsAtom);
  const allWorkers = useAtomValue(workersAtom);
  const allConversations = useAtomValue(conversationsAtom);

  // Scope conversations to this project by cwd prefix
  const projectConversations = useMemo(() =>
    allConversations.filter(c => c.cwd.startsWith(projectPath)),
    [allConversations, projectPath],
  );

  // Collect bead IDs referenced by project conversations (worker sessions)
  const projectBeadIds = useMemo(() => {
    const ids = new Set<string>();
    for (const c of projectConversations) {
      if (c.worker_metadata?.bead) ids.add(c.worker_metadata.bead);
    }
    return ids;
  }, [projectConversations]);

  // Beads: include all beads (bead project scoping is done server-side;
  // when the backend sends per-project bead snapshots, this will narrow automatically).
  // Also include beads referenced by project conversations.
  const projectBeads = useMemo(() => {
    const result = new Map<string, typeof allBeads[0]>();
    for (const b of allBeads) result.set(b.id, b);
    return Array.from(result.values());
  }, [allBeads]);

  // Workers: those executing beads linked to this project, plus idle/knot workers
  const projectWorkers = useMemo((): WorkerData[] => {
    const executingInProject = new Set<string>();
    for (const w of allWorkers) {
      if (w.state.state === 'executing' && projectBeadIds.has(w.state.bead)) {
        executingInProject.add(w.worker);
      }
    }
    return allWorkers.filter(w => {
      if (executingInProject.has(w.worker)) return true;
      // Idle/knot workers are shared across projects
      if (w.state.state !== 'executing') return true;
      return false;
    });
  }, [allWorkers, projectBeadIds]);

  // Keyboard navigation
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      return;
    }

    // Alt+number to switch tabs (1–9 for tabs 1–9, 0 for tab 10)
    if (event.altKey && (event.key >= '1' && event.key <= '9' || event.key === '0')) {
      event.preventDefault();
      const tabIndex = event.key === '0' ? 9 : parseInt(event.key) - 1;
      if (tabIndex < TABS.length) {
        setActiveTab(TABS[tabIndex].id);
      }
      return;
    }

    // Arrow keys: within tablist (no modifier) or Alt+arrow anywhere
    if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
      const inTablist = target.role === 'tab' || target.closest('[role="tablist"]');
      if (inTablist || event.altKey) {
        event.preventDefault();
        const currentIndex = TABS.findIndex(t => t.id === activeTab);
        const direction = event.key === 'ArrowRight' ? 1 : -1;
        const newIndex = (currentIndex + direction + TABS.length) % TABS.length;
        setActiveTab(TABS[newIndex].id);

        // Focus the new tab button for in-tablist navigation
        if (inTablist && tabsRef.current) {
          const btn = tabsRef.current.children[newIndex] as HTMLElement;
          btn?.focus();
        }
      }
    }

    // Home/End to jump to first/last tab when in tablist
    if ((event.key === 'Home' || event.key === 'End') && (target.role === 'tab' || target.closest('[role="tablist"]'))) {
      event.preventDefault();
      const newIndex = event.key === 'Home' ? 0 : TABS.length - 1;
      setActiveTab(TABS[newIndex].id);
      if (tabsRef.current) {
        const btn = tabsRef.current.children[newIndex] as HTMLElement;
        btn?.focus();
      }
    }
  }, [activeTab]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return (
    <div className="project-detail">
      <header className="project-detail-header">
        <div className="project-title-row">
          <div>
            <h1 className="project-name">{projectName}</h1>
            <p className="project-path">{projectPath}</p>
          </div>
          <div className="project-stats">
            <div className="stat">
              <span className="stat-value">{projectBeads.length}</span>
              <span className="stat-label">Beads</span>
            </div>
            <div className="stat">
              <span className="stat-value">{projectWorkers.length}</span>
              <span className="stat-label">Workers</span>
            </div>
            <div className="stat">
              <span className="stat-value">{projectConversations.length}</span>
              <span className="stat-label">Conversations</span>
            </div>
          </div>
        </div>

        <div
          ref={tabsRef}
          className="tab-list"
          role="tablist"
          aria-label="Project detail views"
        >
          {TABS.map((tab) => (
            <button
              key={tab.id}
              className={`tab-button ${activeTab === tab.id ? 'active' : ''}`}
              onClick={() => setActiveTab(tab.id)}
              role="tab"
              aria-selected={activeTab === tab.id}
              aria-controls={`panel-${tab.id}`}
              tabIndex={activeTab === tab.id ? 0 : -1}
              title={`${tab.label} - ${tab.description} (Alt+${tab.keyboardShortcut})`}
            >
              <span className="tab-label">{tab.label}</span>
              <span className="tab-shortcut">{tab.keyboardShortcut}</span>
            </button>
          ))}
        </div>
      </header>

      <main className="project-detail-main">
        <div
          ref={panelRef}
          id={`panel-${activeTab}`}
          className="tab-panel"
          role="tabpanel"
          aria-labelledby={`tab-${activeTab}`}
          tabIndex={0}
        >
          {activeTab === 'stitches' && (
            <StitchesTab
              projectName={projectName}
              projectPath={projectPath}
              conversations={projectConversations}
              onSwitchTab={setActiveTab}
            />
          )}
          {activeTab === 'fleet' && (
            <div className="panel-content">
              <FleetMap workers={projectWorkers} />
            </div>
          )}
          {activeTab === 'graph' && (
            <div className="panel-content">
              <BeadGraph beads={projectBeads} />
            </div>
          )}
          {activeTab === 'conversations' && (
            <div className="panel-content">
              <ConversationPane conversations={projectConversations} />
            </div>
          )}
          {activeTab === 'cost' && (
            <div className="panel-content">
              <CostPanel projectName={projectName} conversations={projectConversations} />
            </div>
          )}
          {activeTab === 'capacity' && (
            <div className="panel-content">
              <CapacityPanel projectName={projectName} />
            </div>
          )}
          {activeTab === 'files' && (
            <div className="panel-content">
              <FilesTab projectName={projectName} projectPath={projectPath} />
            </div>
          )}
          {activeTab === 'debug' && (
            <div className="panel-content">
              <DebugPanel projectName={projectName} projectPath={projectPath} />
            </div>
          )}
          {activeTab === 'diff' && (
            <div className="diff-viewer-wrap">
              <DiffViewer projectName={projectName} />
            </div>
          )}
          {activeTab === 'net-diff' && (
            <div className="diff-viewer-wrap">
              <StitchNetDiff
                projectName={projectName}
                projectPath={projectPath}
                conversations={projectConversations}
              />
            </div>
          )}
        </div>
      </main>

      <footer className="project-detail-footer">
        <span className="keyboard-hint">
          Press <kbd>Alt</kbd> + <kbd>1-9</kbd> to switch tabs, <kbd>Alt</kbd> + <kbd>←</kbd>/<kbd>→</kbd> to navigate
        </span>
      </footer>
    </div>
  );
}
