import { useAtom, useSetAtom } from 'jotai';
import { useState, useEffect } from 'react';
import { wsConnectedAtom, projectsAtom, configStatusAtom, Project } from './atoms';
import { useWebSocket } from './useWebSocket';
import ProjectDetail from './ProjectDetail';
import FleetMap from './FleetMap';
import BeadList from './BeadList';
import ConversationPane from './ConversationPane';

// Demo project data - will be replaced with real project registry
const DEMO_PROJECTS: Project[] = [
  { name: 'HOOP', path: '/home/coding/HOOP', activeBeads: 3, workers: 2 },
  { name: 'NEEDLE', path: '/home/coding/NEEDLE', activeBeads: 5, workers: 3 },
  { name: 'FABRIC', path: '/home/coding/FABRIC', activeBeads: 0, workers: 0 },
];

export default function App() {
  const [wsConnected] = useAtom(wsConnectedAtom);
  const [projects] = useAtom(projectsAtom);
  const [configStatus] = useAtom(configStatusAtom);
  const setProjects = useSetAtom(projectsAtom);
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);

  // Initialize WebSocket connection
  useWebSocket();

  // Initialize projects atom with demo data
  useEffect(() => {
    if (projects.length === 0) {
      setProjects(DEMO_PROJECTS);
    }
  }, [projects, setProjects]);

  // Handle hash-based routing
  useEffect(() => {
    const handleHashChange = () => {
      const hash = window.location.hash.slice(1); // Remove #
      if (hash) {
        const project = DEMO_PROJECTS.find(p => p.name === hash);
        if (project) {
          setSelectedProject(project);
        }
      } else {
        setSelectedProject(null);
      }
    };

    window.addEventListener('hashchange', handleHashChange);
    handleHashChange(); // Initial check

    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

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
              ← All Projects
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

  // Overview page - show all projects
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
        {/* Projects Overview */}
        <section className="projects-section">
          <h2>Projects</h2>
          <div className="projects-grid">
            {DEMO_PROJECTS.map(project => (
              <a
                key={project.name}
                href={`#${project.name}`}
                className="project-card"
              >
                <div className="project-card-header">
                  <h3 className="project-name">{project.name}</h3>
                  <span className="project-arrow">→</span>
                </div>
                <div className="project-card-meta">
                  <span className="meta-item">
                    <span className="meta-label">Path:</span>
                    <span className="meta-value">{project.path}</span>
                  </span>
                  <span className="meta-item">
                    <span className="meta-label">Active Beads:</span>
                    <span className="meta-value">{project.activeBeads}</span>
                  </span>
                  <span className="meta-item">
                    <span className="meta-label">Workers:</span>
                    <span className="meta-value">{project.workers}</span>
                  </span>
                </div>
              </a>
            ))}
          </div>
        </section>

        <FleetMap />
        <BeadList />
        <ConversationPane />
      </main>
    </div>
  );
}
