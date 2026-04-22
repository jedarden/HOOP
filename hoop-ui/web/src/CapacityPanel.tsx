import { useAtomValue } from 'jotai';
import { useMemo } from 'react';
import { conversationsAtom, workersAtom } from './atoms';

interface CapacityPanelProps {
  projectName: string;
}

interface AccountCapacity {
  accountId: string;
  adapter: string;
  limits: {
    tokens5h: number;
    tokens7d: number;
    requestsPerDay: number;
    spendUsdPerDay: number;
  };
  usage: {
    tokens5h: number;
    tokens7d: number;
    requestsToday: number;
    spendUsdToday: number;
  };
  utilization: {
    tokens5h: number;
    tokens7d: number;
    requestsToday: number;
    spendToday: number;
  };
}

// Claude rate limits (default - should come from config)
const CLAUDE_LIMITS = {
  tokensPer5h: 200000,
  tokensPer7d: 1000000,
  requestsPerDay: 500,
  spendUsdPerDay: 200,
};

function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

function getUtilizationLevel(utilization: number): 'low' | 'medium' | 'high' | 'critical' {
  if (utilization >= 90) return 'critical';
  if (utilization >= 70) return 'high';
  if (utilization >= 40) return 'medium';
  return 'low';
}

function getUtilizationColor(level: string): string {
  switch (level) {
    case 'critical': return '#ea4335';
    case 'high': return '#fbbc04';
    case 'medium': return '#f9ab00';
    case 'low': return '#34a853';
    default: return '#999';
  }
}

