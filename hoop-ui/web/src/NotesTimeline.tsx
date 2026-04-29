import { useState, useEffect, useCallback, useMemo } from 'react';
import { useAtomValue } from 'jotai';
import { dictatedNotesAtom } from './atoms';

export interface NoteSummary {
  stitch_id: string;
  project: string;
  title: string;
  kind: string;
  recorded_at: string;
  transcribed_at: string;
  duration_secs: number | null;
  language: string | null;
  tags: string[];
  transcript_preview: string;
  transcript: string;
  last_activity_at: string;
  created_at: string;
  audio_filename: string;
  transcription_status: 'Pending' | 'Completed' | 'Failed';
}

interface TimelineEntry {
  note: NoteSummary;
  startPct: number;
  widthPct: number;
}

const DURATION_OPTIONS = [1, 4, 8, 24, 168] as const;
const DURATION_LABELS: Record<number, string> = { 1: '1h', 4: '4h', 8: '8h', 24: '24h', 168: '7d' };

const STATUS_COLORS: Record<string, string> = {
  Pending: '#f9ab00',
  Completed: '#34a853',
  Failed: '#ea4335',
};

function formatDuration(secs: number | null): string {
  if (secs === null) return '-';
  const s = Math.floor(secs);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m`;
  const hr = Math.floor(m / 60);
  const rem = m % 60;
  return rem ? `${hr}h ${rem}m` : `${hr}h`;
}

function formatShortTime(iso: string, multiDay: boolean): string {
  const d = new Date(iso);
  if (multiDay) {
    return d.toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  }
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
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

interface NotesTimelineProps {
  projectName: string;
  onNoteClick?: (stitchId: string) => void;
}

export function NotesTimeline({ projectName, onNoteClick }: NotesTimelineProps) {
  const [hours, setHours] = useState<number>(24);
  const [notes, setNotes] = useState<NoteSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [tooltip, setTooltip] = useState<{
    title: string;
    transcript: string;
    status: string;
    duration: string;
    startLabel: string;
    x: number;
    y: number;
  } | null>(null);

  const allNotes = useAtomValue(dictatedNotesAtom);

  const fetchNotes = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const res = await fetch(`/api/p/${encodeURIComponent(projectName)}/dictated-notes`);
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }
      const data: NoteSummary[] = await res.json();
      setNotes(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load notes');
      setNotes([]);
    } finally {
      setLoading(false);
    }
  }, [projectName]);

  useEffect(() => {
    fetchNotes();
  }, [fetchNotes]);

  const windowStart = Date.now() - hours * 3_600_000;
  const windowEnd = Date.now();
  const range = windowEnd - windowStart;
  const multiDay = hours >= 24;
  const ticks = buildAxisTicks(windowStart, windowEnd);

  const timelineEntries: TimelineEntry[] = useMemo(() => {
    return notes
      .filter(note => {
        const noteTime = new Date(note.recorded_at).getTime();
        return noteTime >= windowStart && noteTime <= windowEnd;
      })
      .map(note => {
        const noteTime = new Date(note.recorded_at).getTime();
        const startPct = ((noteTime - windowStart) / range) * 100;
        // Minimum 3px width for visibility
        const widthPct = Math.max(0.3, (3 * 60_000 / range) * 100); // Assume min 3 min duration
        return { note, startPct, widthPct };
      })
      .sort((a, b) => a.note.recorded_at.localeCompare(b.note.recorded_at));
  }, [notes, windowStart, range]);

  const handleNoteClick = useCallback(
    (stitchId: string) => {
      onNoteClick?.(stitchId);
    },
    [onNoteClick],
  );

  if (loading && notes.length === 0) {
    return (
      <section className="notes-timeline-section">
        <div className="timeline-header">
          <h2>Voice Notes</h2>
        </div>
        <div className="fleet-loading">
          <div className="fleet-loading-spinner" />
          <span>Loading notes…</span>
        </div>
      </section>
    );
  }

  if (error) {
    return (
      <section className="notes-timeline-section">
        <div className="timeline-header">
          <h2>Voice Notes</h2>
        </div>
        <div className="fleet-error">
          <span>{error}</span>
        </div>
      </section>
    );
  }

  return (
    <section className="notes-timeline-section">
      <div className="timeline-header">
        <h2>Voice Notes</h2>
        <div className="timeline-window-picker">
          {DURATION_OPTIONS.map(h => (
            <button
              key={h}
              className={`timeline-window-btn${hours === h ? ' active' : ''}`}
              onClick={() => { setHours(h); setLoading(true); }}
            >
              {DURATION_LABELS[h]}
            </button>
          ))}
        </div>
      </div>

      {timelineEntries.length === 0 ? (
        <div className="fleet-empty">
          No voice notes in the last {DURATION_LABELS[hours]}
          <div className="fleet-empty-hint">
            Press <kbd>⌘ ⇧ D</kbd> to start dictating
          </div>
        </div>
      ) : (
        <div className="timeline-container">
          <div className="timeline-row">
            <div className="timeline-track">
              {timelineEntries.map(({ note, startPct, widthPct }) => {
                const color = STATUS_COLORS[note.transcription_status] ?? '#9aa0a6';
                const statusIcon = note.transcription_status === 'Completed' ? '✓' :
                                  note.transcription_status === 'Pending' ? '⋯' : '✕';

                return (
                  <div
                    key={note.stitch_id}
                    className="timeline-segment timeline-segment--note"
                    style={{ left: `${startPct}%`, width: `${widthPct}%`, background: color }}
                    onMouseEnter={e =>
                      setTooltip({
                        title: note.title,
                        transcript: note.transcript_preview,
                        status: note.transcription_status,
                        duration: formatDuration(note.duration_secs),
                        startLabel: formatShortTime(note.recorded_at, multiDay),
                        x: e.clientX,
                        y: e.clientY,
                      })
                    }
                    onMouseMove={e =>
                      setTooltip(prev => (prev ? { ...prev, x: e.clientX, y: e.clientY } : null))
                    }
                    onMouseLeave={() => setTooltip(null)}
                    onClick={() => handleNoteClick(note.stitch_id)}
                    role="button"
                    tabIndex={0}
                    onKeyDown={e =>
                      e.key === 'Enter' && handleNoteClick(note.stitch_id)
                    }
                    aria-label={`${note.title} — ${formatDuration(note.duration_secs)} — ${note.transcription_status}`}
                  >
                    <span className="timeline-segment-icon" aria-hidden="true">
                      {statusIcon}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>

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

      {/* Fixed-position tooltip */}
      {tooltip && (
        <div
          className="timeline-tooltip"
          style={{ left: tooltip.x + 14, top: tooltip.y - 8 }}
        >
          <div className="tt-title">{tooltip.title}</div>
          <div className="tt-row">
            <span className="tt-key">Duration</span>
            <span className="tt-val">{tooltip.duration}</span>
          </div>
          <div className="tt-row">
            <span className="tt-key">Status</span>
            <span className={`tt-val tt-outcome tt-outcome-${tooltip.status.toLowerCase()}`}>
              {tooltip.status}
            </span>
          </div>
          <div className="tt-row">
            <span className="tt-key">Recorded</span>
            <span className="tt-val">{tooltip.startLabel}</span>
          </div>
          {tooltip.transcript && (
            <div className="tt-transcript">
              {tooltip.transcript}
            </div>
          )}
          <div className="tt-hint">Click to view note</div>
        </div>
      )}
    </section>
  );
}
