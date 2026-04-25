import { useState, useEffect, useCallback, useMemo } from 'react';
import { useAtomValue, useSetAtom } from 'jotai';
import { beadsAtom, conversationsAtom, selectedConversationIdAtom, workersAtom } from './atoms';

interface TimelineSegment {
  start: string;
  end: string | null;
  bead_id: string;
  outcome: string;
}

interface WorkerTimelineEntry {
  worker: string;
  segments: TimelineSegment[];
  heartbeats: string[];
  liveness: string;
}

interface TimelineResponse {
  window_start: string;
  window_end: string;
  workers: WorkerTimelineEntry[];
}

interface TooltipData {
  beadTitle: string;
  bead_id: string;
  outcome: string;
  duration: string;
  startLabel: string;
  endLabel: string | null;
  x: number;
  y: number;
}

const HOURS_OPTIONS = [1, 4, 8, 24, 168] as const;
const HOURS_LABELS: Record<number, string> = { 1: '1h', 4: '4h', 8: '8h', 24: '24h', 168: '7d' };

const SEGMENT_COLORS: Record<string, string> = {
  active: '#34a853',
  closed: '#1976d2',
  released: '#f9a825',
  knot: '#ea4335',
  unknown: '#9aa0a6',
};

function formatDuration(startIso: string, endIso: string | null, windowEndIso: string): string {
  const start = new Date(startIso).getTime();
  const end = new Date(endIso ?? windowEndIso).getTime();
  const sec = Math.max(0, Math.floor((end - start) / 1000));
  if (sec < 60) return `${sec}s`;
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min}m`;
  const hr = Math.floor(min / 60);
  const rem = min % 60;
  return rem ? `${hr}h ${rem}m` : `${hr}h`;
}

function formatShortTime(iso: string, multiDay: boolean): string {
  const d = new Date(iso);
  if (multiDay) {
    return d.toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  }
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function livenessClass(liveness: string): string {
  switch (liveness) {
    case 'live': return 'tl-liveness-live';
    case 'hung': return 'tl-liveness-hung';
    case 'dead': return 'tl-liveness-dead';
    default: return 'tl-liveness-unknown';
  }
}

function buildAxisTicks(windowStart: number, windowEnd: number): { pct: number; label: string }[] {
  const range = windowEnd - windowStart;
  const MIN = 60_000;
  const HR = 3_600_000;
  const DAY = 86_400_000;

  let intervalMs: number;
  if (range <= HR) intervalMs = 5 * MIN;
  else if (range <= 4 * HR) intervalMs = 30 * MIN;
  else if (range <= 8 * HR) intervalMs = HR;
  else if (range <= DAY) intervalMs = 4 * HR;
  else intervalMs = DAY;

  const ticks: { pct: number; label: string }[] = [];
  const first = Math.ceil(windowStart / intervalMs) * intervalMs;
  const multiDay = range > DAY;

  for (let t = first; t < windowEnd; t += intervalMs) {
    const pct = ((t - windowStart) / range) * 100;
    const d = new Date(t);
    const label =
      intervalMs >= DAY
        ? d.toLocaleDateString([], { month: 'short', day: 'numeric' })
        : multiDay
        ? d.toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
        : d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    ticks.push({ pct, label });
  }
  return ticks;
}

export default function WorkerTimeline() {
  const [hours, setHours] = useState<number>(24);
  const [data, setData] = useState<TimelineResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [tooltip, setTooltip] = useState<TooltipData | null>(null);

  const beads = useAtomValue(beadsAtom);
  const conversations = useAtomValue(conversationsAtom);
  const workers = useAtomValue(workersAtom);
  const setSelectedConvId = useSetAtom(selectedConversationIdAtom);

  const beadTitles = useMemo(() => {
    const m: Record<string, string> = {};
    for (const b of beads) m[b.id] = b.title;
    return m;
  }, [beads]);

  // Stable key that changes only when a worker changes state — triggers re-fetch
  const workerStateKey = useMemo(
    () =>
      workers
        .map(w => `${w.worker}:${w.state.state}`)
        .sort()
        .join('|'),
    [workers],
  );

  const fetchData = useCallback(async () => {
    try {
      const res = await fetch(`/api/workers/timeline?hours=${hours}`);
      if (res.ok) setData(await res.json());
    } catch {
      /* keep stale data on network error */
    } finally {
      setLoading(false);
    }
  }, [hours]);

  // Re-fetch when hours changes or window mounts
  useEffect(() => {
    setLoading(true);
    fetchData();
    const id = setInterval(fetchData, 30_000);
    return () => clearInterval(id);
  }, [fetchData]);

  // Re-fetch when any worker changes state (live updates via WS)
  useEffect(() => {
    if (workerStateKey) fetchData();
  }, [workerStateKey, fetchData]);

  const handleSegmentClick = useCallback(
    (workerName: string, beadId: string) => {
      const conv =
        conversations.find(
          c =>
            c.kind === 'worker' &&
            c.worker_metadata?.worker === workerName &&
            c.worker_metadata?.bead === beadId,
        ) ??
        conversations.find(c => c.kind === 'worker' && c.worker_metadata?.worker === workerName);
      if (conv) setSelectedConvId(conv.id);
      window.location.hash = '#/fleet';
    },
    [conversations, setSelectedConvId],
  );

  if (loading && !data) {
    return (
      <section className="fleet-section timeline-section">
        <div className="timeline-header">
          <h2>Worker Timeline</h2>
        </div>
        <div className="fleet-loading">
          <div className="fleet-loading-spinner" />
          <span>Loading timeline…</span>
        </div>
      </section>
    );
  }

  const workerRows = data?.workers ?? [];
  const windowStart = data ? new Date(data.window_start).getTime() : Date.now() - hours * 3_600_000;
  const windowEnd = data ? new Date(data.window_end).getTime() : Date.now();
  const range = windowEnd - windowStart;
  const multiDay = hours >= 24;
  const ticks = buildAxisTicks(windowStart, windowEnd);

  return (
    <section className="fleet-section timeline-section">
      <div className="timeline-header">
        <h2>Worker Timeline</h2>
        <div className="timeline-window-picker">
          {HOURS_OPTIONS.map(h => (
            <button
              key={h}
              className={`timeline-window-btn${hours === h ? ' active' : ''}`}
              onClick={() => { setHours(h); setLoading(true); }}
            >
              {HOURS_LABELS[h]}
            </button>
          ))}
        </div>
      </div>

      {workerRows.length === 0 ? (
        <div className="fleet-empty">
          No worker activity in the last {HOURS_LABELS[hours]}
        </div>
      ) : (
        <div className="timeline-container">
          {workerRows.map(wt => (
            <div key={wt.worker} className="timeline-row">
              <div className="timeline-worker-label">
                <span className={`tl-liveness-dot ${livenessClass(wt.liveness)}`} />
                <span className="timeline-worker-name">{wt.worker}</span>
              </div>

              <div className="timeline-track">
                {/* Execution segments */}
                {wt.segments.map((seg, i) => {
                  const segStart = new Date(seg.start).getTime();
                  const segEnd = seg.end ? new Date(seg.end).getTime() : windowEnd;
                  const left = Math.max(0, ((segStart - windowStart) / range) * 100);
                  const width = Math.max(
                    0.3,
                    ((Math.min(segEnd, windowEnd) - Math.max(segStart, windowStart)) / range) * 100,
                  );
                  const color = SEGMENT_COLORS[seg.outcome] ?? '#9aa0a6';
                  const beadTitle = beadTitles[seg.bead_id] ?? seg.bead_id;

                  return (
                    <div
                      key={i}
                      className={`timeline-segment${seg.outcome === 'active' ? ' tl-seg-active' : ''}`}
                      style={{ left: `${left}%`, width: `${width}%`, background: color }}
                      onMouseEnter={e =>
                        setTooltip({
                          beadTitle,
                          bead_id: seg.bead_id,
                          outcome: seg.outcome,
                          duration: formatDuration(seg.start, seg.end, data!.window_end),
                          startLabel: formatShortTime(seg.start, multiDay),
                          endLabel: seg.end ? formatShortTime(seg.end, multiDay) : null,
                          x: e.clientX,
                          y: e.clientY,
                        })
                      }
                      onMouseMove={e =>
                        setTooltip(prev => (prev ? { ...prev, x: e.clientX, y: e.clientY } : null))
                      }
                      onMouseLeave={() => setTooltip(null)}
                      onClick={() => handleSegmentClick(wt.worker, seg.bead_id)}
                      role="button"
                      tabIndex={0}
                      onKeyDown={e =>
                        e.key === 'Enter' && handleSegmentClick(wt.worker, seg.bead_id)
                      }
                      aria-label={`${beadTitle} — ${formatDuration(seg.start, seg.end, data!.window_end)} — ${seg.outcome}`}
                    />
                  );
                })}

                {/* Heartbeat tick marks */}
                {wt.heartbeats.map((hb, i) => {
                  const t = new Date(hb).getTime();
                  const pct = ((t - windowStart) / range) * 100;
                  if (pct < 0 || pct > 100) return null;
                  return (
                    <div
                      key={`hb-${i}`}
                      className="timeline-heartbeat"
                      style={{ left: `${pct}%` }}
                    />
                  );
                })}
              </div>
            </div>
          ))}

          {/* Time axis */}
          <div className="timeline-axis-row">
            <div className="timeline-axis-spacer" />
            <div className="timeline-axis">
              {ticks.map((tick, i) => (
                <div key={i} className="timeline-axis-tick" style={{ left: `${tick.pct}%` }}>
                  <div className="timeline-axis-line" />
                  <span className="timeline-axis-label">{tick.label}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Fixed-position tooltip — not clipped by section overflow */}
      {tooltip && (
        <div
          className="timeline-tooltip"
          style={{ left: tooltip.x + 14, top: tooltip.y - 8 }}
        >
          <div className="tt-title">{tooltip.beadTitle}</div>
          <div className="tt-row">
            <span className="tt-key">Duration</span>
            <span className="tt-val">{tooltip.duration}</span>
          </div>
          <div className="tt-row">
            <span className="tt-key">Outcome</span>
            <span className={`tt-val tt-outcome tt-outcome-${tooltip.outcome}`}>{tooltip.outcome}</span>
          </div>
          <div className="tt-row">
            <span className="tt-key">Start</span>
            <span className="tt-val">{tooltip.startLabel}</span>
          </div>
          {tooltip.endLabel && (
            <div className="tt-row">
              <span className="tt-key">End</span>
              <span className="tt-val">{tooltip.endLabel}</span>
            </div>
          )}
          <div className="tt-hint">Click to open transcript</div>
        </div>
      )}
    </section>
  );
}