export default function CapacityPanel({ projectName }: CapacityPanelProps) {
  const conversations = useAtomValue(conversationsAtom);
  const workers = useAtomValue(workersAtom);

  // Calculate capacity by account
  const capacityByAccount = useMemo(() => {
    const accounts = new Map<string, AccountCapacity>();

    const now = new Date();
    const fiveHoursAgo = new Date(now.getTime() - 5 * 60 * 60 * 1000);
    const sevenDaysAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());

    conversations.forEach(conv => {
      const convDate = new Date(conv.created_at);
      const accountId = conv.provider; // In real impl, this would be actual account ID

      const existing = accounts.get(accountId) || {
        accountId,
        adapter: conv.provider,
        limits: CLAUDE_LIMITS,
        usage: {
          tokens5h: 0,
          tokens7d: 0,
          requestsToday: 0,
          spendUsdToday: 0,
        },
        utilization: {
          tokens5h: 0,
          tokens7d: 0,
          requestsToday: 0,
          spendToday: 0,
        },
      };

      // Count tokens in windows
      const tokens = conv.total_tokens;
      if (convDate > fiveHoursAgo) {
        existing.usage.tokens5h += tokens;
      }
      if (convDate > sevenDaysAgo) {
        existing.usage.tokens7d += tokens;
      }
      if (convDate > today) {
        existing.usage.requestsToday += 1;
      }

      accounts.set(accountId, existing);
    });

    // Calculate utilization
    accounts.forEach(account => {
      account.utilization.tokens5h = (account.usage.tokens5h / account.limits.tokens5h) * 100;
      account.utilization.tokens7d = (account.usage.tokens7d / account.limits.tokens7d) * 100;
      account.utilization.requestsToday = (account.usage.requestsToday / account.limits.requestsPerDay) * 100;
      account.utilization.spendToday = (account.usage.spendUsdToday / account.limits.spendUsdPerDay) * 100;
    });

    return Array.from(accounts.values());
  }, [conversations]);

  // Get active workers by adapter
  const activeWorkersByAdapter = useMemo(() => {
    const byAdapter = new Map<string, number>();
    workers.forEach(w => {
      if (w.state.state === 'executing' && w.liveness === 'Live') {
        const adapter = w.state.adapter;
        byAdapter.set(adapter, (byAdapter.get(adapter) || 0) + 1);
      }
    });
    return byAdapter;
  }, [workers]);

  return (
    <div className="capacity-panel">
      <div className="capacity-header">
        <h3>Capacity & Rate Limits</h3>
        <div className="capacity-summary">
          <div className="capacity-summary-item">
            <span className="capacity-label">Active Workers</span>
            <span className="capacity-value">{workers.filter(w => w.state.state === 'executing' && w.liveness === 'Live').length}</span>
          </div>
        </div>
      </div>

      <div className="capacity-sections">
        {/* Capacity by Account */}
        <section className="capacity-section">
          <h4>Rate Limit Status by Account</h4>
          {capacityByAccount.length === 0 ? (
            <p className="capacity-empty">No capacity data available yet</p>
          ) : (
            <div className="capacity-accounts">
              {capacityByAccount.map(account => {
                const tokens5hLevel = getUtilizationLevel(account.utilization.tokens5h);
                const tokens7dLevel = getUtilizationLevel(account.utilization.tokens7d);

                return (
                  <div key={account.accountId} className="capacity-account">
                    <div className="account-header">
                      <span className="account-name">{account.adapter}</span>
                      <span className="account-id">{account.accountId}</span>
                    </div>

                    {/* 5-hour window */}
                    <div className="capacity-metric">
                      <div className="metric-header">
                        <span className="metric-label">5-Hour Window</span>
                        <span className="metric-value" style={{ color: getUtilizationColor(tokens5hLevel) }}>
                          {account.utilization.tokens5h.toFixed(1)}%
                        </span>
                      </div>
                      <div className="capacity-bar">
                        <div
                          className={`capacity-bar-fill utilization-${tokens5hLevel}`}
                          style={{ width: `${Math.min(account.utilization.tokens5h, 100)}%` }}
                        />
                      </div>
                      <div className="metric-details">
                        <span>{formatNumber(account.usage.tokens5h)} / {formatNumber(account.limits.tokens5h)} tokens</span>
                      </div>
                    </div>

                    {/* 7-day window */}
                    <div className="capacity-metric">
                      <div className="metric-header">
                        <span className="metric-label">7-Day Window</span>
                        <span className="metric-value" style={{ color: getUtilizationColor(tokens7dLevel) }}>
                          {account.utilization.tokens7d.toFixed(1)}%
                        </span>
                      </div>
                      <div className="capacity-bar">
                        <div
                          className={`capacity-bar-fill utilization-${tokens7dLevel}`}
                          style={{ width: `${Math.min(account.utilization.tokens7d, 100)}%` }}
                        />
                      </div>
                      <div className="metric-details">
                        <span>{formatNumber(account.usage.tokens7d)} / {formatNumber(account.limits.tokens7d)} tokens</span>
                      </div>
                    </div>

                    {/* Daily requests */}
                    <div className="capacity-metric">
                      <div className="metric-header">
                        <span className="metric-label">Daily Requests</span>
                        <span className="metric-value">
                          {account.usage.requestsToday} / {account.limits.requestsPerDay}
                        </span>
                      </div>
                      <div className="capacity-bar">
                        <div
                          className="capacity-bar-fill"
                          style={{ width: `${(account.usage.requestsToday / account.limits.requestsPerDay) * 100}%` }}
                        />
                      </div>
                    </div>

                    {/* Active workers */}
                    {activeWorkersByAdapter.has(account.adapter) && (
                      <div className="capacity-workers">
                        <span className="workers-count">{activeWorkersByAdapter.get(account.adapter)} active worker(s)</span>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </section>

        {/* Capacity Legend */}
        <section className="capacity-section">
          <h4>Utilization Levels</h4>
          <div className="capacity-legend">
            <div className="legend-item">
              <span className="legend-dot utilization-low" />
              <span className="legend-label">Low (&lt;40%)</span>
            </div>
            <div className="legend-item">
              <span className="legend-dot utilization-medium" />
              <span className="legend-label">Medium (40-70%)</span>
            </div>
            <div className="legend-item">
              <span className="legend-dot utilization-high" />
              <span className="legend-label">High (70-90%)</span>
            </div>
            <div className="legend-item">
              <span className="legend-dot utilization-critical" />
              <span className="legend-label">Critical (&gt;90%)</span>
            </div>
          </div>
        </section>

        {/* Notes */}
        <section className="capacity-section">
          <h4>Capacity Notes</h4>
          <div className="capacity-notes">
            <p className="capacity-note">
              <strong>Observation only:</strong> HOOP displays capacity utilization but does not enforce limits.
              Rate limiting and throttling are managed by NEEDLE or the adapter directly.
            </p>
            <p className="capacity-note">
              <strong>5h/7d windows:</strong> Claude uses rolling windows for token limits. The 5-hour window
              resets continuously; the 7-day window tracks usage over the past week.
            </p>
            <p className="capacity-note">
              <strong>Burn rate forecast:</strong> At current usage patterns, the 5h window will exhaust in
              approximately {capacityByAccount.length > 0 && capacityByAccount[0].utilization.tokens5h > 0
                ? `${Math.floor(300 / (capacityByAccount[0].utilization.tokens5h / 100 * 300))} minutes`
                : 'N/A'}.
            </p>
          </div>
        </section>
      </div>
    </div>
  );
}
