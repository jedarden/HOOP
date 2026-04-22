import { useAtom } from 'jotai';
import { wsConnectedAtom } from './atoms';
import { useWebSocket } from './useWebSocket';
import FleetMap from './FleetMap';
import BeadList from './BeadList';
import ConversationPane from './ConversationPane';

export default function App() {
  const [wsConnected] = useAtom(wsConnectedAtom);

  // Initialize WebSocket connection
  useWebSocket();

  return (
    <div className="app">
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
        <FleetMap />
        <BeadList />
        <ConversationPane />
      </main>
    </div>
  );
}
