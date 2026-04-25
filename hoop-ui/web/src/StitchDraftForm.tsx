import { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { useAtomValue } from 'jotai';
import { projectCardsAtom, beadsAtom, BeadData } from './atoms';
import { UploadManager, formatBytes } from './components/UploadManager';

// Stitch kind — determines decomposition behavior
export type StitchKind = 'task' | 'fix' | 'investigation' | 'genesis' | 'review';

// Map UI kind → decompose API kind. null = no decomposition, single bead.
function decomposeKind(kind: StitchKind): string | null {
  switch (kind) {
    case 'task': return 'feature';
    case 'fix': return 'fix';
    case 'investigation': return 'investigation';
    default: return null;
  }
}

// Whether this kind supports decomposition (multi-bead graph)
function isDecomposable(kind: StitchKind): boolean {
  return decomposeKind(kind) !== null;
}

interface GraphBead {
  key: string;
  title: string;
  issue_type: string;
  depends_on: string[];
  body_template: string | null;
  priority: number | null;
  labels: string[];
}

interface BeadGraph {
  rule_name: string;
  beads: GraphBead[];
}

interface DecomposeResponse {
  graph: BeadGraph;
  rule_name: string;
  bead_count: number;
  dedup_matches?: DedupMatchRef[];
  preview?: StitchPreviewData;
}

interface DedupMatchRef {
  id: string;
  project: string;
  title: string;
  kind: string;
  similarity: number;
}

interface SubmitResponse {
  stitch_id: string;
  graph: BeadGraph;
  created_beads: CreatedBeadInfo[];
  errors: string[];
  rolled_back: boolean;
}

interface CreatedBeadInfo {
  key: string;
  id: string;
  title: string;
  issue_type: string;
}

interface BeadSummary {
  id: string;
  title: string;
  issue_type: string;
  priority: number;
  dependencies: string[];
}

// Preview types — "What Will This Take?" prediction data
interface StitchPreviewData {
  schema_version?: string;
  prediction: PredictionData | null;
  risk_patterns: RiskPatternMatch[];
  file_conflicts: FileConflict[];
  similar_stitches: SimilarStitchRef[];
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
  kind: StitchKind;
  priority: string;
  assignee: string;
  labels: string[];
  dependencies: BeadSummary[];
  hasAcceptanceCriteria: boolean;
}

interface StitchDraftFormProps {
  projectName: string;
  onClose: () => void;
  onCreated: (beadIds: string[], stitchId?: string) => void;
}

// ── Simple markdown → HTML ────────────────────────────────────────────────

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
    if (raw.startsWith('### ')) { if (inUl) { out.push('</ul>'); inUl = false; } out.push(`<h3>${inlineFmt(raw.slice(4))}</h3>`); continue; }
    if (raw.startsWith('## '))  { if (inUl) { out.push('</ul>'); inUl = false; } out.push(`<h2>${inlineFmt(raw.slice(3))}</h2>`); continue; }
    if (raw.startsWith('# '))   { if (inUl) { out.push('</ul>'); inUl = false; } out.push(`<h1>${inlineFmt(raw.slice(2))}</h1>`); continue; }
    if (/^---+$/.test(raw.trim())) { if (inUl) { out.push('</ul>'); inUl = false; } out.push('<hr>'); continue; }
    if (/^[-*] /.test(raw)) {
      if (!inUl) { out.push('<ul>'); inUl = true; }
      out.push(`<li>${inlineFmt(raw.slice(2))}</li>`);
      continue;
    }
    if (raw.trim() === '') {
      if (inUl) { out.push('</ul>'); inUl = false; }
      out.push('<br>');
      continue;
    }
    if (inUl) { out.push('</ul>'); inUl = false; }
    out.push(`<p>${inlineFmt(raw)}</p>`);
  }
  if (inUl) out.push('</ul>');
  if (inCode) out.push('</code></pre>');
  return out.join('');
}

function inlineFmt(s: string): string {
  return s
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/`(.+?)`/g, '<code>$1</code>')
    .replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2">$1</a>');
}

// ── Label chip input ──────────────────────────────────────────────────────

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

