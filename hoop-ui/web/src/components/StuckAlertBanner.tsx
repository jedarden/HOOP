import { useAtomValue } from 'jotai';
import { stuckAlertsAtom, StuckAlert } from '../atoms';

const STUCK_REASON_LABELS: Record<string, string> = {
  idle_timeout: 'Idle Timeout',
  max_runtime_exceeded: 'Max Runtime Exceeded',
  content_seen_grace_exceeded: 'Content Seen Grace Exceeded',
  heartbeat_transition_silence: 'Heartbeat Transition Silence',
  repeated_retry: 'Repeated Retry',
};

function formatTimestamp(ts: string | null): string {
  if (!ts) return 'N/A';
  try {
    const date = new Date(ts);
    return date.toLocaleTimeString();
  } catch {
    return 'Invalid';
  }
}

function timeSince(ts: string | null): string {
  if (!ts) return 'N/A';
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

function AlertCard({ alert }: { alert: StuckAlert }) {
  const handleDismiss = () => {
    // Dismiss is handled by the parent component via filtering
  };

  return (
    <div className="stuck-alert-card" role="alert" aria-live="polite">
      <div className="alert-header">
        <span className="alert-icon">⚠</span>
        <span className="alert-title">Worker Stuck: {alert.worker}</span>
      </div>
      <div className="alert-body">
        <div className="alert-row">
          <span className="alert-label">Reason:</span>
          <span className="alert-value">{STUCK_REASON_LABELS[alert.reason] || alert.reason}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Bead:</span>
          <span className="alert-value">{alert.bead}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Elapsed:</span>
          <span className="alert-value">{Math.floor(alert.elapsed_secs / 60)}m {alert.elapsed_secs % 60}s</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Idle:</span>
          <span className="alert-value">{Math.floor(alert.idle_secs / 60)}m {alert.idle_secs % 60}s</span>
        </div>
        {(alert.reason === 'heartbeat_transition_silence' || alert.reason === 'repeated_retry') && (
          <>
            <div className="alert-row">
              <span className="alert-label">Last heartbeat:</span>
              <span className="alert-value" title={formatTimestamp(alert.last_heartbeat_at)}>
                {timeSince(alert.last_heartbeat_at)}
              </span>
            </div>
            {alert.last_transition_at && (
              <div className="alert-row">
                <span className="alert-label">Last transition:</span>
                <span className="alert-value" title={formatTimestamp(alert.last_transition_at)}>
                  {timeSince(alert.last_transition_at)}
                </span>
              </div>
            )}
            {alert.retry_count > 0 && (
              <div className="alert-row">
                <span className="alert-label">Retry count:</span>
                <span className="alert-value">{alert.retry_count}</span>
              </div>
            )}
          </>
        )}
        <div className="alert-message">{alert.message}</div>
      </div>
    </div>
  );
}

export function StuckAlertBanner() {
  const stuckAlerts = useAtomValue(stuckAlertsAtom);

  if (stuckAlerts.size === 0) {
    return null;
  }

  return (
    <div className="stuck-alerts-banner">
      <div className="stuck-alerts-container">
        {Array.from(stuckAlerts.values()).map((alert) => (
          <AlertCard key={alert.worker} alert={alert} />
        ))}
      </div>
    </div>
  );
}
