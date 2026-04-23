import { useAtomValue } from 'jotai';
import { useState } from 'react';
import { capacityAtom, workersAtom, type AccountCapacity } from './atoms';

interface CapacityPanelProps {
  projectName: string;
}

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

function formatForecast(minutes: number | null | undefined): string {
  if (minutes === null || minutes === undefined || minutes === Infinity || minutes <= 0) return 'N/A';
  if (minutes < 60) return `~${Math.floor(minutes)}m`;
  if (minutes < 1440) return `~${Math.floor(minutes / 60)}h`;
  return `~${Math.floor(minutes / 1440)}d`;
}

function formatResetsAt(iso?: string | null): string {
  if (!iso) return '';
  try {
    const d = new Date(iso);
    const now = new Date();
    const diffMin = (d.getTime() - now.getTime()) / 60000;
    if (diffMin < 0) return '';
    if (diffMin < 60) return `resets in ~${Math.floor(diffMin)}m`;
    if (diffMin < 1440) return `resets in ~${Math.floor(diffMin / 60)}h`;
    return `resets in ~${Math.floor(diffMin / 1440)}d`;
  } catch {
    return '';
  }
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

  const level5h = getUtilizationLevel(account.utilization_5h);
  const level7d = getUtilizationLevel(account.utilization_7d);

  return (
    <div className="capacity-row">
      <div className="capacity-row-header">
        <div className="capacity-account-info">
          <span className="capacity-adapter-name">{account.account_id}</span>
          <span className="capacity-model-name">{account.plan_type} · {account.rate_limit_tier.replace('default_claude_', '')}</span>
        </div>
        <div className="capacity-account-meta">
          {activeWorkers > 0 && (
            <span className="capacity-active-workers">{activeWorkers} active</span>
          )}
          <span className="capacity-source-badge" title={`Data from ${account.source}`}>
            {account.source === 'api_cache' ? 'API' : 'JSONL'}
          </span>
        </div>
      </div>

      <div className="capacity-meters">
        {/* 5h meter */}
        <div className="capacity-meter">
          <div className="meter-label">
            <span>5h</span>
            <span
              className="meter-percent"
              style={{ color: getUtilizationColor(level5h) }}
              onMouseEnter={(e) => handleMouseEnter(e, (
                <div className="tooltip-content">
                  <div className="tooltip-row">
                    <span>Weighted tokens:</span>
                    <strong>{formatNumber(account.tokens_5h)}</strong>
                  </div>
                  <div className="tooltip-row">
                    <span>Turns:</span>
                    <strong>{account.turns_5h}</strong>
                  </div>
                  {account.resets_at_5h && (
                    <div className="tooltip-row">
                      <span>Resets:</span>
                      <strong>{formatResetsAt(account.resets_at_5h)}</strong>
                    </div>
                  )}
                </div>
              ))}
              onMouseLeave={handleMouseLeave}
            >
              {account.utilization_5h.toFixed(0)}%
            </span>
          </div>
          <div className="meter-track">
            <div
              className={`meter-fill utilization-${level5h}`}
              style={{ width: `${Math.min(account.utilization_5h, 100)}%` }}
            />
            {account.forecast_full_5h_min != null && account.forecast_full_5h_min < 300 && (
              <div
                className="meter-forecast-arrow"
                style={{
                  left: `${Math.min(100, account.utilization_5h + 5)}%`,
                }}
                onMouseEnter={(e) => handleMouseEnter(e, (
                  <div className="tooltip-content">
                    <div className="tooltip-row">
                      <span>Burn rate:</span>
                      <strong>{formatNumber(account.burn_rate_per_min)}/min</strong>
                    </div>
                    <div className="tooltip-row">
                      <span>Full in:</span>
                      <strong>{formatForecast(account.forecast_full_5h_min)}</strong>
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
              style={{ color: getUtilizationColor(level7d) }}
              onMouseEnter={(e) => handleMouseEnter(e, (
                <div className="tooltip-content">
                  <div className="tooltip-row">
                    <span>Weighted tokens:</span>
                    <strong>{formatNumber(account.tokens_7d)}</strong>
                  </div>
                  <div className="tooltip-row">
                    <span>Turns:</span>
                    <strong>{account.turns_7d}</strong>
                  </div>
                  {account.forecast_full_7d_min != null && (
                    <>
                      <div className="tooltip-divider" />
                      <div className="tooltip-row">
                        <span>Full in:</span>
                        <strong>{formatForecast(account.forecast_full_7d_min)}</strong>
                      </div>
                    </>
                  )}
                </div>
              ))}
              onMouseLeave={handleMouseLeave}
            >
              {account.utilization_7d.toFixed(0)}%
            </span>
          </div>
          <div className="meter-track">
            <div
              className={`meter-fill utilization-${level7d}`}
              style={{ width: `${Math.min(account.utilization_7d, 100)}%` }}
            />
            {account.forecast_full_7d_min != null && account.forecast_full_7d_min < 1440 && (
              <div
                className="meter-forecast-arrow"
                style={{
                  left: `${Math.min(100, account.utilization_7d + 5)}%`,
                }}
                onMouseEnter={(e) => handleMouseEnter(e, (
                  <div className="tooltip-content">
                    <div className="tooltip-row">
                      <span>Full in:</span>
                      <strong>{formatForecast(account.forecast_full_7d_min)}</strong>
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
      </div>

      {/* Forecast text */}
      {account.forecast_full_5h_min != null && account.forecast_full_5h_min < 60 && (
        <div className="capacity-forecast-text">
          5h limit in {formatForecast(account.forecast_full_5h_min)} at current burn ({formatNumber(account.burn_rate_per_min)}/min)
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
  const accounts = useAtomValue(capacityAtom);
  const workers = useAtomValue(workersAtom);

  const activeWorkersByAccount = (() => {
    const byAccount = new Map<string, number>();
    workers.forEach(w => {
      if (w.state.state === 'executing' && w.liveness === 'Live') {
        // Map adapter to account — claude adapters all map to claude-default
        const key = w.state.adapter === 'claude' ? 'claude-default' : w.state.adapter;
        byAccount.set(key, (byAccount.get(key) || 0) + 1);
      }
    });
    return byAccount;
  })();

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
            <span className="capacity-value">{accounts.length}</span>
          </div>
        </div>
      </div>

      <div className="capacity-content">
        {accounts.length === 0 ? (
          <p className="capacity-empty">No capacity data available yet</p>
        ) : (
          <div className="capacity-rows">
            {accounts.map(account => (
              <CapacityRow
                key={account.account_id}
                account={account}
                activeWorkers={activeWorkersByAccount.get(account.account_id) || 0}
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
          {accounts.length > 0 && (
            <p className="capacity-note">
              <strong>Source:</strong> {accounts[0].source === 'api_cache' ? 'Claude API cache (exact)' : 'JSONL estimate (±5%)'}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
