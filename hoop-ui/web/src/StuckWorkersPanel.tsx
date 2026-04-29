import { useAtomValue } from 'jotai';
import { useMemo } from 'react';
import { stuckAlertsAtom, workersAtom, beadsAtom, type StuckAlert } from './atoms';

interface StuckWorkersPanelProps {
  projectName: string;
  projectPath: string;
}

function formatReason(reason: StuckAlert['reason']): string {
  switch (reason) {
    case 'idle_timeout':
      return 'Idle timeout';
    case 'max_runtime_exceeded':
      return 'Max runtime exceeded';
    case 'content_seen_grace_exceeded':
      return 'Content grace exceeded';
    case 'heartbeat_transition_silence':
      return 'Heartbeat transition silence';
    case 'repeated_retry':
      return 'Repeated retry';
    default:
      return reason;
  }
}

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${mins}m`;
}

function formatHeartbeatDelta(iso: string | null): string {
  if (!iso) return 'Never';
  const then = new Date(iso).getTime();
  const now = Date.now();
  const diffSec = Math.floor((now - then) / 1000);
  return `${formatDuration(diffSec)} ago`;
}

function formatTime(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false });
}

interface StuckWorkerRowProps {
  alert: StuckAlert;
  beadTitle: string;
}

function StuckWorkerRow({ alert, beadTitle }: StuckWorkerRowProps) {
  return (
    <div className="stuck-worker-row">
      <div className="stuck-worker-header">
        <span className="stuck-worker-name">{alert.worker}</span>
        <span className={`stuck-reason-badge stuck-reason-${alert.reason}`}>
          {formatReason(alert.reason)}
        </span>
      </div>
      <div className="stuck-worker-details">
        <div className="stuck-detail-item">
          <span className="stuck-detail-label">Bead</span>
          <span className="stuck-detail-value">{alert.bead}</span>
        </div>
        {beadTitle && beadTitle !== alert.bead && (
          <div className="stuck-detail-item">
            <span className="stuck-detail-label">Title</span>
            <span className="stuck-detail-value">{beadTitle}</span>
          </div>
        )}
        <div className="stuck-detail-item">
          <span className="stuck-detail-label">Started</span>
          <span className="stuck-detail-value">{formatTime(alert.started_at)}</span>
        </div>
        <div className="stuck-detail-item">
          <span className="stuck-detail-label">Elapsed</span>
          <span className="stuck-detail-value">{formatDuration(alert.elapsed_secs)}</span>
        </div>
        <div className="stuck-detail-item">
          <span className="stuck-detail-label">Idle</span>
          <span className="stuck-detail-value">{formatDuration(alert.idle_secs)}</span>
        </div>
        <div className="stuck-detail-item">
          <span className="stuck-detail-label">Last heartbeat</span>
          <span className="stuck-detail-value">{formatHeartbeatDelta(alert.last_heartbeat_at)}</span>
        </div>
        {alert.last_transition_at && (
          <div className="stuck-detail-item">
            <span className="stuck-detail-label">Last transition</span>
            <span className="stuck-detail-value">{formatTime(alert.last_transition_at)}</span>
          </div>
        )}
        {alert.retry_count > 0 && (
          <div className="stuck-detail-item">
            <span className="stuck-detail-label">Retry count</span>
            <span className="stuck-detail-value">{alert.retry_count}</span>
          </div>
        )}
      </div>
      {alert.message && (
        <div className="stuck-worker-message" title={alert.message}>
          {alert.message}
        </div>
      )}
    </div>
  );
}

export default function StuckWorkersPanel({ projectName }: StuckWorkersPanelProps) {
  const stuckAlerts = useAtomValue(stuckAlertsAtom);
  const workers = useAtomValue(workersAtom);
  const beads = useAtomValue(beadsAtom);

  // Filter stuck alerts to this project (or all projects if projectName is empty for fleet view)
  const projectStuckAlerts = useMemo(() => {
    const alerts: Array<{ alert: StuckAlert; beadTitle: string }> = [];
    const isFleetView = !projectName;

    for (const [, alert] of stuckAlerts) {
      // Find the worker for this alert
      const worker = workers.find(w => w.worker === alert.worker);
      if (!worker) continue;

      // Check if worker belongs to this project (or all projects for fleet view)
      // Workers executing beads are scoped to the bead's project
      // Idle/knot workers are shared across all projects
      const isInProject = (() => {
        if (isFleetView) {
          // Fleet view: include all workers
          return true;
        }
        if (worker.state.state === 'executing') {
          const bead = beads.find(b => b.id === alert.bead);
          return bead?.project === projectName;
        }
        // Include idle/knot workers - they're relevant to all projects
        return true;
      })();

      if (isInProject) {
        const bead = beads.find(b => b.id === alert.bead);
        alerts.push({
          alert,
          beadTitle: bead?.title ?? alert.bead,
        });
      }
    }

    // Sort by elapsed time (most stuck first)
    return alerts.sort((a, b) => b.alert.elapsed_secs - a.alert.elapsed_secs);
  }, [stuckAlerts, workers, beads, projectName]);

  // Also count workers in Knot state (may not have alert yet)
  const knotWorkers = useMemo(() => {
    return workers.filter(w => w.state.state === 'knot');
  }, [workers]);

  const totalStuck = projectStuckAlerts.length + knotWorkers.filter(w => !stuckAlerts.has(w.worker)).length;

  const isFleetView = !projectName;

  return (
    <div className="stuck-workers-panel">
      <div className="stuck-workers-header">
        <h3>{isFleetView ? 'Fleet Stuck Workers' : 'Stuck Workers'}</h3>
        <div className="stuck-workers-summary">
          <div className="stuck-summary-item">
            <span className="stuck-label">Stuck Workers</span>
            <span className="stuck-value">{totalStuck}</span>
          </div>
        </div>
      </div>

      <div className="stuck-workers-content">
        {totalStuck === 0 ? (
          <div className="stuck-workers-empty">
            <p>No stuck workers</p>
            <p className="stuck-workers-empty-hint">
              Workers that exceed idle timeout, max runtime, or other stuck conditions will appear here.
            </p>
          </div>
        ) : (
          <div className="stuck-workers-list">
            {projectStuckAlerts.map(({ alert, beadTitle }) => (
              <StuckWorkerRow key={alert.worker} alert={alert} beadTitle={beadTitle} />
            ))}
            {/* Show knot workers without alerts */}
            {knotWorkers
              .filter(w => !stuckAlerts.has(w.worker))
              .map(worker => (
                <div key={worker.worker} className="stuck-worker-row">
                  <div className="stuck-worker-header">
                    <span className="stuck-worker-name">{worker.worker}</span>
                    <span className="stuck-reason-badge stuck-reason-knot">Knot</span>
                  </div>
                  <div className="stuck-worker-details">
                    <div className="stuck-detail-item">
                      <span className="stuck-detail-label">State</span>
                      <span className="stuck-detail-value">
                        {worker.state.state === 'knot' ? worker.state.reason : 'Unknown'}
                      </span>
                    </div>
                    <div className="stuck-detail-item">
                      <span className="stuck-detail-label">Last heartbeat</span>
                      <span className="stuck-detail-value">{formatHeartbeatDelta(worker.last_heartbeat)}</span>
                    </div>
                  </div>
                </div>
              ))}
          </div>
        )}

        {/* Notes */}
        <div className="stuck-workers-notes">
          <p className="stuck-note">
            <strong>Stuck detection:</strong> Workers are flagged as stuck when they exceed idle timeout,
            max runtime, or fail to show heartbeat transitions.
          </p>
          <p className="stuck-note">
            <strong>Last heartbeat delta:</strong> Time since the worker's last heartbeat.
            Large values indicate the worker may be unresponsive.
          </p>
        </div>
      </div>
    </div>
  );
}
