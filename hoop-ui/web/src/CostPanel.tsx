import { useAtomValue } from 'jotai';
import { useState, useMemo, useEffect } from 'react';
import { conversationsAtom, capacityAtom, Conversation, type AccountCapacity } from './atoms';

interface CostPanelProps {
  projectName: string;
  conversations?: Conversation[];
}

interface CostBreakdown {
  adapter: string;
  model: string | null;
  totalTokens: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  totalCost: number;
  requestCount: number;
}

interface TimeRangeCost {
  label: string;
  tokens: number;
  cost: number;
}

interface CostBucket {
  date: string;
  project: string;
  adapter: string;
  model: string;
  strand: string | null;
  /** "fleet" (NEEDLE worker session) or "operator" (all others). Set at aggregation time. */
  classification: string;
  usage: {
    input_tokens: number;
    output_tokens: number;
    cache_read_tokens: number;
    cache_write_tokens: number;
  };
  request_count: number;
  cost_usd: number;
}

interface ClassificationSplit {
  fleet: number;
  operator: number;
}

function formatCurrency(amount: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 4,
  }).format(amount);
}

function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

// Fallback pricing for client-side estimation when API data is unavailable
const FALLBACK_PRICING: Record<string, { input: number; output: number }> = {
  'claude-opus': { input: 15 / 1000000, output: 75 / 1000000 },
  'claude-sonnet': { input: 3 / 1000000, output: 15 / 1000000 },
  'claude-haiku': { input: 0.25 / 1000000, output: 1.25 / 1000000 },
  'gpt-4': { input: 30 / 1000000, output: 60 / 1000000 },
  'gpt-3.5-turbo': { input: 0.5 / 1000000, output: 1.5 / 1000000 },
};

function rlLevel(utilization: number): 'low' | 'medium' | 'high' | 'critical' {
  if (utilization >= 90) return 'critical';
  if (utilization >= 80) return 'high';
  if (utilization >= 60) return 'medium';
  return 'low';
}

function rlColor(level: string): string {
  switch (level) {
    case 'critical': return '#ea4335';
    case 'high': return '#fbbc04';
    case 'medium': return '#f9ab00';
    default: return '#34a853';
  }
}

function rlResetsIn(iso?: string | null): string {
  if (!iso) return '';
  const diffMin = (new Date(iso).getTime() - Date.now()) / 60000;
  if (diffMin <= 0) return '';
  if (diffMin < 60) return `resets in ~${Math.floor(diffMin)}m`;
  if (diffMin < 1440) return `resets in ~${Math.floor(diffMin / 60)}h`;
  return `resets in ~${Math.floor(diffMin / 1440)}d`;
}

