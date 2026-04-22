import { useAtomValue } from 'jotai';
import { useState, useCallback, useEffect, useRef } from 'react';
import { beadsAtom, workersAtom, conversationsAtom } from './atoms';
import FleetMap from './FleetMap';
import ConversationPane from './ConversationPane';
import BeadGraph from './BeadGraph';
import CostPanel from './CostPanel';
import CapacityPanel from './CapacityPanel';
import StitchesTab from './StitchesTab';
import FilesTab from './FilesTab';

type TabId = 'stitches' | 'fleet' | 'graph' | 'conversations' | 'cost' | 'capacity' | 'files';

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
  { id: 'files', label: 'Files', description: 'Project file browser', keyboardShortcut: '7' },
];

export interface ProjectDetailProps {
  projectName: string;
  projectPath: string;
}

export default function ProjectDetail({ projectName, projectPath }: ProjectDetailProps) {
  const [activeTab, setActiveTab] = useState<TabId>('stitches');
  const tabsRef = useRef<HTMLDivElement>(null);

  // Filter data for this project
  const allBeads = useAtomValue(beadsAtom);
  const allWorkers = useAtomValue(workersAtom);
  const allConversations = useAtomValue(conversationsAtom);

  // Filter beads by project (for now, all beads are shown - this will be updated with project filtering)
  const projectBeads = allBeads;

  // Keyboard navigation
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    // Check if user is typing in an input field
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      return;
    }

    // Alt+number to switch tabs
    if (event.altKey && event.key >= '1' && event.key <= '7') {
      event.preventDefault();
      const tabIndex = parseInt(event.key) - 1;
      if (tabIndex < TABS.length) {
        setActiveTab(TABS[tabIndex].id);
      }
    }

    // Arrow keys for tab navigation
    if (event.altKey) {
      const currentIndex = TABS.findIndex(t => t.id === activeTab);
      if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
        event.preventDefault();
        const direction = event.key === 'ArrowRight' ? 1 : -1;
        const newIndex = (currentIndex + direction + TABS.length) % TABS.length;
        setActiveTab(TABS[newIndex].id);
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
              <span className="stat-value">{allWorkers.length}</span>
              <span className="stat-label">Workers</span>
            </div>
            <div className="stat">
              <span className="stat-value">{allConversations.length}</span>
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
          id={`panel-${activeTab}`}
          className="tab-panel"
          role="tabpanel"
          aria-labelledby={`tab-${activeTab}`}
        >
          {activeTab === 'stitches' && <StitchesTab projectName={projectName} projectPath={projectPath} />}
          {activeTab === 'fleet' && (
            <div className="panel-content">
              <FleetMap />
            </div>
          )}
          {activeTab === 'graph' && (
            <div className="panel-content">
              <BeadGraph beads={projectBeads} />
            </div>
          )}
          {activeTab === 'conversations' && (
            <div className="panel-content">
              <ConversationPane />
            </div>
          )}
          {activeTab === 'cost' && (
            <div className="panel-content">
              <CostPanel projectName={projectName} />
            </div>
          )}
          {activeTab === 'capacity' && (
            <div className="panel-content">
              <CapacityPanel projectName={projectName} />
            </div>
          )}
          {activeTab === 'files' && (
            <div className="panel-content">
              <FilesTab projectPath={projectPath} />
            </div>
          )}
        </div>
      </main>

      <footer className="project-detail-footer">
        <span className="keyboard-hint">
          Press <kbd>Alt</kbd> + <kbd>1-7</kbd> to switch tabs, <kbd>Alt</kbd> + <kbd>←</kbd>/<kbd>→</kbd> to navigate
        </span>
      </footer>
    </div>
  );
}
