import { useAtomValue } from 'jotai';
import { useMemo } from 'react';
import { conversationsAtom } from './atoms';

interface CostPanelProps {
  projectName: string;
}

interface CostBreakdown {
  adapter: string;
  model: string | null;
  totalTokens: number;
  inputTokens: number;
  outputTokens: number;
  totalCost: number;
  requestCount: number;
}

interface TimeRangeCost {
  label: string;
  tokens: number;
  cost: number;
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

// Approximate pricing (will be updated with real rates)
const PRICING: Record<string, { input: number; output: number }> = {
  'claude-opus': { input: 15 / 1000000, output: 75 / 1000000 },
  'claude-sonnet': { input: 3 / 1000000, output: 15 / 1000000 },
  'claude-haiku': { input: 0.25 / 1000000, output: 1.25 / 1000000 },
  'gpt-4': { input: 30 / 1000000, output: 60 / 1000000 },
  'gpt-3.5-turbo': { input: 0.5 / 1000000, output: 1.5 / 1000000 },
};

export default function CostPanel({ projectName: _projectName }: CostPanelProps) {
  const conversations = useAtomValue(conversationsAtom);

  // Calculate cost breakdown by adapter/model
  const costByAdapter = useMemo(() => {
    const breakdown = new Map<string, CostBreakdown>();

    conversations.forEach(conv => {
      const key = `${conv.provider}${conv.worker_metadata?.worker ? `-${conv.worker_metadata.worker}` : ''}`;
      const existing = breakdown.get(key) || {
        adapter: conv.provider,
        model: conv.worker_metadata?.worker || null,
        totalTokens: 0,
        inputTokens: 0,
        outputTokens: 0,
        totalCost: 0,
        requestCount: 0,
      };

      existing.totalTokens += conv.total_tokens;
      existing.requestCount += 1;

      // Estimate input/output split (rough approximation)
      const inputSplit = 0.7;
      existing.inputTokens += Math.floor(conv.total_tokens * inputSplit);
      existing.outputTokens += Math.ceil(conv.total_tokens * (1 - inputSplit));

      // Calculate cost using approximate pricing
      const pricingKey = conv.provider.toLowerCase();
      const pricing = PRICING[pricingKey] || PRICING['gpt-3.5-turbo'];
      existing.totalCost += (existing.inputTokens * pricing.input) + (existing.outputTokens * pricing.output);

      breakdown.set(key, existing);
    });

    return Array.from(breakdown.values()).sort((a, b) => b.totalCost - a.totalCost);
  }, [conversations]);

  // Calculate time range costs
  const timeRangeCosts = useMemo(() => {
    const now = new Date();
    const ranges: TimeRangeCost[] = [
      { label: 'Today', tokens: 0, cost: 0 },
      { label: 'This Week', tokens: 0, cost: 0 },
      { label: 'This Month', tokens: 0, cost: 0 },
    ];

    conversations.forEach(conv => {
      const convDate = new Date(conv.created_at);
      const tokens = conv.total_tokens;

      // Estimate cost
      const pricingKey = conv.provider.toLowerCase();
      const pricing = PRICING[pricingKey] || PRICING['gpt-3.5-turbo'];
      const inputTokens = Math.floor(tokens * 0.7);
      const outputTokens = Math.ceil(tokens * 0.3);
      const cost = (inputTokens * pricing.input) + (outputTokens * pricing.output);

      // Today
      if (convDate.toDateString() === now.toDateString()) {
        ranges[0].tokens += tokens;
        ranges[0].cost += cost;
      }

      // This week
      const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
      if (convDate > weekAgo) {
        ranges[1].tokens += tokens;
        ranges[1].cost += cost;
      }

      // This month
      const monthAgo = new Date(now.getFullYear(), now.getMonth(), 1);
      if (convDate > monthAgo) {
        ranges[2].tokens += tokens;
        ranges[2].cost += cost;
      }
    });

    return ranges;
  }, [conversations]);

  const totalCost = costByAdapter.reduce((sum, b) => sum + b.totalCost, 0);
  const totalTokens = costByAdapter.reduce((sum, b) => sum + b.totalTokens, 0);

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

        {/* Adapter Breakdown */}
        <section className="cost-section">
          <h4>Cost by Adapter</h4>
          {costByAdapter.length === 0 ? (
            <p className="cost-empty">No cost data available yet</p>
          ) : (
            <div className="cost-breakdown">
              {costByAdapter.map(breakdown => {
                const percentage = totalCost > 0 ? (breakdown.totalCost / totalCost) * 100 : 0;
                return (
                  <div key={breakdown.adapter} className="cost-breakdown-item">
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

        {/* Cost per Bead */}
        <section className="cost-section">
          <h4>Cost Analysis Notes</h4>
          <div className="cost-notes">
            <p className="cost-note">
              <strong>Note:</strong> Cost figures are estimates based on token counts and standard pricing.
              Actual costs may vary based on provider-specific pricing, caching, and promotional credits.
            </p>
            <p className="cost-note">
              Rate limit windows (5h/7d for Claude) are shown in the Capacity panel.
            </p>
          </div>
        </section>
      </div>
    </div>
  );
}
