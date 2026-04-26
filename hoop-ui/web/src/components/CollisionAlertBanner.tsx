import { useAtom } from 'jotai';
import { collisionAlertsAtom, CollisionAlert } from '../atoms';

function formatTimestamp(ts: string): string {
  try {
    const date = new Date(ts);
    return date.toLocaleTimeString();
  } catch {
    return 'Invalid';
  }
}

function timeSince(ts: string): string {
  try {
    const date = new Date(ts);
    const now = new Date();
    const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    return `${hours}h ago`;
  } catch {
    return 'Invalid';
  }
}

function AlertCard({ alert, onDismiss }: { alert: CollisionAlert; onDismiss: () => void }) {
  return (
    <div className="collision-alert-card" role="alert" aria-live="polite">
      <div className="alert-header">
        <span className="alert-icon">🔄</span>
        <span className="alert-title">File Collision Detected</span>
        <button
          className="alert-dismiss"
          onClick={onDismiss}
          aria-label="Dismiss alert"
        >
          ×
        </button>
      </div>
      <div className="alert-body">
        <div className="alert-row">
          <span className="alert-label">Workers:</span>
          <span className="alert-value">
            <code>{alert.worker_a}</code> ↔ <code>{alert.worker_b}</code>
          </span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Beads:</span>
          <span className="alert-value">
            <span className="bead-link">{alert.bead_a}</span>
            {' '}↔{' '}
            <span className="bead-link">{alert.bead_b}</span>
          </span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Detected:</span>
          <span className="alert-value" title={formatTimestamp(alert.detected_at)}>
            {timeSince(alert.detected_at)}
          </span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Overlapping files ({alert.overlapping_files.length}):</span>
        </div>
        <div className="collision-files-list">
          {alert.overlapping_files.map((file) => (
            <div key={file} className="collision-file-item">
              <code>{file}</code>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

export function CollisionAlertBanner() {
  const [collisionAlerts, setCollisionAlerts] = useAtom(collisionAlertsAtom);

  if (collisionAlerts.size === 0) {
    return null;
  }

  const handleDismiss = (alertId: string) => {
    setCollisionAlerts((prev) => {
      const updated = new Map(prev);
      updated.delete(alertId);
      return updated;
    });
  };

  return (
    <div className="collision-alerts-banner">
      <div className="collision-alerts-container">
        {Array.from(collisionAlerts.values()).map((alert) => (
          <AlertCard
            key={alert.alert_id}
            alert={alert}
            onDismiss={() => handleDismiss(alert.alert_id)}
          />
        ))}
      </div>
    </div>
  );
}