// ── Attachment picker ─────────────────────────────────────────────────────

interface AttachmentItem {
  file: File;
  id: string;
  status: 'pending' | 'uploading' | 'complete' | 'error';
  progress: number;
  error?: string;
  previewUrl?: string;
}

function AttachmentPicker({ attachments, onAdd, onRemove }: {
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
      <button type="button" className="bdf-attachment-btn" onClick={() => inputRef.current?.click()}>
        + Attach files
      </button>
      {attachments.length > 0 && (
        <ul className="bdf-attachment-list">
          {attachments.map(a => (
            <li key={a.id} className={`bdf-attachment-item bdf-attachment-${a.status}`}>
              {a.previewUrl && (
                <img
                  src={a.previewUrl}
                  alt={a.file.name}
                  className="bdf-attachment-preview"
                />
              )}
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

// ── Dependency type-ahead picker ──────────────────────────────────────────

function DepPicker({ selectedDeps, availableBeads, onAdd, onRemove }: {
  selectedDeps: BeadSummary[];
  availableBeads: BeadSummary[];
  onAdd: (bead: BeadSummary) => void;
  onRemove: (id: string) => void;
}) {
  const [search, setSearch] = useState('');
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const selectedIds = useMemo(() => new Set(selectedDeps.map(d => d.id)), [selectedDeps]);
  const filtered = useMemo(() => {
    if (!search.trim()) return availableBeads.filter(b => !selectedIds.has(b.id)).slice(0, 10);
    const q = search.toLowerCase();
    return availableBeads
      .filter(b => !selectedIds.has(b.id) && (b.id.toLowerCase().includes(q) || b.title.toLowerCase().includes(q)))
      .slice(0, 10);
  }, [search, availableBeads, selectedIds]);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) setOpen(false);
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
              onMouseDown={e => { e.preventDefault(); onAdd(bead); setSearch(''); setOpen(false); }}
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

// ── Stitch bead graph preview (DAG visualization) ─────────────────────────

function typeColor(type: string) {
  switch (type) {
    case 'task': return { bg: '#e3f2fd', border: '#1976d2', text: '#1565c0' };
    case 'fix': return { bg: '#fce4ec', border: '#e91e63', text: '#c2185b' };
    case 'review': return { bg: '#f3e5f5', border: '#9c27b0', text: '#7b1fa2' };
    case 'genesis': return { bg: '#e8f5e9', border: '#388e3c', text: '#2e7d32' };
    default: return { bg: '#f5f5f5', border: '#999', text: '#666' };
  }
}

function StitchGraphPreview({ graph }: { graph: BeadGraph }) {
  const NODE_W = 130;
  const NODE_H = 52;
  const H_GAP = 60;
  const V_GAP = 16;
  const PADDING = 24;

  // Compute depth (longest path from roots) for horizontal layering
  const depthMap = useMemo(() => {
    const map = new Map<string, number>();
    const visited = new Set<string>();

    function compute(key: string): number {
      if (map.has(key)) return map.get(key)!;
      if (visited.has(key)) return 0;
      visited.add(key);

      const bead = graph.beads.find(b => b.key === key);
      if (!bead || bead.depends_on.length === 0) {
        map.set(key, 0);
        return 0;
      }
      const maxDep = Math.max(...bead.depends_on.map(d => compute(d)));
      const depth = maxDep + 1;
      map.set(key, depth);
      return depth;
    }

    for (const bead of graph.beads) compute(bead.key);
    return map;
  }, [graph]);

  // Group by depth level
  const levels = useMemo(() => {
    const lvl = new Map<number, string[]>();
    for (const [key, depth] of depthMap) {
      if (!lvl.has(depth)) lvl.set(depth, []);
      lvl.get(depth)!.push(key);
    }
    return lvl;
  }, [depthMap]);

  const maxDepth = Math.max(...depthMap.values(), 0);
  const maxLevelSize = Math.max(...[...levels.values()].map(l => l.length), 1);

  const totalW = PADDING * 2 + (maxDepth + 1) * (NODE_W + H_GAP);
  const totalH = PADDING * 2 + maxLevelSize * (NODE_H + V_GAP);

  // Position each bead
  const positions = useMemo(() => {
    const pos = new Map<string, { x: number; y: number }>();
    for (const [depth, keys] of levels) {
      const x = PADDING + depth * (NODE_W + H_GAP);
      const totalHeight = keys.length * (NODE_H + V_GAP) - V_GAP;
      const startY = (totalH - totalHeight) / 2;
      keys.forEach((key, i) => {
        pos.set(key, { x, y: startY + i * (NODE_H + V_GAP) });
      });
    }
    return pos;
  }, [levels, totalH]);

  if (graph.beads.length === 0) return null;

  return (
    <svg
      className="stitch-graph-preview"
      width={totalW}
      height={totalH}
      viewBox={`0 0 ${totalW} ${totalH}`}
      aria-label="Bead decomposition graph preview"
    >
      <defs>
        <marker id="sg-arrow" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
          <path d="M0,0 L0,6 L8,3 z" fill="#888" />
        </marker>
      </defs>

      {graph.beads.map(bead => {
        const pos = positions.get(bead.key);
        if (!pos) return null;
        const colors = typeColor(bead.issue_type);
        return (
          <g key={bead.key}>
            {bead.depends_on.map(depKey => {
              const depPos = positions.get(depKey);
              if (!depPos) return null;
              return (
                <line
                  key={`${depKey}-${bead.key}`}
                  x1={depPos.x + NODE_W}
                  y1={depPos.y + NODE_H / 2}
                  x2={pos.x}
                  y2={pos.y + NODE_H / 2}
                  stroke="#888"
                  strokeWidth="1.5"
                  markerEnd="url(#sg-arrow)"
                />
              );
            })}
            <rect
              x={pos.x}
              y={pos.y}
              width={NODE_W}
              height={NODE_H}
              rx={8}
              fill={colors.bg}
              stroke={colors.border}
              strokeWidth="1.5"
            />
            <text
              x={pos.x + NODE_W / 2}
              y={pos.y + 20}
              textAnchor="middle"
              fontSize="9"
              fontWeight="600"
              fill={colors.text}
            >
              {bead.issue_type.toUpperCase()}
            </text>
            <text
              x={pos.x + NODE_W / 2}
              y={pos.y + 38}
              textAnchor="middle"
              fontSize="7.5"
              fill="#555"
            >
              {bead.title.length > 22 ? bead.title.slice(0, 22) + '…' : bead.title}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

// ── Preview card showing "What Will This Take?" data ──────────────────────

function PreviewCard({
  preview,
  loading,
  error,
}: {
  preview: StitchPreviewData | null;
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

  if (!preview) return null;

  const formatCurrency = (val: number) => `$${val.toFixed(2)}`;
  const formatDuration = (seconds: number) => {
    if (seconds < 60) return `${Math.round(seconds)}s`;
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

// ── Main form ─────────────────────────────────────────────────────────────

export default function StitchDraftForm({ projectName, onClose, onCreated }: StitchDraftFormProps) {
  const allProjects = useAtomValue(projectCardsAtom);
  const allBeads = useAtomValue(beadsAtom);

  const [form, setForm] = useState<FormState>({
    title: '',
    description: '',
    kind: 'task',
    priority: '',
    assignee: '',
    labels: [],
    dependencies: [],
    hasAcceptanceCriteria: false,
  });

  const [selectedProject, setSelectedProject] = useState(projectName);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [availableBeads, setAvailableBeads] = useState<BeadSummary[]>([]);
  const [loadingBeads, setLoadingBeads] = useState(false);
  const [attachments, setAttachments] = useState<AttachmentItem[]>([]);
  const [isDragOver, setIsDragOver] = useState(false);

  // Keep a ref to attachments for cleanup on unmount without stale closure
  const attachmentsRef = useRef<AttachmentItem[]>([]);
  useEffect(() => { attachmentsRef.current = attachments; }, [attachments]);
  useEffect(() => {
    return () => {
      attachmentsRef.current.forEach(a => { if (a.previewUrl) URL.revokeObjectURL(a.previewUrl); });
    };
  }, []);;

  // Decomposition preview state
  const [decomposeGraph, setDecomposeGraph] = useState<BeadGraph | null>(null);
  const [decomposeLoading, setDecomposeLoading] = useState(false);
  const [decomposeError, setDecomposeError] = useState<string | null>(null);
  const [dedupMatches, setDedupMatches] = useState<DedupMatchRef[]>([]);
  const [forceCreate, setForceCreate] = useState(false);

  // "What Will This Take?" preview state
  const [stitchPreview, setStitchPreview] = useState<StitchPreviewData | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

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
      .then((beads: BeadSummary[]) => setAvailableBeads(beads))
      .catch(() => {
        const fallback: BeadSummary[] = (allBeads as BeadData[])
          .filter(b => b.status === 'open')
          .map(b => ({ id: b.id, title: b.title, issue_type: b.issue_type, priority: b.priority, dependencies: b.dependencies }));
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

  // Debounced decomposition preview
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (!form.title.trim() || !selectedProject || !isDecomposable(form.kind)) {
        setDecomposeGraph(null);
        setDecomposeError(null);
        // Don't clear dedup matches here — non-decomposable dedup runs separately
        setStitchPreview(null);
        return;
      }

      const apiKind = decomposeKind(form.kind)!;
      setDecomposeLoading(true);
      setDecomposeError(null);
      setPreviewLoading(true);

      fetch(`/api/p/${encodeURIComponent(selectedProject)}/stitch/decompose`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          kind: apiKind,
          title: form.title.trim(),
          description: form.description.trim() || null,
          has_acceptance_criteria: form.hasAcceptanceCriteria,
          priority: form.priority !== '' ? parseInt(form.priority, 10) : inferredPriority,
          labels: form.labels.length > 0 ? form.labels : null,
        }),
      })
        .then(async (res) => {
          if (!res.ok) {
            const text = await res.text();
            throw new Error(text || `Decompose failed: ${res.status}`);
          }
          return res.json() as Promise<DecomposeResponse>;
        })
        .then((data) => {
          setDecomposeGraph(data.graph);
          setDedupMatches(data.dedup_matches || []);
          setStitchPreview(data.preview || null);
        })
        .catch((err) => {
          console.error('Decompose preview error:', err);
          setDecomposeError(err.message || 'Failed to preview decomposition');
          setDecomposeGraph(null);
          setDedupMatches([]);
          setStitchPreview(null);
        })
        .finally(() => {
          setDecomposeLoading(false);
          setPreviewLoading(false);
        });
    }, 600);

    return () => clearTimeout(timeoutId);
  }, [form.title, form.description, form.kind, form.hasAcceptanceCriteria, form.priority, form.labels, selectedProject, inferredPriority]);

  // Debounced "What Will This Take?" preview fetch
  // Only runs for non-decomposable kinds (genesis, review) since decomposable kinds
  // get preview data from the decomposition response
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      // Skip for decomposable kinds - they get preview from decompose endpoint
      if (isDecomposable(form.kind)) {
        setStitchPreview(null);
        setPreviewError(null);
        return;
      }

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
          return res.json() as Promise<StitchPreviewData>;
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
  }, [form.title, form.description, form.labels, selectedProject, form.kind]);

  // Debounced dedup check for non-decomposable kinds (genesis, review)
  // Decomposable kinds get dedup from the decomposition preview response
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (isDecomposable(form.kind)) {
        return; // dedup comes from decompose endpoint
      }
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
  }, [form.title, form.description, form.kind, selectedProject]);

  // Reset dedup state when kind changes between decomposable and non-decomposable
  useEffect(() => {
    setDedupMatches([]);
    setForceCreate(false);
  }, [form.kind]);

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
      dependencies: f.dependencies.some(d => d.id === bead.id) ? f.dependencies : [...f.dependencies, bead],
    }));
  }, []);

  const handleDepRemove = useCallback((id: string) => {
    setForm(f => ({ ...f, dependencies: f.dependencies.filter(d => d.id !== id) }));
  }, []);

  const addFiles = useCallback((files: File[]) => {
    const newItems: AttachmentItem[] = files.map(f => ({
      file: f,
      id: `${f.name}-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
      status: 'pending' as const,
      progress: 0,
      previewUrl: f.type.startsWith('image/') ? URL.createObjectURL(f) : undefined,
    }));
    setAttachments(prev => [...prev, ...newItems]);
  }, []);

  const handleAttachmentAdd = useCallback((files: FileList) => {
    addFiles(Array.from(files));
  }, [addFiles]);

  const handleAttachmentRemove = useCallback((id: string) => {
    setAttachments(prev => {
      const item = prev.find(a => a.id === id);
      if (item?.previewUrl) URL.revokeObjectURL(item.previewUrl);
      return prev.filter(a => a.id !== id);
    });
  }, []);

  const handleFormPaste = useCallback((e: React.ClipboardEvent<HTMLFormElement>) => {
    const items = e.clipboardData?.items;
    if (!items) return;
    const imageFiles: File[] = [];
    for (const item of Array.from(items)) {
      if (item.type.startsWith('image/')) {
        const file = item.getAsFile();
        if (file) imageFiles.push(new File([file], file.name || `image-${Date.now()}.png`, { type: file.type }));
      }
    }
    if (imageFiles.length > 0) addFiles(imageFiles);
  }, [addFiles]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) addFiles(files);
  }, [addFiles]);

  const markdownHtml = useMemo(() => renderMarkdown(form.description), [form.description]);

  // ── Submit handler ────────────────────────────────────────────────────

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canSubmit) return;
    setIsSubmitting(true);
    setSubmitError(null);

    try {
      const priority = form.priority !== '' ? parseInt(form.priority, 10) : inferredPriority;

      if (isDecomposable(form.kind)) {
        // Multi-bead stitch via decomposition
        const apiKind = decomposeKind(form.kind)!;
        const res = await fetch(`/api/p/${encodeURIComponent(selectedProject)}/stitch/submit`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            kind: apiKind,
            title: form.title.trim(),
            description: form.description.trim() || null,
            has_acceptance_criteria: form.hasAcceptanceCriteria,
            priority,
            labels: form.labels.length > 0 ? form.labels : null,
            source: 'form',
            force_create: forceCreate,
          }),
        });

        if (!res.ok) {
          const text = await res.text();
          // Check if this is a dedup conflict response
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

        const data: SubmitResponse = await res.json();
        const beadIds = data.created_beads.map(b => b.id);
        const stitchId = data.stitch_id;

        // Upload attachments to the first created bead
        if (attachments.length > 0 && beadIds.length > 0) {
          await uploadAttachments(beadIds[0], attachments, setAttachments);
        }

        if (data.errors.length > 0) {
          setSubmitError(`Partial success: ${data.errors.join('; ')}`);
          if (beadIds.length > 0) onCreated(beadIds, stitchId);
          return;
        }

        onCreated(beadIds, stitchId);
      } else {
        // Single bead (genesis, review) — use the beads API directly
        const res = await fetch(`/api/p/${encodeURIComponent(selectedProject)}/beads`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            title: form.title.trim(),
            description: form.description.trim() || null,
            issue_type: form.kind,
            priority,
            dependencies: form.dependencies.map(d => d.id),
            assignee: form.assignee.trim() || null,
            labels: form.labels.length > 0 ? form.labels : null,
            source: 'form',
            force_create: forceCreate,
          }),
        });

        if (!res.ok) {
          const text = await res.text();
          // Check if this is a dedup conflict response
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

        if (attachments.length > 0) {
          await uploadAttachments(beadId, attachments, setAttachments);
        }

        onCreated([beadId]);
      }
    } catch (err) {
      setSubmitError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  // Graph summary text
  const graphSummary = useMemo(() => {
    if (!decomposeGraph) return null;
    const n = decomposeGraph.beads.length;
    const types = [...new Set(decomposeGraph.beads.map(b => b.issue_type))];
    return { count: n, types, ruleName: decomposeGraph.rule_name };
  }, [decomposeGraph]);

  return (
    <div className="bead-draft-overlay" role="dialog" aria-modal="true" aria-label="New stitch draft">
      <div className="bead-draft-form-panel stitch-draft-panel">
        <div className="bead-draft-header">
          <h2 className="bead-draft-title">New Stitch</h2>
          <button className="bead-draft-close" onClick={onClose} aria-label="Close form">×</button>
        </div>

        <form
          onSubmit={handleSubmit}
          className={`bead-draft-form${isDragOver ? ' bdf-drag-over' : ''}`}
          onPaste={handleFormPaste}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          noValidate
        >
          {/* Target project */}
          <div className="bdf-field">
            <label className="bdf-label" htmlFor="sdf-project">
              Project <span className="bdf-required" aria-hidden>*</span>
            </label>
            <select
              id="sdf-project"
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
              <p className="bdf-error-msg">This project has no valid workspace — cannot create stitches.</p>
            )}
          </div>

          {/* Title */}
          <div className="bdf-field">
            <label className="bdf-label" htmlFor="sdf-title">
              Title <span className="bdf-required" aria-hidden>*</span>
            </label>
            <input
              id="sdf-title"
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

          {/* Description with live markdown preview */}
          <div className="bdf-field">
            <label className="bdf-label" htmlFor="sdf-description">Description (markdown)</label>
            <textarea
              id="sdf-description"
              className="bdf-textarea"
              value={form.description}
              onChange={e => setForm(f => ({ ...f, description: e.target.value }))}
              placeholder="Describe the stitch in markdown…"
              rows={5}
            />
            {form.description.trim() && (
              <div
                className="bdf-markdown-preview bdf-live-preview"
                dangerouslySetInnerHTML={{ __html: markdownHtml }}
              />
            )}
          </div>

          {/* Kind + Priority row */}
          <div className="bdf-row">
            <div className="bdf-field bdf-field-half">
              <label className="bdf-label" htmlFor="sdf-kind">
                Kind {isDecomposable(form.kind) && <span className="sdf-kind-badge">decomposes</span>}
              </label>
              <select
                id="sdf-kind"
                className="bdf-select"
                value={form.kind}
                onChange={e => setForm(f => ({ ...f, kind: e.target.value as StitchKind }))}
              >
                <option value="task">task (default)</option>
                <option value="fix">fix</option>
                <option value="investigation">investigation</option>
                <option value="genesis">genesis</option>
                <option value="review">review</option>
              </select>
            </div>

            <div className="bdf-field bdf-field-half">
              <label className="bdf-label" htmlFor="sdf-priority">
                Priority <span className="bdf-hint">(0 = highest; default: {inferredPriority})</span>
              </label>
              <input
                id="sdf-priority"
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

          {/* Acceptance criteria toggle — only for fix */}
          {form.kind === 'fix' && (
            <div className="bdf-field sdf-acceptance-row">
              <label className="sdf-checkbox-label">
                <input
                  type="checkbox"
                  checked={form.hasAcceptanceCriteria}
                  onChange={e => setForm(f => ({ ...f, hasAcceptanceCriteria: e.target.checked }))}
                />
                Has acceptance criteria
                <span className="bdf-hint"> — produces tests + fix + review beads</span>
              </label>
            </div>
          )}

          {/* Decomposition graph preview */}
          {isDecomposable(form.kind) && form.title.trim() && selectedProject && (
            <div className="bdf-field">
              <div className="bdf-label-row">
                <span className="bdf-label">Bead graph</span>
                {decomposeLoading && <span className="sdf-graph-loading">Computing…</span>}
              </div>
              {decomposeError && (
                <div className="sdf-decompose-error">{decomposeError}</div>
              )}
              {decomposeGraph && decomposeGraph.beads.length > 0 && (
                <div className="sdf-graph-container">
                  <StitchGraphPreview graph={decomposeGraph} />
                  <p className="bdf-graph-caption sdf-graph-info">
                    {graphSummary && (
                      <>
                        Rule: <strong>{graphSummary.ruleName}</strong> — {graphSummary.count} bead{graphSummary.count !== 1 ? 's' : ''} will be created
                        ({graphSummary.types.join(', ')})
                      </>
                    )}
                  </p>
                </div>
              )}
              {!decomposeGraph && !decomposeLoading && !decomposeError && (
                <p className="bdf-graph-caption">Enter a title and select a project to preview decomposition</p>
              )}
            </div>
          )}

          {/* Dedup warning — semantic pre-dedup at draft time */}
          {dedupMatches.length > 0 && (
            <div className="bdf-field">
              <div className="sdf-dedup-warning">
                <strong>Similar work already in progress</strong>
                <p className="sdf-dedup-message">
                  This looks like <span className="sdf-dedup-ref">{dedupMatches[0].project}/{dedupMatches[0].id}</span> ({dedupMatches[0].title}), which is in progress.
                </p>
                {dedupMatches.length > 1 && (
                  <ul className="sdf-dedup-list">
                    {dedupMatches.slice(1).map(m => (
                      <li key={m.id} className="sdf-dedup-item">
                        <span className="sdf-dedup-project">{m.project}</span>
                        <span className="sdf-dedup-title">{m.title}</span>
                        <span className="sdf-dedup-similarity">{Math.round(m.similarity * 100)}%</span>
                      </li>
                    ))}
                  </ul>
                )}
                <div className="sdf-dedup-actions">
                  <button
                    type="button"
                    className="bdf-btn-dedup bdf-btn-dedup-continue"
                    onClick={onClose}
                  >
                    Continue that
                  </button>
                  <button
                    type="button"
                    className="bdf-btn-dedup bdf-btn-dedup-child"
                    onClick={() => {
                      const bestId = dedupMatches[0].id;
                      if (!form.dependencies.some(d => d.id === bestId)) {
                        setForm(f => ({
                          ...f,
                          dependencies: [...f.dependencies, {
                            id: bestId,
                            title: dedupMatches[0].title,
                            issue_type: dedupMatches[0].kind,
                            priority: 2,
                            dependencies: [],
                          }],
                        }));
                      }
                      setForceCreate(true);
                      setDedupMatches([]);
                    }}
                  >
                    Add as child
                  </button>
                  <button
                    type="button"
                    className="bdf-btn-dedup bdf-btn-dedup-new"
                    onClick={() => {
                      if (selectedProject) {
                        fetch(`/api/p/${encodeURIComponent(selectedProject)}/beads/dedup-dismiss`, {
                          method: 'POST',
                        }).catch(() => { /* non-critical */ });
                      }
                      setForceCreate(true);
                      setDedupMatches([]);
                    }}
                  >
                    Proceed as new
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Non-decomposable info */}
          {!isDecomposable(form.kind) && form.title.trim() && (
            <div className="bdf-field">
              <span className="bdf-label">Bead graph</span>
              <div className="sdf-single-bead-hint">
                <span className={`sdf-type-indicator sdf-type-${form.kind}`}>{form.kind}</span>
                <p className="bdf-graph-caption">
                  Single {form.kind} bead will be created — no decomposition rule for this kind.
                </p>
              </div>
            </div>
          )}

          {/* "What Will This Take?" preview card */}
          {form.title.trim() && selectedProject && (
            <PreviewCard
              preview={stitchPreview}
              loading={previewLoading}
              error={previewError}
            />
          )}

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

          {/* Assignee hint */}
          <div className="bdf-field">
            <label className="bdf-label" htmlFor="sdf-assignee">Assignee hint</label>
            <input
              id="sdf-assignee"
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
            <LabelInput labels={form.labels} onAdd={handleLabelAdd} onRemove={handleLabelRemove} />
          </div>

          {/* Attachments */}
          <div className="bdf-field">
            <label className="bdf-label">Attachments</label>
            <AttachmentPicker attachments={attachments} onAdd={handleAttachmentAdd} onRemove={handleAttachmentRemove} />
          </div>

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
              className="bdf-btn-submit sdf-btn-stitch"
              disabled={!canSubmit}
            >
              {isSubmitting
                ? 'Creating…'
                : isDecomposable(form.kind)
                  ? `Create Stitch (${decomposeGraph?.beads.length ?? '?'} beads)`
                  : 'Create Stitch'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// ── Helper: upload attachments to a bead ──────────────────────────────────

async function uploadAttachments(
  beadId: string,
  attachments: AttachmentItem[],
  setAttachments: React.Dispatch<React.SetStateAction<AttachmentItem[]>>,
) {
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
