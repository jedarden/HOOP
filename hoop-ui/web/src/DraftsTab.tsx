import { useState, useEffect, useCallback, useMemo } from 'react';
import { useAtomValue } from 'jotai';
import { DraftRow, DraftStatus, projectCardsAtom } from './atoms';

type DraftFilter = 'pending' | 'edited' | 'all';

function formatTimeAgo(timestamp: string): string {
  const now = new Date();
  const then = new Date(timestamp);
  const seconds = Math.floor((now.getTime() - then.getTime()) / 1000);

  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86400)}d`;
}

function getKindBadge(kind: string): { label: string; className: string } {
  switch (kind) {
    case 'investigation':
      return { label: 'Investigation', className: 'badge-kind-investigation' };
    case 'fix':
      return { label: 'Fix', className: 'badge-kind-fix' };
    case 'feature':
      return { label: 'Feature', className: 'badge-kind-feature' };
    default:
      return { label: kind, className: 'badge-kind-unknown' };
  }
}

function getStatusBadge(status: DraftStatus): { label: string; className: string } {
  switch (status) {
    case 'pending':
      return { label: 'Pending', className: 'status-draft-pending' };
    case 'edited':
      return { label: 'Edited', className: 'status-draft-edited' };
    case 'approved':
      return { label: 'Approved', className: 'status-draft-approved' };
    case 'submitted':
      return { label: 'Submitted', className: 'status-draft-submitted' };
    case 'rejected':
      return { label: 'Rejected', className: 'status-draft-rejected' };
    default:
      return { label: status, className: 'status-unknown' };
  }
}

function getSourceBadge(source: string): { label: string; className: string } {
  switch (source) {
    case 'agent':
      return { label: 'Agent', className: 'badge-source-agent' };
    case 'chat':
      return { label: 'Chat', className: 'badge-source-chat' };
    case 'form':
      return { label: 'Form', className: 'badge-source-form' };
    default:
      return { label: source, className: 'badge-source-unknown' };
  }
}

interface DraftDetailModalProps {
  draft: DraftRow;
  onClose: () => void;
  onApproved: (draftId: string, stitchId: string) => void;
  onEdited: (draftId: string) => void;
  onRejected: (draftId: string) => void;
}

function DraftDetailModal({ draft, onClose, onApproved, onEdited, onRejected }: DraftDetailModalProps) {
  const [isApproving, setIsApproving] = useState(false);
  const [isRejecting, setIsRejecting] = useState(false);
  const [rejectionReason, setRejectionReason] = useState('');
  const [showRejectionInput, setShowRejectionInput] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleApprove = async () => {
    setIsApproving(true);
    setError(null);

    try {
      const response = await fetch(`/api/drafts/${draft.id}/approve`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ force_create: false }),
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `Failed to approve draft: ${response.status}`);
      }

      const data = await response.json();
      onApproved(draft.id, data.stitch_id);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsApproving(false);
    }
  };

  const handleReject = async () => {
    setIsRejecting(true);
    setError(null);

    try {
      const response = await fetch(`/api/drafts/${draft.id}/reject`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reason: rejectionReason || null }),
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `Failed to reject draft: ${response.status}`);
      }

      onRejected(draft.id);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsRejecting(false);
      setShowRejectionInput(false);
      setRejectionReason('');
    }
  };

  const handleEdit = () => {
    onEdited(draft.id);
  };

  const kindBadge = getKindBadge(draft.kind);
  const statusBadge = getStatusBadge(draft.status as DraftStatus);
  const sourceBadge = getSourceBadge(draft.source);

  return (
    <div className="draft-detail-overlay" role="dialog" aria-modal="true" aria-label="Draft details">
      <div className="draft-detail-panel">
        <div className="draft-detail-header">
          <div className="draft-detail-title-row">
            <h2 className="draft-detail-title">{draft.title}</h2>
            <button className="draft-detail-close" onClick={onClose} aria-label="Close">×</button>
          </div>
          <div className="draft-detail-badges">
            <span className={`badge ${kindBadge.className}`}>{kindBadge.label}</span>
            <span className={`badge ${statusBadge.className}`}>{statusBadge.label}</span>
            <span className={`badge ${sourceBadge.className}`}>{sourceBadge.label}</span>
            {draft.priority !== null && (
              <span className="badge badge-priority">P{draft.priority}</span>
            )}
          </div>
        </div>

        <div className="draft-detail-body">
          <div className="draft-detail-meta">
            <span className="draft-meta-item">
              <strong>ID:</strong> {draft.id}
            </span>
            <span className="draft-meta-item">
              <strong>Project:</strong> {draft.project}
            </span>
            <span className="draft-meta-item">
              <strong>Created:</strong> {formatTimeAgo(draft.created_at)} ago by {draft.created_by}
            </span>
            {draft.agent_session_id && (
              <span className="draft-meta-item">
                <strong>Session:</strong> {draft.agent_session_id}
              </span>
            )}
            {draft.version > 1 && (
              <span className="draft-meta-item">
                <strong>Version:</strong> {draft.version}
              </span>
            )}
          </div>

          {draft.description && (
            <div className="draft-detail-description">
              <h3>Description</h3>
              <p className="draft-description-text">{draft.description}</p>
            </div>
          )}

          {draft.labels.length > 0 && (
            <div className="draft-detail-labels">
              <h3>Labels</h3>
              <div className="draft-labels-list">
                {draft.labels.map(label => (
                  <span key={label} className="draft-label-chip">{label}</span>
                ))}
              </div>
            </div>
          )}

          {draft.rejection_reason && (
            <div className="draft-detail-rejection">
              <h3>Rejection Reason</h3>
              <p className="draft-rejection-text">{draft.rejection_reason}</p>
            </div>
          )}

          {error && (
            <div className="draft-detail-error" role="alert">
              <strong>Error:</strong> {error}
            </div>
          )}
        </div>

        <div className="draft-detail-actions">
          {draft.status === 'pending' || draft.status === 'edited' ? (
            <>
              <button
                type="button"
                className="draft-btn-draft-action draft-btn-cancel"
                onClick={onClose}
                disabled={isApproving || isRejecting}
              >
                Cancel
              </button>
              <button
                type="button"
                className="draft-btn-draft-action draft-btn-edit"
                onClick={handleEdit}
                disabled={isApproving || isRejecting}
              >
                Edit Draft
              </button>
              {!showRejectionInput ? (
                <>
                  <button
                    type="button"
                    className="draft-btn-draft-action draft-btn-reject"
                    onClick={() => setShowRejectionInput(true)}
                    disabled={isApproving || isRejecting}
                  >
                    Reject
                  </button>
                  <button
                    type="button"
                    className="draft-btn-draft-action draft-btn-approve"
                    onClick={handleApprove}
                    disabled={isApproving || isRejecting}
                  >
                    {isApproving ? 'Approving…' : 'Approve & Submit'}
                  </button>
                </>
              ) : (
                <>
                  <input
                    type="text"
                    className="draft-rejection-input"
                    placeholder="Rejection reason (optional)"
                    value={rejectionReason}
                    onChange={e => setRejectionReason(e.target.value)}
                    autoFocus
                  />
                  <button
                    type="button"
                    className="draft-btn-draft-action draft-btn-cancel-reject"
                    onClick={() => {
                      setShowRejectionInput(false);
                      setRejectionReason('');
                    }}
                    disabled={isRejecting}
                  >
                    Cancel
                  </button>
                  <button
                    type="button"
                    className="draft-btn-draft-action draft-btn-confirm-reject"
                    onClick={handleReject}
                    disabled={isRejecting}
                  >
                    {isRejecting ? 'Rejecting…' : 'Confirm Reject'}
                  </button>
                </>
              )}
            </>
          ) : (
            <button
              type="button"
              className="draft-btn-draft-action draft-btn-close"
              onClick={onClose}
            >
              Close
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

interface DraftEditModalProps {
  draft: DraftRow;
  onClose: () => void;
  onUpdated: (draftId: string) => void;
}

function DraftEditModal({ draft, onClose, onUpdated }: DraftEditModalProps) {
  const [title, setTitle] = useState(draft.title);
  const [description, setDescription] = useState(draft.description || '');
  const [kind, setKind] = useState(draft.kind);
  const [priority, setPriority] = useState(draft.priority?.toString() || '');
  const [labels, setLabels] = useState(draft.labels.join(', '));
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async () => {
    setIsSaving(true);
    setError(null);

    try {
      const response = await fetch(`/api/drafts/${draft.id}/edit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          title: title || draft.title,
          description: description || null,
          kind: kind || draft.kind,
          priority: priority ? parseInt(priority, 10) : null,
          labels: labels ? labels.split(',').map(l => l.trim()).filter(l => l) : null,
        }),
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `Failed to edit draft: ${response.status}`);
      }

      onUpdated(draft.id);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="draft-detail-overlay" role="dialog" aria-modal="true" aria-label="Edit draft">
      <div className="draft-detail-panel">
        <div className="draft-detail-header">
          <h2 className="draft-detail-title">Edit Draft</h2>
          <button className="draft-detail-close" onClick={onClose} aria-label="Close">×</button>
        </div>

        <div className="draft-detail-body">
          <div className="draft-edit-field">
            <label className="draft-edit-label" htmlFor="edit-title">Title</label>
            <input
              id="edit-title"
              type="text"
              className="draft-edit-input"
              value={title}
              onChange={e => setTitle(e.target.value)}
              placeholder="Draft title"
            />
          </div>

          <div className="draft-edit-field">
            <label className="draft-edit-label" htmlFor="edit-description">Description</label>
            <textarea
              id="edit-description"
              className="draft-edit-textarea"
              value={description}
              onChange={e => setDescription(e.target.value)}
              placeholder="Draft description"
              rows={4}
            />
          </div>

          <div className="draft-edit-row">
            <div className="draft-edit-field draft-edit-half">
              <label className="draft-edit-label" htmlFor="edit-kind">Kind</label>
              <select
                id="edit-kind"
                className="draft-edit-select"
                value={kind}
                onChange={e => setKind(e.target.value)}
              >
                <option value="investigation">Investigation</option>
                <option value="fix">Fix</option>
                <option value="feature">Feature</option>
              </select>
            </div>

            <div className="draft-edit-field draft-edit-half">
              <label className="draft-edit-label" htmlFor="edit-priority">Priority</label>
              <input
                id="edit-priority"
                type="number"
                className="draft-edit-input"
                value={priority}
                onChange={e => setPriority(e.target.value)}
                min="0"
                max="9"
                placeholder="0-9"
              />
            </div>
          </div>

          <div className="draft-edit-field">
            <label className="draft-edit-label" htmlFor="edit-labels">Labels (comma-separated)</label>
            <input
              id="edit-labels"
              type="text"
              className="draft-edit-input"
              value={labels}
              onChange={e => setLabels(e.target.value)}
              placeholder="label1, label2, label3"
            />
          </div>

          {error && (
            <div className="draft-detail-error" role="alert">
              <strong>Error:</strong> {error}
            </div>
          )}
        </div>

        <div className="draft-detail-actions">
          <button
            type="button"
            className="draft-btn-draft-action draft-btn-cancel"
            onClick={onClose}
            disabled={isSaving}
          >
            Cancel
          </button>
          <button
            type="button"
            className="draft-btn-draft-action draft-btn-approve"
            onClick={handleSave}
            disabled={isSaving}
          >
            {isSaving ? 'Saving…' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  );
}

