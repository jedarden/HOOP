import { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { useAtomValue } from 'jotai';
import { projectCardsAtom, beadsAtom, BeadData } from './atoms';
import { UploadManager, formatBytes } from './components/UploadManager';

export type BeadKind = 'task' | 'genesis' | 'review' | 'fix' | 'bug' | 'epic';
export type StitchKind = 'investigation' | 'fix' | 'feature';

interface BeadSummary {
  id: string;
  title: string;
  issue_type: string;
  priority: number;
  dependencies: string[];
}

// Preview types
interface StitchPreview {
  schema_version: string;
  prediction: PredictionData | null;
  risk_patterns: RiskPatternMatch[];
  file_conflicts: FileConflict[];
  similar_stitches: SimilarStitchRef[];
}

// Dedup match from the semantic pre-dedup check
interface DedupMatchRef {
  id: string;
  project: string;
  title: string;
  kind: string;
  similarity: number;
}

interface PredictionData {
  cost: PercentileEstimate;
  duration: PercentileEstimate;
  likely_adapter_model: string | null;
  similar_count: number;
  data_range: DateRange;
}

interface PercentileEstimate {
  p50: number;
  p90: number;
  count: number;
}

interface DateRange {
  start: string;
  end: string;
}

interface RiskPatternMatch {
  pattern: RiskPatternInfo;
  confidence: number;
  matched_keywords: string[];
  matched_labels: string[];
}

interface RiskPatternInfo {
  id: string;
  name: string;
  description: string;
  fix_recommendation: string;
  severity: string;
  category: string;
}

interface FileConflict {
  bead_id: string;
  title: string;
  project: string;
  overlapping_files: string[];
}

interface SimilarStitchRef {
  id: string;
  title: string;
  similarity: number;
}

interface FormState {
  title: string;
  description: string;
  kind: BeadKind;
  stitchKind: StitchKind;
  priority: string;
  assignee: string;
  labelInput: string;
  labels: string[];
  depSearch: string;
  dependencies: BeadSummary[];
  hasAcceptanceCriteria: boolean;
  /** If true, submit via stitch decomposition instead of single bead */
  stitchMode: boolean;
}

interface BeadDraftFormProps {
  projectName: string;
  onClose: () => void;
  onCreated: (beadId: string, stitchId?: string) => void;
}

// Simple markdown → HTML (no external lib, covers common cases)
function renderMarkdown(md: string): string {
  const escaped = md
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  const lines = escaped.split('\n');
  const out: string[] = [];
  let inUl = false;
  let inCode = false;

  for (const raw of lines) {
    if (raw.startsWith('```')) {
      if (inUl) { out.push('</ul>'); inUl = false; }
      if (inCode) { out.push('</code></pre>'); inCode = false; }
      else { out.push('<pre><code>'); inCode = true; }
      continue;
    }
    if (inCode) { out.push(raw + '\n'); continue; }

    // headings
    if (raw.startsWith('### ')) { if (inUl) { out.push('</ul>'); inUl = false; } out.push(`<h3>${inline(raw.slice(4))}</h3>`); continue; }
    if (raw.startsWith('## '))  { if (inUl) { out.push('</ul>'); inUl = false; } out.push(`<h2>${inline(raw.slice(3))}</h2>`); continue; }
    if (raw.startsWith('# '))   { if (inUl) { out.push('</ul>'); inUl = false; } out.push(`<h1>${inline(raw.slice(2))}</h1>`); continue; }

    // horizontal rule
    if (/^---+$/.test(raw.trim())) { if (inUl) { out.push('</ul>'); inUl = false; } out.push('<hr>'); continue; }

    // unordered list
    if (/^[-*] /.test(raw)) {
      if (!inUl) { out.push('<ul>'); inUl = true; }
      out.push(`<li>${inline(raw.slice(2))}</li>`);
      continue;
    }

    // blank line closes list / paragraph
    if (raw.trim() === '') {
      if (inUl) { out.push('</ul>'); inUl = false; }
      out.push('<br>');
      continue;
    }

    if (inUl) { out.push('</ul>'); inUl = false; }
    out.push(`<p>${inline(raw)}</p>`);
  }

  if (inUl) out.push('</ul>');
  if (inCode) out.push('</code></pre>');
  return out.join('');
}

function inline(s: string): string {
  return s
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/`(.+?)`/g, '<code>$1</code>')
    .replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2">$1</a>');
}

