import { useAtomValue } from 'jotai';
import { useMemo, useState } from 'react';
import { conversationsAtom, workersAtom } from './atoms';

interface CapacityPanelProps {
  projectName: string;
}

interface AccountCapacity {
  accountId: string;
  adapter: string;
  model?: string;
  limits: {
    tokensPer5h: number;
    tokensPer7d: number;
    requestsPerDay: number;
    spendUsdPerDay: number;
  };
  usage: {
    tokensPer5h: number;
    tokensPer7d: number;
    requestsToday: number;
    spendUsdToday: number;
  };
  utilization: {
    tokensPer5h: number;
    tokensPer7d: number;
    requestsToday: number;
    spendToday: number;
  };
  burnRate: {
    tokensPerMinute: number;
  };
  forecast: {
    minutesUntilFull5h: number | null;
    minutesUntilFull7d: number | null;
  };
}

// Claude rate limits (default - should come from config)
const CLAUDE_LIMITS: AccountCapacity['limits'] = {
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

function formatForecast(minutes: number | null): string {
  if (minutes === null || minutes === Infinity || minutes <= 0) return 'N/A';
  if (minutes < 60) return `~${Math.floor(minutes)}m`;
  if (minutes < 1440) return `~${Math.floor(minutes / 60)}h`;
  return `~${Math.floor(minutes / 1440)}d`;
}

interface CapacityRowProps {
  account: AccountCapacity;
  activeWorkers: number;
}

function CapacityRow({ account, activeWorkers }: CapacityRowProps) {
  const [tooltip, setTooltip] = useState<{
    show: boolean;
    x: number;
    y: number;
    content: React.ReactNode;
  }>({ show: false, x: 0, y: 0, content: null });

  const handleMouseEnter = (e: React.MouseEvent, content: React.ReactNode) => {
    const rect = e.currentTarget.getBoundingClientRect();
    setTooltip({
      show: true,
      x: rect.left + rect.width / 2,
      y: rect.top - 8,
      content,
    });
  };

  const handleMouseLeave = () => {
    setTooltip(prev => ({ ...prev, show: false }));
  };

  const tokens5hLevel = getUtilizationLevel(account.utilization.tokensPer5h);
  const tokens7dLevel = getUtilizationLevel(account.utilization.tokensPer7d);

  return (
    <div className="capacity-row">
      <div className="capacity-row-header">
        <div className="capacity-account-info">
          <span className="capacity-adapter-name">{account.adapter}</span>
          {account.model && <span className="capacity-model-name">{account.model}</span>}
        </div>
        {activeWorkers > 0 && (
          <span className="capacity-active-workers">{activeWorkers} active</span>
        )}
      </div>

      <div className="capacity-meters">
        {/* 5h meter */}
        <div className="capacity-meter">
          <div className="meter-label">
            <span>5h</span>
            <span
              className="meter-percent"
              style={{ color: getUtilizationColor(tokens5hLevel) }}
              onMouseEnter={(e) => handleMouseEnter(e, (
                <div className="tooltip-content">
                  <div className="tooltip-row">
                    <span>Used:</span>
                    <strong>{formatNumber(account.usage.tokensPer5h)}</strong>
                  </div>
                  <div className="tooltip-row">
                    <span>Limit:</span>
                    <strong>{formatNumber(account.limits.tokensPer5h)}</strong>
                  </div>
                  <div className="tooltip-row">
                    <span>Remaining:</span>
                    <strong>{formatNumber(account.limits.tokensPer5h - account.usage.tokensPer5h)}</strong>
                  </div>
                </div>
              ))}
              onMouseLeave={handleMouseLeave}
            >
              {account.utilization.tokensPer5h.toFixed(0)}%
            </span>
          </div>
          <div className="meter-track">
            <div
              className={`meter-fill utilization-${tokens5hLevel}`}
              style={{ width: `${Math.min(account.utilization.tokensPer5h, 100)}%` }}
            />
            {/* Forecast arrow */}
            {account.forecast.minutesUntilFull5h !== null && account.forecast.minutesUntilFull5h < 300 && (
              <div
                className="meter-forecast-arrow"
                style={{
                  left: `${Math.min(100, account.utilization.tokensPer5h + (account.burnRate.tokensPerMinute * 30 / account.limits.tokensPer5h) * 100)}%`,
                }}
                onMouseEnter={(e) => handleMouseEnter(e, (
                  <div className="tooltip-content">
                    <div className="tooltip-row">
                      <span>Burn rate:</span>
                      <strong>{formatNumber(account.burnRate.tokensPerMinute)}/min</strong>
                    </div>
                    <div className="tooltip-row">
                      <span>Full in:</span>
                      <strong>{formatForecast(account.forecast.minutesUntilFull5h)}</strong>
                    </div>
                  </div>
                ))}
                onMouseLeave={handleMouseLeave}
              >
                ▼
              </div>
            )}
          </div>
        </div>

        {/* 7d meter */}
        <div className="capacity-meter">
          <div className="meter-label">
            <span>7d</span>
            <span
              className="meter-percent"
              style={{ color: getUtilizationColor(tokens7dLevel) }}
              onMouseEnter={(e) => handleMouseEnter(e, (
                <div className="tooltip-content">
                  <div className="tooltip-row">
                    <span>Used:</span>
                    <strong>{formatNumber(account.usage.tokensPer7d)}</strong>
                  </div>
                  <div className="tooltip-row">
                    <span>Limit:</span>
                    <strong>{formatNumber(account.limits.tokensPer7d)}</strong>
                  </div>
                  <div className="tooltip-row">
                    <span>Remaining:</span>
                    <strong>{formatNumber(account.limits.tokensPer7d - account.usage.tokensPer7d)}</strong>
                  </div>
                  {account.forecast.minutesUntilFull7d !== null && (
                    <>
                      <div className="tooltip-divider" />
                      <div className="tooltip-row">
                        <span>Full in:</span>
                        <strong>{formatForecast(account.forecast.minutesUntilFull7d)}</strong>
                      </div>
                    </>
                  )}
                </div>
              ))}
              onMouseLeave={handleMouseLeave}
            >
              {account.utilization.tokensPer7d.toFixed(0)}%
            </span>
          </div>
          <div className="meter-track">
            <div
              className={`meter-fill utilization-${tokens7dLevel}`}
              style={{ width: `${Math.min(account.utilization.tokensPer7d, 100)}%` }}
            />
            {/* Forecast arrow for 7d if approaching limit */}
            {account.forecast.minutesUntilFull7d !== null && account.forecast.minutesUntilFull7d < 1440 && (
              <div
                className="meter-forecast-arrow"
                style={{
                  left: `${Math.min(100, account.utilization.tokensPer7d + (account.burnRate.tokensPerMinute * 60 / account.limits.tokensPer7d) * 100)}%`,
                }}
                onMouseEnter={(e) => handleMouseEnter(e, (
                  <div className="tooltip-content">
                    <div className="tooltip-row">
                      <span>Full in:</span>
                      <strong>{formatForecast(account.forecast.minutesUntilFull7d)}</strong>
                    </div>
                  </div>
                ))}
                onMouseLeave={handleMouseLeave}
              >
                ▼
              </div>
            )}
          </div>
        </div>

        {/* Spend meter (where applicable) */}
        {account.limits.spendUsdPerDay > 0 && (
          <div className="capacity-meter capacity-meter-spend">
            <div className="meter-label">
              <span>$</span>
              <span
                className="meter-percent"
                style={{ color: getUtilizationColor(getUtilizationLevel(account.utilization.spendToday)) }}
                onMouseEnter={(e) => handleMouseEnter(e, (
                  <div className="tooltip-content">
                    <div className="tooltip-row">
                      <span>Spent today:</span>
                      <strong>${account.usage.spendUsdToday.toFixed(2)}</strong>
                    </div>
                    <div className="tooltip-row">
                      <span>Daily limit:</span>
                      <strong>${account.limits.spendUsdPerDay.toFixed(2)}</strong>
                    </div>
                  </div>
                ))}
                onMouseLeave={handleMouseLeave}
              >
                {account.utilization.spendToday.toFixed(0)}%
              </span>
            </div>
            <div className="meter-track">
              <div
                className={`meter-fill utilization-${getUtilizationLevel(account.utilization.spendToday)}`}
                style={{ width: `${Math.min(account.utilization.spendToday, 100)}%` }}
              />
            </div>
          </div>
        )}
      </div>

      {/* Forecast text */}
      {(account.forecast.minutesUntilFull5h !== null && account.forecast.minutesUntilFull5h < 60) && (
        <div className="capacity-forecast-text">
          Full in {formatForecast(account.forecast.minutesUntilFull5h)} at current burn
        </div>
      )}

      {/* Floating tooltip */}
      {tooltip.show && (
        <div
          className="capacity-tooltip"
          style={{
            left: `${tooltip.x}px`,
            top: `${tooltip.y}px`,
            transform: 'translate(-50%, -100%)',
          }}
        >
          {tooltip.content}
        </div>
      )}
    </div>
  );
}

export default function CapacityPanel({ projectName: _projectName }: CapacityPanelProps) {
  const conversations = useAtomValue(conversationsAtom);
  const workers = useAtomValue(workersAtom);

  // Calculate capacity by account
  const capacityByAccount = useMemo(() => {
    const accounts = new Map<string, AccountCapacity>();

    const now = new Date();
    const fiveHoursAgo = new Date(now.getTime() - 5 * 60 * 60 * 1000);
    const sevenDaysAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000);

    conversations.forEach(conv => {
      const convDate = new Date(conv.created_at);
      const accountId = conv.provider; // In real impl, this would be actual account ID

      const existing = accounts.get(accountId) || {
        accountId,
        adapter: conv.provider,
        limits: CLAUDE_LIMITS,
        usage: {
          tokensPer5h: 0,
          tokensPer7d: 0,
          requestsToday: 0,
          spendUsdToday: 0,
        },
        utilization: {
          tokensPer5h: 0,
          tokensPer7d: 0,
          requestsToday: 0,
          spendToday: 0,
        },
        burnRate: {
          tokensPerMinute: 0,
        },
        forecast: {
          minutesUntilFull5h: null,
          minutesUntilFull7d: null,
        },
      };

      // Count tokens in windows
      const tokens = conv.total_tokens;
      if (convDate > fiveHoursAgo) {
        existing.usage.tokensPer5h += tokens;
      }
      if (convDate > sevenDaysAgo) {
        existing.usage.tokensPer7d += tokens;
      }
      if (convDate > today) {
        existing.usage.requestsToday += 1;
      }
      // Calculate burn rate from last hour
      if (convDate > oneHourAgo) {
        existing.burnRate.tokensPerMinute += tokens / 60;
      }

      accounts.set(accountId, existing);
    });

    // Calculate utilization and forecast
    accounts.forEach(account => {
      account.utilization.tokensPer5h = (account.usage.tokensPer5h / account.limits.tokensPer5h) * 100;
      account.utilization.tokensPer7d = (account.usage.tokensPer7d / account.limits.tokensPer7d) * 100;
      account.utilization.requestsToday = (account.usage.requestsToday / account.limits.requestsPerDay) * 100;
      account.utilization.spendToday = (account.usage.spendUsdToday / account.limits.spendUsdPerDay) * 100;

      // Forecast: minutes until full at current burn rate
      if (account.burnRate.tokensPerMinute > 0) {
        const remaining5h = account.limits.tokensPer5h - account.usage.tokensPer5h;
        const remaining7d = account.limits.tokensPer7d - account.usage.tokensPer7d;
        account.forecast.minutesUntilFull5h = remaining5h > 0 ? remaining5h / account.burnRate.tokensPerMinute : 0;
        account.forecast.minutesUntilFull7d = remaining7d > 0 ? remaining7d / account.burnRate.tokensPerMinute : 0;
      } else {
        account.forecast.minutesUntilFull5h = null;
        account.forecast.minutesUntilFull7d = null;
      }
    });

    return Array.from(accounts.values());
  }, [conversations]);

  // Get active workers by adapter
  const activeWorkersByAdapter = useMemo(() => {
    const byAdapter = new Map<string, number>();
    workers.forEach(w => {
      if (w.state.state === 'executing' && w.liveness === 'Live') {
        const adapter = w.state.adapter || 'unknown';
        byAdapter.set(adapter, (byAdapter.get(adapter) || 0) + 1);
      }
    });
    return byAdapter;
  }, [workers]);

  const activeWorkerCount = workers.filter(w => w.state.state === 'executing' && w.liveness === 'Live').length;

  return (
    <div className="capacity-panel">
      <div className="capacity-header">
        <h3>Capacity</h3>
        <div className="capacity-summary">
          <div className="capacity-summary-item">
            <span className="capacity-label">Active Workers</span>
            <span className="capacity-value">{activeWorkerCount}</span>
          </div>
          <div className="capacity-summary-item">
            <span className="capacity-label">Accounts</span>
            <span className="capacity-value">{capacityByAccount.length}</span>
          </div>
        </div>
      </div>

      <div className="capacity-content">
        {capacityByAccount.length === 0 ? (
          <p className="capacity-empty">No capacity data available yet</p>
        ) : (
          <div className="capacity-rows">
            {capacityByAccount.map(account => (
              <CapacityRow
                key={account.accountId}
                account={account}
                activeWorkers={activeWorkersByAdapter.get(account.adapter) || 0}
              />
            ))}
          </div>
        )}

        {/* Legend */}
        <div className="capacity-legend">
          <div className="legend-item">
            <span className="legend-dot utilization-low" />
            <span className="legend-label">&lt;40%</span>
          </div>
          <div className="legend-item">
            <span className="legend-dot utilization-medium" />
            <span className="legend-label">40-70%</span>
          </div>
          <div className="legend-item">
            <span className="legend-dot utilization-high" />
            <span className="legend-label">70-90%</span>
          </div>
          <div className="legend-item">
            <span className="legend-dot utilization-critical" />
            <span className="legend-label">&gt;90%</span>
          </div>
          <div className="legend-item legend-forecast">
            <span className="legend-arrow">▼</span>
            <span className="legend-label">Forecast</span>
          </div>
        </div>

        {/* Notes */}
        <div className="capacity-notes">
          <p className="capacity-note">
            <strong>Observation only:</strong> HOOP displays capacity but does not enforce limits.
          </p>
          <p className="capacity-note">
            <strong>Forecast arrows</strong> show when the limit will be reached at current burn rate.
          </p>
        </div>
      </div>
    </div>
  );
}
