import { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { useAtomValue } from 'jotai';
import { projectCardsAtom, beadsAtom, BeadData } from './atoms';

export type BeadKind = 'task' | 'genesis' | 'review' | 'fix' | 'bug' | 'epic';

interface BeadSummary {
  id: string;
  title: string;
  issue_type: string;
  priority: number;
  dependencies: string[];
}

interface FormState {
  title: string;
  description: string;
  kind: BeadKind;
  priority: string;
  assignee: string;
  labelInput: string;
  labels: string[];
  depSearch: string;
  dependencies: BeadSummary[];
}

interface BeadDraftFormProps {
  projectName: string;
  onClose: () => void;
  onCreated: (beadId: string) => void;
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

export default function BeadDraftForm({ projectName, onClose, onCreated }: BeadDraftFormProps) {
  const allProjects = useAtomValue(projectCardsAtom);
  const allBeads = useAtomValue(beadsAtom);

  const [form, setForm] = useState<FormState>({
    title: '',
    description: '',
    kind: 'task',
    priority: '',
    assignee: '',
    labelInput: '',
    labels: [],
    depSearch: '',
    dependencies: [],
  });

  const [selectedProject, setSelectedProject] = useState(projectName);
  const [showPreview, setShowPreview] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [availableBeads, setAvailableBeads] = useState<BeadSummary[]>([]);
  const [loadingBeads, setLoadingBeads] = useState(false);

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
      };

      const res = await fetch(`/api/p/${encodeURIComponent(selectedProject)}/beads`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });

      if (!res.ok) {
        const text = await res.text();
        setSubmitError(text || `Server error ${res.status}`);
        return;
      }

      const data = await res.json();
      onCreated(data.id);
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