// Bead graph delta: new bead + its dependencies as a mini SVG
function BeadGraphDelta({ newTitle, deps }: { newTitle: string; deps: BeadSummary[] }) {
  const CIRCLE_R = 20;
  const COL_W = 160;
  const ROW_H = 52;
  const PADDING = 20;

  const depCount = Math.max(deps.length, 1);
  const totalH = depCount * ROW_H + PADDING * 2;
  const totalW = deps.length > 0 ? COL_W * 2 + PADDING * 2 + 60 : COL_W + PADDING * 2;

  const newX = deps.length > 0 ? PADDING + COL_W + 60 + COL_W / 2 : PADDING + COL_W / 2;
  const newY = totalH / 2;

  return (
    <svg
      className="bead-graph-delta"
      width={totalW}
      height={totalH}
      viewBox={`0 0 ${totalW} ${totalH}`}
      aria-label="Bead dependency graph preview"
    >
      <defs>
        <marker id="arrow" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
          <path d="M0,0 L0,6 L8,3 z" fill="#888" />
        </marker>
      </defs>

      {/* Dep nodes */}
      {deps.map((dep, i) => {
        const depX = PADDING + COL_W / 2;
        const depY = PADDING + i * ROW_H + ROW_H / 2;
        const edgeX2 = newX - CIRCLE_R - 4;
        const edgeY2 = newY;
        return (
          <g key={dep.id}>
            <line
              x1={depX + CIRCLE_R + 4}
              y1={depY}
              x2={edgeX2}
              y2={edgeY2}
              stroke="#888"
              strokeWidth="1.5"
              markerEnd="url(#arrow)"
            />
            <circle cx={depX} cy={depY} r={CIRCLE_R} fill="#f0f0f0" stroke="#aaa" strokeWidth="1.5" />
            <text x={depX} y={depY + 4} textAnchor="middle" fontSize="9" fill="#555">
              {dep.id.length > 12 ? dep.id.slice(0, 12) + '…' : dep.id}
            </text>
            <text x={depX} y={depY + 30} textAnchor="middle" fontSize="8" fill="#999">
              {dep.title.length > 18 ? dep.title.slice(0, 18) + '…' : dep.title}
            </text>
          </g>
        );
      })}

      {/* New bead node */}
      <circle cx={newX} cy={newY} r={CIRCLE_R + 4} fill="#e3f2fd" stroke="#1976d2" strokeWidth="2" />
      <text x={newX} y={newY + 4} textAnchor="middle" fontSize="9" fontWeight="bold" fill="#1976d2">
        NEW
      </text>
      <text x={newX} y={newY + 36} textAnchor="middle" fontSize="8" fill="#555">
        {newTitle.length > 20 ? newTitle.slice(0, 20) + '…' : newTitle}
      </text>
    </svg>
  );
}

// Multi-value label input
function LabelInput({ labels, onAdd, onRemove }: { labels: string[]; onAdd: (l: string) => void; onRemove: (l: string) => void }) {
  const [input, setInput] = useState('');

  const handleKey = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if ((e.key === 'Enter' || e.key === ',') && input.trim()) {
      e.preventDefault();
      onAdd(input.trim());
      setInput('');
    } else if (e.key === 'Backspace' && !input && labels.length > 0) {
      onRemove(labels[labels.length - 1]);
    }
  };

  return (
    <div className="label-input-wrapper">
      {labels.map(l => (
        <span key={l} className="label-chip">
          {l}
          <button type="button" onClick={() => onRemove(l)} className="label-chip-remove" aria-label={`Remove ${l}`}>×</button>
        </span>
      ))}
      <input
        type="text"
        value={input}
        onChange={e => setInput(e.target.value)}
        onKeyDown={handleKey}
        placeholder={labels.length === 0 ? 'Add labels (Enter or comma)' : ''}
        className="label-chip-input"
      />
    </div>
  );
}

interface AttachmentItem {
  file: File;
  id: string;
  status: 'pending' | 'uploading' | 'complete' | 'error';
  progress: number;
  error?: string;
}

