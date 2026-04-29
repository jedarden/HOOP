import { useAtom } from 'jotai';
import { costAnomalyAlertsAtom, CostAnomalyAlert } from '../atoms';

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

function formatCost(usd: number): string {
  return `$${usd.toFixed(2)}`;
}

function AlertCard({ alert, onDismiss }: { alert: CostAnomalyAlert; onDismiss: () => void }) {
  return (
    <div className="cost-anomaly-alert-card" role="alert" aria-live="polite">
      <div className="alert-header">
        <span className="alert-icon">💸</span>
        <span className="alert-title">Cost Anomaly Detected</span>
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
          <span className="alert-label">Stitch:</span>
          <span className="alert-value">
            <a href={`#/patterns/${alert.stitch_id}`} className="stitch-link">
              {alert.stitch_title}
            </a>
          </span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Project:</span>
          <span className="alert-value">{alert.project}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Actual cost:</span>
          <span className="alert-value alert-value-highlight">{formatCost(alert.cost_usd)}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Expected (mean):</span>
          <span className="alert-value">{formatCost(alert.band.mean_usd)}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Threshold (mean + 2σ):</span>
          <span className="alert-value">{formatCost(alert.band.upper_2sigma_usd)}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Similar stitches:</span>
          <span className="alert-value">{alert.band.similar_count}</span>
        </div>
        <div className="alert-row">
          <span className="alert-label">Detected:</span>
          <span className="alert-value" title={formatTimestamp(alert.detected_at)}>
            {timeSince(alert.detected_at)}
          </span>
        </div>

        {alert.closest_pattern && (
          <div className="pattern-match-section">
            <div className="pattern-match-header">
              <span className="pattern-match-label">Closest pattern match:</span>
              <span className="pattern-similarity">
                {(alert.closest_pattern.similarity * 100).toFixed(0)}% similar
              </span>
            </div>
            <div className="pattern-name">{alert.closest_pattern.pattern_name}</div>
            <div className="pattern-recommendation">
              <div className="pattern-recommendation-label">Recommended fix:</div>
              <pre className="pattern-template">{alert.closest_pattern.recommended_fix_template_md}</pre>
            </div>
            <a
              href={`#/patterns/${alert.closest_pattern.pattern_id}`}
              className="pattern-detail-link"
            >
              View pattern details →
            </a>
            <div className="pattern-actions">
              <button
                className="draft-mitigation-button"
                disabled
                title="Coming soon in Phase 4"
              >
                Draft mitigation Stitch
              </button>
              <span className="pattern-actions-hint">Phase 4 feature</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export function CostAnomalyAlertBanner() {
  const [costAnomalyAlerts, setCostAnomalyAlerts] = useAtom(costAnomalyAlertsAtom);

  if (costAnomalyAlerts.size === 0) {
    return null;
  }

  const handleDismiss = (alertId: string) => {
    setCostAnomalyAlerts((prev) => {
      const updated = new Map(prev);
      updated.delete(alertId);
      return updated;
    });
  };

  return (
    <div className="cost-anomaly-alerts-banner">
      <div className="cost-anomaly-alerts-container">
        {Array.from(costAnomalyAlerts.values()).map((alert) => (
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