interface DraftsTabProps {
  projectName?: string;
}

export default function DraftsTab({ projectName }: DraftsTabProps) {
  const allProjects = useAtomValue(projectCardsAtom);
  const [filter, setFilter] = useState<DraftFilter>('pending');
  const [drafts, setDrafts] = useState<DraftRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedDraft, setSelectedDraft] = useState<DraftRow | null>(null);
  const [editingDraft, setEditingDraft] = useState<DraftRow | null>(null);
  const [search, setSearch] = useState('');

  const fetchDrafts = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const url = projectName
        ? `/api/p/${encodeURIComponent(projectName)}/drafts`
        : '/api/drafts';

      const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`Failed to fetch drafts: ${response.status}`);
      }

      const data = await response.json();
      setDrafts(data.drafts || []);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [projectName]);

  useEffect(() => {
    fetchDrafts();
  }, [fetchDrafts]);

  const handleApprove = useCallback((_draftId: string, _stitchId: string) => {
    setDrafts(prev => prev.filter(d => d.id !== _draftId));
    setSelectedDraft(null);
  }, []);

  const handleReject = useCallback((_draftId: string) => {
    setDrafts(prev => prev.filter(d => d.id !== _draftId));
    setSelectedDraft(null);
  }, []);

  const handleEdit = useCallback((draftId: string) => {
    const draft = drafts.find(d => d.id === draftId);
    if (draft) {
      setSelectedDraft(null);
      setEditingDraft(draft);
    }
  }, [drafts]);

  const handleEditUpdate = useCallback((draftId: string) => {
    setEditingDraft(null);
    fetchDrafts();
  }, [fetchDrafts]);

  const filteredDrafts = useMemo(() => {
    let result = drafts;

    // Apply status filter
    if (filter !== 'all') {
      result = result.filter(d => d.status === filter);
    }

    // Apply search filter
    if (search.trim()) {
      const q = search.toLowerCase();
      result = result.filter(d =>
        d.title.toLowerCase().includes(q) ||
        d.id.toLowerCase().includes(q) ||
        (d.description && d.description.toLowerCase().includes(q))
      );
    }

    return result;
  }, [drafts, filter, search]);

  const actionableCount = useMemo(() => {
    return drafts.filter(d => d.status === 'pending' || d.status === 'edited').length;
  }, [drafts]);

  if (loading) {
    return (
      <div className="drafts-tab">
        <div className="drafts-loading">Loading drafts…</div>
      </div>
    );
  }

  return (
    <div className="drafts-tab">
      <div className="drafts-header">
        <h2 className="drafts-title">
          Draft Preview Queue
          {actionableCount > 0 && (
            <span className="drafts-count-badge">{actionableCount}</span>
          )}
        </h2>
        <div className="drafts-filters">
          <button
            className={`drafts-filter-btn ${filter === 'pending' ? 'active' : ''}`}
            onClick={() => setFilter('pending')}
          >
            Pending
          </button>
          <button
            className={`drafts-filter-btn ${filter === 'edited' ? 'active' : ''}`}
            onClick={() => setFilter('edited')}
          >
            Edited
          </button>
          <button
            className={`drafts-filter-btn ${filter === 'all' ? 'active' : ''}`}
            onClick={() => setFilter('all')}
          >
            All
          </button>
        </div>
        <input
          type="text"
          className="drafts-search"
          placeholder="Search drafts…"
          value={search}
          onChange={e => setSearch(e.target.value)}
        />
      </div>

      {error && (
        <div className="drafts-error" role="alert">
          <strong>Error:</strong> {error}
        </div>
      )}

      <div className="drafts-list">
        {filteredDrafts.length === 0 ? (
          <div className="drafts-empty">
            {search.trim() ? 'No drafts match your search.' : `No ${filter} drafts.`}
          </div>
        ) : (
          filteredDrafts.map(draft => {
            const kindBadge = getKindBadge(draft.kind);
            const statusBadge = getStatusBadge(draft.status as DraftStatus);
            const sourceBadge = getSourceBadge(draft.source);

            return (
              <div
                key={draft.id}
                className={`draft-card draft-card-${draft.status}`}
                onClick={() => setSelectedDraft(draft)}
                role="button"
                tabIndex={0}
                onKeyPress={e => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    setSelectedDraft(draft);
                  }
                }}
              >
                <div className="draft-card-header">
                  <h3 className="draft-card-title">{draft.title}</h3>
                  <div className="draft-card-badges">
                    <span className={`badge ${kindBadge.className}`}>{kindBadge.label}</span>
                    <span className={`badge ${statusBadge.className}`}>{statusBadge.label}</span>
                    <span className={`badge ${sourceBadge.className}`}>{sourceBadge.label}</span>
                  </div>
                </div>

                <div className="draft-card-meta">
                  <span className="draft-meta-item">{draft.id}</span>
                  <span className="draft-meta-item">{draft.project}</span>
                  <span className="draft-meta-item">{formatTimeAgo(draft.created_at)} ago</span>
                  <span className="draft-meta-item">{draft.created_by}</span>
                  {draft.version > 1 && (
                    <span className="draft-meta-item">v{draft.version}</span>
                  )}
                </div>

                {draft.description && (
                  <div className="draft-card-description">
                    {draft.description.length > 150
                      ? `${draft.description.slice(0, 150)}…`
                      : draft.description}
                  </div>
                )}

                {draft.rejection_reason && (
                  <div className="draft-card-rejection">
                    <strong>Rejected:</strong> {draft.rejection_reason}
                  </div>
                )}

                {draft.labels.length > 0 && (
                  <div className="draft-card-labels">
                    {draft.labels.slice(0, 3).map(label => (
                      <span key={label} className="draft-label-chip-small">{label}</span>
                    ))}
                    {draft.labels.length > 3 && (
                      <span className="draft-label-more">+{draft.labels.length - 3}</span>
                    )}
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>

      {selectedDraft && (
        <DraftDetailModal
          draft={selectedDraft}
          onClose={() => setSelectedDraft(null)}
          onApproved={handleApprove}
          onEdited={handleEdit}
          onRejected={handleReject}
        />
      )}

      {editingDraft && (
        <DraftEditModal
          draft={editingDraft}
          onClose={() => setEditingDraft(null)}
          onUpdated={handleEditUpdate}
        />
      )}
    </div>
  );
}
