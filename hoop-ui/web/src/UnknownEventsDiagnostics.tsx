import { useState, useEffect, useCallback } from 'react';

interface LabeledEntry {
  adapter: string;
  event_kind: string;
  count: number;
}

interface UnknownEventSample {
  adapter: string;
  event_kind: string;
  raw_event: string;
  timestamp: string;
  source_path?: string;
  line_number?: number;
}

interface UnknownEventsResponse {
  total_count: number;
  labeled_totals: LabeledEntry[];
  daemon_version: string;
  schema_version: string;
}

interface UnknownEventSamplesResponse {
  samples: UnknownEventSample[];
  total_count: number;
  daemon_version: string;
  schema_version: string;
}

export default function UnknownEventsDiagnostics() {
  const [summary, setSummary] = useState<UnknownEventsResponse | null>(null);
  const [samples, setSamples] = useState<UnknownEventSample[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(true);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [summaryRes, samplesRes] = await Promise.all([
        fetch('/api/diagnostics/unknown-events'),
        fetch('/api/diagnostics/unknown-events/samples'),
      ]);

      if (!summaryRes.ok || !samplesRes.ok) {
        throw new Error(`HTTP ${summaryRes.status || samplesRes.status}`);
      }

      const [summaryData, samplesData]: [UnknownEventsResponse, UnknownEventSamplesResponse] =
        await Promise.all([summaryRes.json(), samplesRes.json()]);

      setSummary(summaryData);
      setSamples(samplesData.samples);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();

    if (autoRefresh) {
      const interval = setInterval(fetchData, 30000); // Refresh every 30s
      return () => clearInterval(interval);
    }
  }, [fetchData, autoRefresh]);

  const formatTimestamp = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (seconds < 60) return `${seconds}s ago`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    return date.toLocaleDateString();
  };

  const hasVersionDrift = summary?.daemon_version && summary?.schema_version
    ? summary.daemon_version !== summary.schema_version
    : false;

  const groupedByAdapter = samples.reduce((acc, sample) => {
    if (!acc[sample.adapter]) {
      acc[sample.adapter] = [];
    }
    acc[sample.adapter].push(sample);
    return acc;
  }, {} as Record<string, UnknownEventSample[]>);

  return (
    <div className="unknown-events-diagnostics">
      <div className="diagnostics-header">
        <div className="header-title">
          <h2>Unknown Events Diagnostics</h2>
          <p className="header-description">
            Never-silent-drop invariant: unknown event kinds from adapters are logged and counted.
          </p>
        </div>
        <div className="header-controls">
          <label className="auto-refresh-toggle">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            Auto-refresh (30s)
          </label>
          <button className="refresh-button" onClick={fetchData}>
            Refresh
          </button>
        </div>
      </div>

      {error && (
        <div className="banner banner-error" role="alert">
          <span className="banner-icon">⚠️</span>
          <span className="banner-message">{error}</span>
          <button className="banner-dismiss" onClick={() => setError(null)}>✕</button>
        </div>
      )}

      {loading && !summary ? (
        <div className="loading-state">Loading diagnostics...</div>
      ) : summary ? (
        <>
          {/* Summary Stats */}
          <div className="diagnostics-summary">
            <div className="summary-card">
              <span className="summary-value">{summary.total_count}</span>
              <span className="summary-label">Total Unknown Events</span>
            </div>
            <div className="summary-card">
              <span className="summary-value">{summary.labeled_totals.length}</span>
              <span className="summary-label">Unique (adapter, kind) pairs</span>
            </div>
            <div className="summary-card">
              <span className="summary-value">{samples.length}</span>
              <span className="summary-label">Buffered Samples</span>
            </div>
            <div className={`summary-card ${hasVersionDrift ? 'version-drift' : ''}`}>
              <span className="summary-value">
                v{summary.daemon_version} / s{summary.schema_version}
              </span>
              <span className="summary-label">
                {hasVersionDrift ? '⚠️ Version Drift' : 'Versions Match'}
              </span>
            </div>
          </div>

          {/* Labeled Totals by Adapter */}
          {summary.labeled_totals.length > 0 && (
            <div className="labeled-totals-section">
              <h3>Counts by Adapter and Event Kind</h3>
              <div className="labeled-totals-grid">
                {summary.labeled_totals.map((entry, idx) => (
                  <div key={`${entry.adapter}-${entry.event_kind}-${idx}`} className="labeled-total-card">
                    <div className="labeled-total-header">
                      <span className="labeled-total-adapter">{entry.adapter}</span>
                      <span className="labeled-total-count">{entry.count}</span>
                    </div>
                    <code className="labeled-total-kind">{entry.event_kind}</code>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Raw Event Samples */}
          {samples.length > 0 ? (
            <div className="samples-section">
              <h3>Last {samples.length} Unknown Event Samples</h3>
              {Object.entries(groupedByAdapter).map(([adapter, adapterSamples]) => (
                <div key={adapter} className="adapter-group">
                  <h4 className="adapter-name">{adapter}</h4>
                  <div className="samples-list">
                    {adapterSamples.map((sample, idx) => (
                      <div key={`${sample.timestamp}-${idx}`} className="sample-card">
                        <div className="sample-header">
                          <span className="sample-kind">{sample.event_kind}</span>
                          <span className="sample-time">{formatTimestamp(sample.timestamp)}</span>
                        </div>
                        {(sample.source_path || sample.line_number) && (
                          <div className="sample-source">
                            {sample.source_path && <span>{sample.source_path}</span>}
                            {sample.line_number && <span>:{sample.line_number}</span>}
                          </div>
                        )}
                        <pre className="sample-raw">
                          <code>{sample.raw_event}</code>
                        </pre>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="empty-state">
              <p>No unknown events recorded</p>
              <p className="empty-hint">All events from adapters are recognized.</p>
            </div>
          )}

          {/* File Issue Link */}
          {summary.labeled_totals.length > 0 && (
            <div className="issue-cta">
              <p>
                <strong>Unknown events indicate adapter drift.</strong> File an issue to update the
                adapter schema:
              </p>
              <a
                href="https://github.com/jedarden/HOOP/issues/new?template=adapter-drift.md"
                target="_blank"
                rel="noopener noreferrer"
                className="issue-link"
              >
                File Adapter Update Issue
              </a>
            </div>
          )}
        </>
      ) : null}
    </div>
  );
}
