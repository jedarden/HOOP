import { useAtom, useAtomValue } from 'jotai';
import { saturationAlertsAtom, SaturationAlert } from '../atoms';

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

function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

function AlertCard({ alert, onDismiss }: { alert: SaturationAlert; onDismiss: (alertId: string) => void }) {
  return (
    <div className="saturation-alert-card" role="alert" aria-live="polite">
      <div className="alert-header">
        <span className="alert-icon">📊</span>
        <span className="alert-title">Capacity Saturation Alert</span>
        <button
          className="alert-dismiss"
          onClick={() => onDismiss(alert.alert_id)}
          aria-label="Dismiss alert"
          title="Dismiss this alert"
        >
          ✕
        </button>
      </div>
      <div className="alert-body">
        <div className="alert-row">
          <span className="alert-label">Account:</span>
          <span className="alert-value">{alert.account}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Model:</span>
          <span className="alert-value">{alert.model}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Utilization:</span>
          <span className="alert-value saturation-value">{formatPercent(alert.utilization_percent)}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Threshold:</span>
          <span className="alert-value">{formatPercent(alert.threshold_percent)}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Current TPM:</span>
          <span className="alert-value">{alert.current_tpm.toLocaleString()}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Detected:</span>
          <span className="alert-value" title={formatTimestamp(alert.detected_at)}>
            {timeSince(alert.detected_at)}
          </span>
        </div>
      </div>
    </div>
  );
}

export function SaturationAlertBanner() {
  const [saturationAlerts, setSaturationAlerts] = useAtom(saturationAlertsAtom);

  if (saturationAlerts.size === 0) {
    return null;
  }

  const handleDismiss = (alertId: string) => {
    setSaturationAlerts((prev) => {
      const updated = new Map(prev);
      updated.delete(alertId);
      return updated;
    });
  };

  return (
    <div className="saturation-alerts-banner">
      <div className="saturation-alerts-container">
        {Array.from(saturationAlerts.values()).map((alert) => (
          <AlertCard key={alert.alert_id} alert={alert} onDismiss={handleDismiss} />
        ))}
      </div>
    </div>
  );
}
