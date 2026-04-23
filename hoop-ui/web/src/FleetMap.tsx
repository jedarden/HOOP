import { useAtomValue } from 'jotai';
import { workersAtom, WorkerData, WorkerDisplayState, WorkerLiveness } from './atoms';

function formatTimeAgo(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  return `${Math.floor(seconds / 3600)}h`;
}

function getStateLabel(state: WorkerDisplayState): string {
  switch (state.state) {
    case 'executing':
      if (!state.bead || state.bead === '0' || state.bead === '') {
        return 'Working';
      }
      return state.bead;
    case 'idle':
      return 'Idle';
    case 'knot':
      return `Knot: ${state.reason}`;
  }
}

function getStateDetails(state: WorkerDisplayState): string {
  switch (state.state) {
    case 'executing':
      return `${state.adapter}${state.model ? ` · ${state.model}` : ''}`;
    case 'idle':
      return state.last_strand ? `Last: ${state.last_strand}` : 'Waiting';
    case 'knot':
      return state.reason;
  }
}

function getWorkerStateColor(state: WorkerDisplayState, liveness: WorkerLiveness): string {
  if (liveness === 'Dead') return 'worker-dead';
  if (liveness === 'Hung') return 'worker-hung';
  switch (state.state) {
    case 'executing':
      return 'worker-executing';
    case 'idle':
      return 'worker-idle';
    case 'knot':
      return 'worker-knot';
  }
}

function getWorkerStateLabel(state: WorkerDisplayState, liveness: WorkerLiveness): string {
  if (liveness === 'Dead') return 'dead';
  if (liveness === 'Hung') return 'hung';
  switch (state.state) {
    case 'executing':
      return 'executing';
    case 'idle':
      return 'idle';
    case 'knot':
      return 'knot';
  }
}

function WorkerCard({ worker }: { worker: WorkerData }) {
  const stateColor = getWorkerStateColor(worker.state, worker.liveness);
  const stateLabel = getWorkerStateLabel(worker.state, worker.liveness);

  return (
    <div className={`worker-card ${stateColor}`}>
      <div className="worker-header">
        <h3 className="worker-name">{worker.worker}</h3>
        <span className="worker-status">{stateLabel}</span>
      </div>
      <div className="worker-body">
        <div className="worker-row">
          <span className="worker-label">Activity:</span>
          <span className="worker-value">{getStateLabel(worker.state)}</span>
        </div>
        <div className="worker-row">
          <span className="worker-label">Details:</span>
          <span className="worker-value">{getStateDetails(worker.state)}</span>
        </div>
        <div className="worker-row">
          <span className="worker-label">Last heartbeat:</span>
          <span className="worker-value">{formatTimeAgo(worker.heartbeat_age_secs)} ago</span>
        </div>
      </div>
      <a
        href={`#transcript/${worker.worker}`}
        className="worker-link"
        onClick={(e) => {
          e.preventDefault();
          window.location.hash = `transcript/${worker.worker}`;
          console.log('Navigate to transcript:', worker.worker);
        }}
      >
        View transcript →
      </a>
    </div>
  );
}

interface FleetMapProps {
  workers?: WorkerData[];
}

export default function FleetMap({ workers: workersProp }: FleetMapProps) {
  const globalWorkers = useAtomValue(workersAtom);
  const workers = workersProp ?? globalWorkers;

  if (workers.length === 0) {
    return (
      <section className="fleet-section">
        <h2>Fleet Map</h2>
        <div className="fleet-empty">
          <p>No workers detected. Waiting for heartbeats...</p>
        </div>
      </section>
    );
  }

  return (
    <section className="fleet-section">
      <h2>Fleet Map ({workers.length} workers)</h2>
      <div className="worker-grid">
        {workers.map((worker) => (
          <WorkerCard key={worker.worker} worker={worker} />
        ))}
      </div>
    </section>
  );
}