function RateLimitWindowRow({ account }: { account: AccountCapacity }) {
  const windows = [
    { label: '5h', utilization: account.utilization_5h, tokens: account.tokens_5h, resetsAt: account.resets_at_5h },
    { label: '7d', utilization: account.utilization_7d, tokens: account.tokens_7d, resetsAt: account.resets_at_7d },
  ] as const;

  return (
    <div className="rl-row">
      <div className="rl-row-account">
        {account.account_id}
        <span className="rl-row-plan"> · {account.plan_type}</span>
      </div>
      <div className="rl-row-meters">
        {windows.map(w => {
          const pct = Math.min(w.utilization, 100);
          const level = rlLevel(w.utilization);
          const nearLimit = w.utilization >= 80;
          const resetText = rlResetsIn(w.resetsAt);
          return (
            <div key={w.label} className="rl-meter">
              <div className="rl-meter-label">
                <span className="rl-meter-window">{w.label}</span>
                <span className={`rl-meter-pct rl-meter-pct--${level}`}>
                  {w.utilization.toFixed(0)}%
                  {nearLimit && <span className="rl-near-limit-badge">!</span>}
                </span>
              </div>
              <div className="rl-bar">
                <div
                  className={`rl-bar-fill${nearLimit ? ' rl-bar-fill--near-limit' : ''}`}
                  style={{ width: `${pct}%`, background: rlColor(level) }}
                  title={`${formatNumber(w.tokens)} weighted tokens`}
                />
                <div className="rl-bar-zone" />
                <div className="rl-bar-threshold" />
              </div>
              {resetText && <div className="rl-reset-text">{resetText}</div>}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export default function CostPanel({ projectName, conversations: conversationsProp }: CostPanelProps) {
  const globalConversations = useAtomValue(conversationsAtom);
  const conversations = conversationsProp ?? globalConversations;
  const allCapacity = useAtomValue(capacityAtom);
  const claudeCapacity = allCapacity.filter(a => a.adapter === 'claude');
  const [apiBuckets, setApiBuckets] = useState<CostBucket[] | null>(null);
  const [loading, setLoading] = useState(true);

  // Fetch real cost data from backend
  useEffect(() => {
    let cancelled = false;
    setLoading(true);

    fetch(`/api/cost/buckets/${encodeURIComponent(projectName)}`)
      .then(res => res.ok ? res.json() : Promise.reject(new Error(`${res.status}`)))
      .then((data: CostBucket[]) => {
        if (!cancelled) setApiBuckets(data);
      })
      .catch(() => {
        // Fall back to client-side estimation
        if (!cancelled) setApiBuckets(null);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => { cancelled = true; };
  }, [projectName]);

  // Cost breakdown by adapter/model
  const costByAdapter = useMemo((): CostBreakdown[] => {
    if (apiBuckets && apiBuckets.length > 0) {
      // Use backend data
      const breakdown = new Map<string, CostBreakdown>();
      for (const bucket of apiBuckets) {
        const key = `${bucket.adapter}:${bucket.model}`;
        const existing = breakdown.get(key) || {
          adapter: bucket.adapter,
          model: bucket.model || null,
          totalTokens: 0,
          inputTokens: 0,
          outputTokens: 0,
          cacheReadTokens: 0,
          cacheWriteTokens: 0,
          totalCost: 0,
          requestCount: 0,
        };
        existing.totalTokens += bucket.usage.input_tokens + bucket.usage.output_tokens;
        existing.inputTokens += bucket.usage.input_tokens;
        existing.outputTokens += bucket.usage.output_tokens;
        existing.cacheReadTokens += bucket.usage.cache_read_tokens;
        existing.cacheWriteTokens += bucket.usage.cache_write_tokens;
        existing.totalCost += bucket.cost_usd;
        existing.requestCount += bucket.request_count;
        breakdown.set(key, existing);
      }
      return Array.from(breakdown.values()).sort((a, b) => b.totalCost - a.totalCost);
    }

    // Client-side fallback
    const breakdown = new Map<string, CostBreakdown>();
    conversations.forEach(conv => {
      const key = `${conv.provider}:${conv.worker_metadata?.worker || 'default'}`;
      const existing = breakdown.get(key) || {
        adapter: conv.provider,
        model: conv.worker_metadata?.worker || null,
        totalTokens: 0,
        inputTokens: 0,
        outputTokens: 0,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
        totalCost: 0,
        requestCount: 0,
      };

      existing.totalTokens += conv.total_tokens;
      existing.requestCount += 1;

      const inputSplit = 0.7;
      existing.inputTokens += Math.floor(conv.total_tokens * inputSplit);
      existing.outputTokens += Math.ceil(conv.total_tokens * (1 - inputSplit));

      const pricingKey = conv.provider.toLowerCase();
      const pricing = FALLBACK_PRICING[pricingKey] || FALLBACK_PRICING['gpt-3.5-turbo'];
      existing.totalCost += (existing.inputTokens * pricing.input) + (existing.outputTokens * pricing.output);

      breakdown.set(key, existing);
    });
    return Array.from(breakdown.values()).sort((a, b) => b.totalCost - a.totalCost);
  }, [apiBuckets, conversations]);

  // Time range costs
  const timeRangeCosts = useMemo((): TimeRangeCost[] => {
    const now = new Date();
    const ranges: TimeRangeCost[] = [
      { label: 'Today', tokens: 0, cost: 0 },
      { label: 'This Week', tokens: 0, cost: 0 },
      { label: 'This Month', tokens: 0, cost: 0 },
    ];

    if (apiBuckets && apiBuckets.length > 0) {
      for (const bucket of apiBuckets) {
        const bucketDate = new Date(bucket.date);
        const tokens = bucket.usage.input_tokens + bucket.usage.output_tokens;
        const cost = bucket.cost_usd;

        if (bucketDate.toDateString() === now.toDateString()) {
          ranges[0].tokens += tokens;
          ranges[0].cost += cost;
        }

        const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
        if (bucketDate > weekAgo) {
          ranges[1].tokens += tokens;
          ranges[1].cost += cost;
        }

        const monthStart = new Date(now.getFullYear(), now.getMonth(), 1);
        if (bucketDate >= monthStart) {
          ranges[2].tokens += tokens;
          ranges[2].cost += cost;
        }
      }
      return ranges;
    }

    // Client-side fallback
    conversations.forEach(conv => {
      const convDate = new Date(conv.created_at);
      const tokens = conv.total_tokens;
      const pricingKey = conv.provider.toLowerCase();
      const pricing = FALLBACK_PRICING[pricingKey] || FALLBACK_PRICING['gpt-3.5-turbo'];
      const inputTokens = Math.floor(tokens * 0.7);
      const outputTokens = Math.ceil(tokens * 0.3);
      const cost = (inputTokens * pricing.input) + (outputTokens * pricing.output);

      if (convDate.toDateString() === now.toDateString()) {
        ranges[0].tokens += tokens;
        ranges[0].cost += cost;
      }

      const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
      if (convDate > weekAgo) {
        ranges[1].tokens += tokens;
        ranges[1].cost += cost;
      }

      const monthStart = new Date(now.getFullYear(), now.getMonth(), 1);
      if (convDate > monthStart) {
        ranges[2].tokens += tokens;
        ranges[2].cost += cost;
      }
    });
    return ranges;
  }, [apiBuckets, conversations]);

  // Fleet vs operator cost split derived from backend classification field
  const classificationSplit = useMemo((): ClassificationSplit => {
    if (!apiBuckets || apiBuckets.length === 0) {
      // Client-side fallback: classify by conversation kind
      const fleet = conversations
        .filter(c => c.kind === 'worker')
        .reduce((sum, c) => {
          const pricingKey = c.provider.toLowerCase();
          const pricing = FALLBACK_PRICING[pricingKey] || FALLBACK_PRICING['gpt-3.5-turbo'];
          const input = Math.floor(c.total_tokens * 0.7);
          const output = Math.ceil(c.total_tokens * 0.3);
          return sum + input * pricing.input + output * pricing.output;
        }, 0);
      const operator = conversations
        .filter(c => c.kind !== 'worker')
        .reduce((sum, c) => {
          const pricingKey = c.provider.toLowerCase();
          const pricing = FALLBACK_PRICING[pricingKey] || FALLBACK_PRICING['gpt-3.5-turbo'];
          const input = Math.floor(c.total_tokens * 0.7);
          const output = Math.ceil(c.total_tokens * 0.3);
          return sum + input * pricing.input + output * pricing.output;
        }, 0);
      return { fleet, operator };
    }
    return apiBuckets.reduce(
      (acc, b) => {
        if (b.classification === 'fleet') acc.fleet += b.cost_usd;
        else acc.operator += b.cost_usd;
        return acc;
      },
      { fleet: 0, operator: 0 } as ClassificationSplit,
    );
  }, [apiBuckets, conversations]);

  const totalCost = costByAdapter.reduce((sum, b) => sum + b.totalCost, 0);
  const totalTokens = costByAdapter.reduce((sum, b) => sum + b.totalTokens, 0);
  const dataSource = apiBuckets ? 'server' : 'estimated';

  return (
    <div className="cost-panel">
      <div className="cost-header">
        <h3>Cost Analysis</h3>
        <div className="cost-summary">
          <div className="cost-summary-item">
            <span className="cost-label">Total Spend (Week)</span>
            <span className="cost-value">{formatCurrency(timeRangeCosts[1].cost)}</span>
          </div>
          <div className="cost-summary-item">
            <span className="cost-label">Total Tokens</span>
            <span className="cost-value">{formatNumber(totalTokens)}</span>
          </div>
        </div>
      </div>

      <div className="cost-sections">
        {/* Time Range Costs */}
        <section className="cost-section">
          <h4>Spend by Time Period</h4>
          <div className="cost-time-ranges">
            {timeRangeCosts.map(range => (
              <div key={range.label} className="cost-time-range">
                <span className="range-label">{range.label}</span>
                <div className="range-values">
                  <span className="range-cost">{formatCurrency(range.cost)}</span>
                  <span className="range-tokens">{formatNumber(range.tokens)} tokens</span>
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Claude Rate Limit Windows */}
        {claudeCapacity.length > 0 && (
          <section className="cost-section">
            <h4>Claude Rate Limit Windows</h4>
            <div className="rl-list">
              {claudeCapacity.map(account => (
                <RateLimitWindowRow key={account.account_id} account={account} />
              ))}
            </div>
          </section>
        )}

        {/* Adapter Breakdown */}
        <section className="cost-section">
          <h4>Cost by Adapter</h4>
          {loading ? (
            <div className="cost-loading">Loading cost data...</div>
          ) : costByAdapter.length === 0 ? (
            <p className="cost-empty">No cost data available yet</p>
          ) : (
            <div className="cost-breakdown">
              {costByAdapter.map(breakdown => {
                const percentage = totalCost > 0 ? (breakdown.totalCost / totalCost) * 100 : 0;
                return (
                  <div key={`${breakdown.adapter}-${breakdown.model}`} className="cost-breakdown-item">
                    <div className="cost-breakdown-header">
                      <span className="adapter-name">
                        {breakdown.adapter}
                        {breakdown.model && <span className="adapter-model"> · {breakdown.model}</span>}
                      </span>
                      <span className="adapter-cost">{formatCurrency(breakdown.totalCost)}</span>
                    </div>
                    <div className="cost-breakdown-details">
                      <div className="cost-bar">
                        <div
                          className="cost-bar-fill"
                          style={{ width: `${percentage}%` }}
                        />
                      </div>
                      <div className="cost-breakdown-stats">
                        <span>{formatNumber(breakdown.totalTokens)} tokens</span>
                        <span>{breakdown.requestCount} requests</span>
                        <span>{percentage.toFixed(1)}% of total</span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </section>

        {/* Fleet vs Operator Split */}
        <section className="cost-section">
          <h4>Fleet vs Ad-hoc Cost</h4>
          <div className="cost-classification-split">
            {(['fleet', 'operator'] as const).map(cls => {
              const cost = classificationSplit[cls];
              const splitTotal = classificationSplit.fleet + classificationSplit.operator;
              const pct = splitTotal > 0 ? (cost / splitTotal) * 100 : 0;
              return (
                <div key={cls} className="cost-classification-item">
                  <div className="cost-classification-header">
                    <span className={`badge badge-${cls === 'fleet' ? 'fleet' : 'operator'} badge-sm`}>
                      {cls === 'fleet' ? 'Fleet (NEEDLE workers)' : 'Ad-hoc / Operator'}
                    </span>
                    <span className="adapter-cost">{formatCurrency(cost)}</span>
                  </div>
                  <div className="cost-breakdown-details">
                    <div className="cost-bar">
                      <div
                        className={`cost-bar-fill cost-bar-${cls}`}
                        style={{ width: `${pct}%` }}
                      />
                    </div>
                    <div className="cost-breakdown-stats">
                      <span>{pct.toFixed(1)}% of total</span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </section>

        {/* Notes */}
        <section className="cost-section">
          <h4>Cost Analysis Notes</h4>
          <div className="cost-notes">
            {dataSource === 'estimated' ? (
              <p className="cost-note">
                <strong>Note:</strong> Cost figures are estimates based on token counts and standard pricing.
                Actual costs may vary based on provider-specific pricing, caching, and promotional credits.
              </p>
            ) : (
              <p className="cost-note">
                <strong>Source:</strong> Server-side cost data from backend pricing configuration.
              </p>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