function AttachmentPicker({
  attachments,
  onAdd,
  onRemove,
}: {
  attachments: AttachmentItem[];
  onAdd: (files: FileList) => void;
  onRemove: (id: string) => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <div className="bdf-attachments">
      <input
        ref={inputRef}
        type="file"
        multiple
        onChange={e => { if (e.target.files?.length) onAdd(e.target.files); e.target.value = ''; }}
        className="bdf-attachment-input"
        aria-label="Attach files"
      />
      <button
        type="button"
        className="bdf-attachment-btn"
        onClick={() => inputRef.current?.click()}
      >
        + Attach files
      </button>
      {attachments.length > 0 && (
        <ul className="bdf-attachment-list">
          {attachments.map(a => (
            <li key={a.id} className={`bdf-attachment-item bdf-attachment-${a.status}`}>
              <span className="bdf-attachment-name">{a.file.name}</span>
              <span className="bdf-attachment-size">{formatBytes(a.file.size)}</span>
              {a.status === 'uploading' && (
                <div className="bdf-attachment-progress">
                  <div className="bdf-attachment-progress-fill" style={{ width: `${a.progress}%` }} />
                </div>
              )}
              {a.status === 'error' && <span className="bdf-attachment-error">{a.error}</span>}
              {a.status !== 'uploading' && (
                <button type="button" className="bdf-attachment-remove" onClick={() => onRemove(a.id)} aria-label={`Remove ${a.file.name}`}>
                  &times;
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

// Dependency type-ahead picker
function DepPicker({
  selectedDeps,
  availableBeads,
  onAdd,
  onRemove,
}: {
  selectedDeps: BeadSummary[];
  availableBeads: BeadSummary[];
  onAdd: (bead: BeadSummary) => void;
  onRemove: (id: string) => void;
}) {
  const [search, setSearch] = useState('');
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const selectedIds = new Set(selectedDeps.map(d => d.id));
  const filtered = useMemo(() => {
    if (!search.trim()) return availableBeads.filter(b => !selectedIds.has(b.id)).slice(0, 10);
    const q = search.toLowerCase();
    return availableBeads
      .filter(b => !selectedIds.has(b.id) && (b.id.toLowerCase().includes(q) || b.title.toLowerCase().includes(q)))
      .slice(0, 10);
  }, [search, availableBeads, selectedIds]);

  // Close dropdown on outside click
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  return (
    <div className="dep-picker" ref={containerRef}>
      <div className="dep-picker-selected">
        {selectedDeps.map(dep => (
          <span key={dep.id} className="dep-chip">
            <span className="dep-chip-id">{dep.id}</span>
            <span className="dep-chip-title">{dep.title}</span>
            <button type="button" onClick={() => onRemove(dep.id)} className="dep-chip-remove" aria-label={`Remove ${dep.id}`}>×</button>
          </span>
        ))}
      </div>
      <div className="dep-picker-input-row">
        <input
          type="text"
          value={search}
          onChange={e => { setSearch(e.target.value); setOpen(true); }}
          onFocus={() => setOpen(true)}
          placeholder="Search open beads…"
          className="dep-search-input"
        />
      </div>
      {open && filtered.length > 0 && (
        <ul className="dep-dropdown" role="listbox">
          {filtered.map(bead => (
            <li
              key={bead.id}
              className="dep-dropdown-item"
              role="option"
              aria-selected={false}
              onMouseDown={e => {
                e.preventDefault();
                onAdd(bead);
                setSearch('');
                setOpen(false);
              }}
            >
              <span className="dep-dropdown-id">{bead.id}</span>
              <span className="dep-dropdown-title">{bead.title}</span>
              <span className="dep-dropdown-type">{bead.issue_type}</span>
            </li>
          ))}
        </ul>
      )}
      {open && filtered.length === 0 && search.trim() && (
        <div className="dep-dropdown dep-dropdown-empty">No matching open beads</div>
      )}
    </div>
  );
}

// Preview card component showing "What Will This Take?" data
function PreviewCard({
  preview,
  loading,
  error,
}: {
  preview: StitchPreview | null;
  loading: boolean;
  error: string | null;
}) {
  if (loading) {
    return (
      <div className="bdf-preview-card">
        <div className="bdf-preview-header">
          <span className="bdf-preview-title">What Will This Take?</span>
          <span className="bdf-preview-loading">Loading…</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bdf-preview-card">
        <div className="bdf-preview-header">
          <span className="bdf-preview-title">What Will This Take?</span>
        </div>
        <div className="bdf-preview-error">{error}</div>
      </div>
    );
  }

  if (!preview) {
    return null;
  }

  const formatCurrency = (val: number) => `$${val.toFixed(2)}`;
  const formatDuration = (seconds: number) => {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
    return `${(seconds / 3600).toFixed(1)}h`;
  };

  return (
    <div className="bdf-preview-card">
      <div className="bdf-preview-header">
        <span className="bdf-preview-title">What Will This Take?</span>
        {preview.prediction && (
          <span className="bdf-preview-meta">
            Based on {preview.prediction.similar_count} similar stitch{preview.prediction.similar_count !== 1 ? 'es' : ''}
          </span>
        )}
      </div>

      {/* Prediction Section */}
      {preview.prediction ? (
        <div className="bdf-preview-section">
          <div className="bdf-preview-subheader">Estimated Cost & Duration</div>
          <div className="bdf-preview-metrics">
            <div className="bdf-metric">
              <span className="bdf-metric-label">Cost</span>
              <span className="bdf-metric-value">
                {formatCurrency(preview.prediction.cost.p50)}
                <span className="bdf-metric-p90"> (p90: {formatCurrency(preview.prediction.cost.p90)})</span>
              </span>
            </div>
            <div className="bdf-metric">
              <span className="bdf-metric-label">Duration</span>
              <span className="bdf-metric-value">
                {formatDuration(preview.prediction.duration.p50)}
                <span className="bdf-metric-p90"> (p90: {formatDuration(preview.prediction.duration.p90)})</span>
              </span>
            </div>
          </div>
          {preview.prediction.likely_adapter_model && (
            <div className="bdf-preview-claimer">
              <span className="bdf-claimer-label">Likely claimer:</span>
              <span className="bdf-claimer-value">{preview.prediction.likely_adapter_model}</span>
            </div>
          )}
        </div>
      ) : (
        <div className="bdf-preview-section">
          <div className="bdf-preview-empty">No similar stitches found in the last 90 days</div>
        </div>
      )}

      {/* Risk Patterns Section */}
      {preview.risk_patterns.length > 0 && (
        <div className="bdf-preview-section">
          <div className="bdf-preview-subheader">Risk Patterns</div>
          <div className="bdf-risk-patterns">
            {preview.risk_patterns.map((match) => (
              <div
                key={match.pattern.id}
                className={`bdf-risk-pattern bdf-risk-severity-${match.pattern.severity}`}
              >
                <div className="bdf-risk-header">
                  <span className="bdf-risk-name">{match.pattern.name}</span>
                  <span className="bdf-risk-confidence">{Math.round(match.confidence * 100)}%</span>
                </div>
                <div className="bdf-risk-description">{match.pattern.description}</div>
                <div className="bdf-risk-fix">
                  <strong>Fix:</strong> {match.pattern.fix_recommendation}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* File Conflicts Section */}
      {preview.file_conflicts.length > 0 && (
        <div className="bdf-preview-section">
          <div className="bdf-preview-subheader">File Overlap Conflicts</div>
          <div className="bdf-file-conflicts">
            {preview.file_conflicts.map((conflict) => (
              <div key={conflict.bead_id} className="bdf-file-conflict">
                <span className="bdf-conflict-bead">
                  {conflict.bead_id}: {conflict.title}
                </span>
                <span className="bdf-conflict-files">
                  {conflict.overlapping_files.length} file{conflict.overlapping_files.length !== 1 ? 's' : ''}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Similar Stitches Section */}
      {preview.similar_stitches.length > 0 && (
        <div className="bdf-preview-section">
          <div className="bdf-preview-subheader">Similar Stitches</div>
          <div className="bdf-similar-stitches">
            {preview.similar_stitches.map((stitch) => (
              <div key={stitch.id} className="bdf-similar-stitch">
                <span className="bdf-similar-id">{stitch.id}</span>
                <span className="bdf-similar-title">{stitch.title}</span>
                <span className="bdf-similar-similarity">{Math.round(stitch.similarity * 100)}%</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export default function BeadDraftForm({ projectName, onClose, onCreated }: BeadDraftFormProps) {
  const allProjects = useAtomValue(projectCardsAtom);
  const allBeads = useAtomValue(beadsAtom);

  const [form, setForm] = useState<FormState>({
    title: '',
    description: '',
    kind: 'task',
    stitchKind: 'fix',
    priority: '',
    assignee: '',
    labelInput: '',
    labels: [],
    depSearch: '',
    dependencies: [],
    hasAcceptanceCriteria: false,
    stitchMode: false,
  });

  const [selectedProject, setSelectedProject] = useState(projectName);
  const [showPreview, setShowPreview] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [availableBeads, setAvailableBeads] = useState<BeadSummary[]>([]);
  const [loadingBeads, setLoadingBeads] = useState(false);
  const [attachments, setAttachments] = useState<AttachmentItem[]>([]);
  const [stitchPreview, setStitchPreview] = useState<StitchPreview | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

  // Semantic dedup state
  const [dedupMatches, setDedupMatches] = useState<DedupMatchRef[]>([]);
  const [forceCreate, setForceCreate] = useState(false);

  // Projects that have a valid path (workspaces exist)
  const validProjects = useMemo(
    () => allProjects.filter(p => p.path && !p.degraded),
    [allProjects],
  );

  const selectedProjectData = validProjects.find(p => p.name === selectedProject);
  const projectHasWorkspace = Boolean(selectedProjectData);

  // Fetch open beads for the dep picker when project changes
  useEffect(() => {
    if (!selectedProject) return;
    setLoadingBeads(true);
    setAvailableBeads([]);
    fetch(`/api/p/${encodeURIComponent(selectedProject)}/beads`)
      .then(r => (r.ok ? r.json() : []))
      .then((beads: BeadSummary[]) => {
        setAvailableBeads(beads);
      })
      .catch(() => {
        // Fall back to the global beads atom
        const fallback: BeadSummary[] = (allBeads as BeadData[])
          .filter(b => b.status === 'open')
          .map(b => ({
            id: b.id,
            title: b.title,
            issue_type: b.issue_type,
            priority: b.priority,
            dependencies: b.dependencies,
          }));
        setAvailableBeads(fallback);
      })
      .finally(() => setLoadingBeads(false));
  }, [selectedProject, allBeads]);

  // Infer default priority from queue length
  const inferredPriority = useMemo(() => {
    const openCount = allBeads.filter(b => b.status === 'open').length;
    if (openCount <= 3) return 0;
    if (openCount <= 8) return 1;
    return 2;
  }, [allBeads]);

  const titleValid = form.title.trim().length > 0;
  const projectValid = projectHasWorkspace;
  const canSubmit = titleValid && projectValid && !isSubmitting;

  const handleLabelAdd = useCallback((label: string) => {
    setForm(f => ({ ...f, labels: f.labels.includes(label) ? f.labels : [...f.labels, label] }));
  }, []);

  const handleLabelRemove = useCallback((label: string) => {
    setForm(f => ({ ...f, labels: f.labels.filter(l => l !== label) }));
  }, []);

  const handleDepAdd = useCallback((bead: BeadSummary) => {
    setForm(f => ({
      ...f,
      dependencies: f.dependencies.some(d => d.id === bead.id)
        ? f.dependencies
        : [...f.dependencies, bead],
    }));
  }, []);

  const handleDepRemove = useCallback((id: string) => {
    setForm(f => ({ ...f, dependencies: f.dependencies.filter(d => d.id !== id) }));
  }, []);

  const handleAttachmentAdd = useCallback((files: FileList) => {
    const newItems: AttachmentItem[] = Array.from(files).map(f => ({
      file: f,
      id: `${f.name}-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
      status: 'pending' as const,
      progress: 0,
    }));
    setAttachments(prev => [...prev, ...newItems]);
  }, []);

  const handleAttachmentRemove = useCallback((id: string) => {
    setAttachments(prev => prev.filter(a => a.id !== id));
  }, []);

  // Debounced preview fetch
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (!form.title.trim() || !selectedProject) {
        setStitchPreview(null);
        setPreviewError(null);
        return;
      }

      setPreviewLoading(true);
      setPreviewError(null);

      const params = new URLSearchParams({
        title: form.title.trim(),
        ...(form.description.trim() && { description: form.description.trim() }),
        ...(form.labels.length > 0 && { labels: form.labels.join(',') }),
      });

      fetch(`/api/p/${encodeURIComponent(selectedProject)}/beads/preview?${params}`)
        .then(async (res) => {
          if (!res.ok) {
            const text = await res.text();
            throw new Error(text || `Preview failed: ${res.status}`);
          }
          return res.json() as Promise<StitchPreview>;
        })
        .then((data) => {
          setStitchPreview(data);
        })
        .catch((err) => {
          console.error('Preview fetch error:', err);
          setPreviewError(err.message || 'Failed to load preview');
        })
        .finally(() => {
          setPreviewLoading(false);
        });
    }, 800); // 800ms debounce

    return () => clearTimeout(timeoutId);
  }, [form.title, form.description, form.labels, selectedProject]);

  // Debounced semantic dedup check — searches across all projects
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (!form.title.trim() || !selectedProject) {
        setDedupMatches([]);
        return;
      }

      fetch(`/api/p/${encodeURIComponent(selectedProject)}/beads/dedup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          title: form.title.trim(),
          description: form.description.trim() || null,
        }),
      })
        .then(async (res) => {
          if (!res.ok) throw new Error(`Dedup check failed: ${res.status}`);
          return res.json() as Promise<{ matches: DedupMatchRef[]; threshold: number; message: string | null }>;
        })
        .then((data) => {
          setDedupMatches(data.matches || []);
        })
        .catch(() => {
          setDedupMatches([]);
        });
    }, 600);

    return () => clearTimeout(timeoutId);
  }, [form.title, form.description, selectedProject]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canSubmit) return;
    setIsSubmitting(true);
    setSubmitError(null);

    try {
      const body = {
        title: form.title.trim(),
        description: form.description.trim() || null,
        issue_type: form.kind,
        priority: form.priority !== '' ? parseInt(form.priority, 10) : inferredPriority,
        dependencies: form.dependencies.map(d => d.id),
        assignee: form.assignee.trim() || null,
        labels: form.labels.length > 0 ? form.labels : null,
        source: 'form',
        force_create: forceCreate,
      };

      const res = await fetch(`/api/p/${encodeURIComponent(selectedProject)}/beads`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });

      if (!res.ok) {
        const text = await res.text();
        // Handle dedup conflict (409) with structured UI
        if (res.status === 409) {
          try {
            const errorData = JSON.parse(text);
            if (errorData.dedup_matches && Array.isArray(errorData.dedup_matches)) {
              setDedupMatches(errorData.dedup_matches);
              setSubmitError(errorData.message || 'Potential duplicate found. Confirm to proceed anyway.');
              return;
            }
          } catch {
            // Not JSON, fall through to normal error handling
          }
        }
        setSubmitError(text || `Server error ${res.status}`);
        return;
      }

      const data = await res.json();
      const beadId: string = data.id;

      // Upload attachments now that we have a bead ID
      if (attachments.length > 0) {
        await Promise.all(attachments.map(async (att) => {
          setAttachments(prev => prev.map(a => a.id === att.id ? { ...a, status: 'uploading' as const, progress: 0 } : a));
          try {
            const manager = new UploadManager(att.file, {
              attachmentType: 'bead',
              resourceId: beadId,
              onProgress: (p) => {
                setAttachments(prev => prev.map(a => a.id === att.id ? { ...a, progress: p.progress } : a));
              },
              onComplete: () => {
                setAttachments(prev => prev.map(a => a.id === att.id ? { ...a, status: 'complete' as const, progress: 100 } : a));
              },
              onError: (_id, err) => {
                setAttachments(prev => prev.map(a => a.id === att.id ? { ...a, status: 'error' as const, error: err.message } : a));
              },
            });
            await manager.start();
          } catch (err) {
            setAttachments(prev => prev.map(a => a.id === att.id ? { ...a, status: 'error' as const, error: String(err) } : a));
          }
        }));
      }

      onCreated(beadId);
    } catch (err) {
      setSubmitError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  const markdownHtml = useMemo(() => renderMarkdown(form.description), [form.description]);

  return (
    <div className="bead-draft-overlay" role="dialog" aria-modal="true" aria-label="New bead draft">
      <div className="bead-draft-form-panel">
        <div className="bead-draft-header">
          <h2 className="bead-draft-title">New Bead</h2>
          <button className="bead-draft-close" onClick={onClose} aria-label="Close form">×</button>
        </div>

        <form onSubmit={handleSubmit} className="bead-draft-form" noValidate>
          {/* Target project */}
          <div className="bdf-field">
            <label className="bdf-label" htmlFor="bdf-project">
              Project <span className="bdf-required" aria-hidden>*</span>
            </label>
            <select
              id="bdf-project"
              className={`bdf-select ${!projectValid ? 'bdf-input-error' : ''}`}
              value={selectedProject}
              onChange={e => setSelectedProject(e.target.value)}
              required
            >
              <option value="">— select project —</option>
              {allProjects.map(p => (
                <option key={p.name} value={p.name} disabled={p.degraded || !p.path}>
                  {p.label || p.name}{p.degraded ? ' (degraded)' : !p.path ? ' (no workspace)' : ''}
                </option>
              ))}
            </select>
            {selectedProject && !projectHasWorkspace && (
              <p className="bdf-error-msg">This project has no valid workspace — cannot create beads.</p>
            )}
          </div>

          {/* Title */}
          <div className="bdf-field">
            <label className="bdf-label" htmlFor="bdf-title">
              Title <span className="bdf-required" aria-hidden>*</span>
            </label>
            <input
              id="bdf-title"
              type="text"
              className={`bdf-input ${!titleValid && form.title !== '' ? 'bdf-input-error' : ''}`}
              value={form.title}
              onChange={e => setForm(f => ({ ...f, title: e.target.value }))}
              placeholder="What needs to be done?"
              required
              autoFocus
            />
            {form.title !== '' && !titleValid && (
              <p className="bdf-error-msg">Title is required.</p>
            )}
          </div>

          {/* Kind + Priority row */}
          <div className="bdf-row">
            <div className="bdf-field bdf-field-half">
              <label className="bdf-label" htmlFor="bdf-kind">Kind</label>
              <select
                id="bdf-kind"
                className="bdf-select"
                value={form.kind}
                onChange={e => setForm(f => ({ ...f, kind: e.target.value as BeadKind }))}
              >
                <option value="task">task</option>
                <option value="genesis">genesis</option>
                <option value="review">review</option>
                <option value="fix">fix</option>
                <option value="bug">bug</option>
                <option value="epic">epic</option>
              </select>
            </div>

            <div className="bdf-field bdf-field-half">
              <label className="bdf-label" htmlFor="bdf-priority">
                Priority <span className="bdf-hint">(0 = highest; default: {inferredPriority})</span>
              </label>
              <input
                id="bdf-priority"
                type="number"
                className="bdf-input"
                value={form.priority}
                onChange={e => setForm(f => ({ ...f, priority: e.target.value }))}
                min="0"
                max="9"
                placeholder={String(inferredPriority)}
              />
            </div>
          </div>

          {/* Description with preview toggle */}
          <div className="bdf-field">
            <div className="bdf-label-row">
              <label className="bdf-label" htmlFor="bdf-description">Description (markdown)</label>
              <button
                type="button"
                className={`bdf-preview-toggle ${showPreview ? 'active' : ''}`}
                onClick={() => setShowPreview(v => !v)}
              >
                {showPreview ? 'Edit' : 'Preview'}
              </button>
            </div>
            {showPreview ? (
              <div
                className="bdf-markdown-preview"
                // eslint-disable-next-line react/no-danger
                dangerouslySetInnerHTML={{ __html: markdownHtml || '<em style="color:#999">Nothing to preview</em>' }}
              />
            ) : (
              <textarea
                id="bdf-description"
                className="bdf-textarea"
                value={form.description}
                onChange={e => setForm(f => ({ ...f, description: e.target.value }))}
                placeholder="Describe the work in markdown…"
                rows={6}
              />
            )}
          </div>

          {/* Dependencies */}
          <div className="bdf-field">
            <label className="bdf-label">
              Dependencies
              {loadingBeads && <span className="bdf-hint"> (loading…)</span>}
            </label>
            <DepPicker
              selectedDeps={form.dependencies}
              availableBeads={availableBeads}
              onAdd={handleDepAdd}
              onRemove={handleDepRemove}
            />
          </div>

          {/* Bead graph delta */}
          {(form.title.trim() || form.dependencies.length > 0) && (
            <div className="bdf-field">
              <span className="bdf-label">Graph delta</span>
              <div className="bdf-graph-preview">
                <BeadGraphDelta
                  newTitle={form.title.trim() || '(untitled)'}
                  deps={form.dependencies}
                />
                <p className="bdf-graph-caption">
                  {form.dependencies.length === 0
                    ? 'No dependencies — bead will be created standalone'
                    : `This bead will depend on ${form.dependencies.length} existing bead${form.dependencies.length !== 1 ? 's' : ''}`}
                </p>
              </div>
            </div>
          )}

          {/* Assignee hint */}
          <div className="bdf-field">
            <label className="bdf-label" htmlFor="bdf-assignee">Assignee hint</label>
            <input
              id="bdf-assignee"
              type="text"
              className="bdf-input"
              value={form.assignee}
              onChange={e => setForm(f => ({ ...f, assignee: e.target.value }))}
              placeholder="Optional — who should pick this up?"
            />
          </div>

          {/* Labels */}
          <div className="bdf-field">
            <label className="bdf-label">Labels</label>
            <LabelInput
              labels={form.labels}
              onAdd={handleLabelAdd}
              onRemove={handleLabelRemove}
            />
          </div>

          {/* Attachments */}
          <div className="bdf-field">
            <label className="bdf-label">Attachments</label>
            <AttachmentPicker
              attachments={attachments}
              onAdd={handleAttachmentAdd}
              onRemove={handleAttachmentRemove}
            />
          </div>

          {/* Preview Card - What Will This Take? */}
          {form.title.trim() && (
            <PreviewCard
              preview={stitchPreview}
              loading={previewLoading}
              error={previewError}
            />
          )}

          {/* Dedup warning */}
          {dedupMatches.length > 0 && (
            <div className="bdf-field">
              <div className="sdf-dedup-warning">
                <strong>Potential duplicate{dedupMatches.length > 1 ? 's' : ''} found</strong>
                <p className="sdf-dedup-message">
                  This looks like similar work that may already be in progress:
                </p>
                <ul className="sdf-dedup-list">
                  {dedupMatches.map(m => (
                    <li key={m.id} className="sdf-dedup-item">
                      <span className="sdf-dedup-project">{m.project}</span>
                      <span className="sdf-dedup-title">{m.title}</span>
                      <span className="sdf-dedup-similarity">{Math.round(m.similarity * 100)}% match</span>
                    </li>
                  ))}
                </ul>
                <div className="sdf-dedup-actions">
                  <button
                    type="button"
                    className="bdf-btn-secondary"
                    onClick={() => {
                      // Report false positive to the backend for stats tracking
                      if (selectedProject) {
                        fetch(`/api/p/${encodeURIComponent(selectedProject)}/beads/dedup-dismiss`, {
                          method: 'POST',
                        }).catch(() => { /* non-critical */ });
                      }
                      setDedupMatches([]);
                    }}
                  >
                    Dismiss and create new
                  </button>
                  <label className="sdf-force-create-label">
                    <input
                      type="checkbox"
                      checked={forceCreate}
                      onChange={e => setForceCreate(e.target.checked)}
                    />
                    Don't ask again for this draft
                  </label>
                </div>
              </div>
            </div>
          )}

          {/* Submit error */}
          {submitError && (
            <div className="bdf-submit-error" role="alert">
              <strong>Error:</strong> {submitError}
            </div>
          )}

          {/* Actions */}
          <div className="bdf-actions">
            <button
              type="button"
              className="bdf-btn-cancel"
              onClick={onClose}
              disabled={isSubmitting}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="bdf-btn-submit"
              disabled={!canSubmit}
            >
              {isSubmitting ? 'Creating…' : 'Create Bead'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
