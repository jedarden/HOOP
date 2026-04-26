import { useState, useEffect, useCallback } from 'react';

interface OrphanBead {
  id: string;
  title: string;
  status: string;
  priority: number;
  issue_type: string;
  created_at: string;
  updated_at: string;
  created_by: string;
  dependencies: string[];
  labels: string[];
}

interface OrphansResponse {
  project: string;
  orphans: OrphanBead[];
  total_count: number;
}

interface StitchSummary {
  id: string;
  title: string;
  kind: string;
}

interface OrphansTabProps {
  projectName: string;
}

export default function OrphansTab({ projectName }: OrphansTabProps) {
  const [orphans, setOrphans] = useState<OrphanBead[]>([]);
  const [stitches, setStitches] = useState<StitchSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [attaching, setAttaching] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [selectedOrphan, setSelectedOrphan] = useState<string | null>(null);
  const [selectedStitch, setSelectedStitch] = useState<string | null>(null);

  // Fetch orphans for this project
  const fetchOrphans = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`/api/p/${encodeURIComponent(projectName)}/orphans`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      const data: OrphansResponse = await response.json();
      setOrphans(data.orphans);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [projectName]);

  // Fetch available stitches for attaching
  const fetchStitches = useCallback(async () => {
    try {
      const response = await fetch(`/api/p/${encodeURIComponent(projectName)}/stitches`);
      if (!response.ok) {
        return;
      }
      const data = await response.json();
      setStitches(data.stitches || []);
    } catch (e) {
      // Non-fatal: we can still show orphans without attach options
      console.error('Failed to fetch stitches:', e);
    }
  }, [projectName]);

  useEffect(() => {
    fetchOrphans();
    fetchStitches();
  }, [fetchOrphans, fetchStitches]);

  // Handle attach orphan to stitch
  const handleAttach = useCallback(async () => {
    if (!selectedOrphan || !selectedStitch) {
      setError('Please select both an orphan bead and a stitch');
      return;
    }

    setAttaching(selectedOrphan);
    setError(null);
    setSuccess(null);

    try {
      const response = await fetch(`/api/p/${encodeURIComponent(projectName)}/orphans/attach`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          bead_id: selectedOrphan,
          stitch_id: selectedStitch,
        }),
      });

      if (!response.ok) {
        const err = await response.text();
        throw new Error(err || `HTTP ${response.status}`);
      }

      const result = await response.json();
      setSuccess(result.message || 'Bead attached successfully');

      // Refresh the orphans list
      await fetchOrphans();

      // Clear selection
      setSelectedOrphan(null);
      setSelectedStitch(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setAttaching(null);
    }
  }, [selectedOrphan, selectedStitch, projectName, fetchOrphans]);

  const formatTimeAgo = (timestamp: string): string => {
    const now = new Date();
    const then = new Date(timestamp);
    const seconds = Math.floor((now.getTime() - then.getTime()) / 1000);

    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
    return `${Math.floor(seconds / 86400)}d`;
  };

  const getTypeBadge = (type: string): { label: string; className: string } => {
    const t = type.toLowerCase();
    switch (t) {
      case 'task': return { label: 'Task', className: 'type-task' };
      case 'bug': return { label: 'Bug', className: 'type-bug' };
      case 'epic': return { label: 'Epic', className: 'type-epic' };
      case 'genesis': return { label: 'Genesis', className: 'type-genesis' };
      case 'review': return { label: 'Review', className: 'type-review' };
      case 'fix': return { label: 'Fix', className: 'type-fix' };
      default: return { label: t, className: 'type-unknown' };
    }
  };

  return (
    <div className="orphans-tab">
      <div className="orphans-header">
        <div className="orphans-title-section">
          <h2>Orphan Beads</h2>
          <p className="orphans-description">
            Beads created outside HOOP (no <code>stitch:*</code> label). Attach them to existing Stitches to track work.
          </p>
        </div>
        <div className="orphans-stats">
          <span className="orphans-count">
            {loading ? '...' : orphans.length} orphan{orphans.length !== 1 ? 's' : ''}
          </span>
        </div>
      </div>

      {error && (
        <div className="banner banner-error" role="alert">
          <span className="banner-icon">⚠️</span>
          <span className="banner-message">{error}</span>
          <button className="banner-dismiss" onClick={() => setError(null)}>✕</button>
        </div>
      )}

      {success && (
        <div className="banner banner-success" role="status">
          <span className="banner-icon">✓</span>
          <span className="banner-message">{success}</span>
          <button className="banner-dismiss" onClick={() => setSuccess(null)}>✕</button>
        </div>
      )}

      {loading ? (
        <div className="orphans-loading">Loading orphans...</div>
      ) : orphans.length === 0 ? (
        <div className="orphans-empty">
          <p>No orphan beads found</p>
          <p className="empty-hint">
            All beads in this project are associated with Stitches. Beads created via <code>br create</code> outside of HOOP will appear here.
          </p>
        </div>
      ) : (
        <>
          {/* Attach controls */}
          {stitches.length > 0 && (
            <div className="orphans-attach-section">
              <h3>Attach Orphan to Stitch</h3>
              <div className="orphans-attach-controls">
                <div className="attach-select-group">
                  <label htmlFor="orphan-select">Orphan Bead:</label>
                  <select
                    id="orphan-select"
                    value={selectedOrphan || ''}
                    onChange={(e) => setSelectedOrphan(e.target.value || null)}
                    disabled={attaching !== null}
                  >
                    <option value="">Select an orphan...</option>
                    {orphans.map(orphan => (
                      <option key={orphan.id} value={orphan.id}>
                        {orphan.id} - {orphan.title}
                      </option>
                    ))}
                  </select>
                </div>

                <div className="attach-select-group">
                  <label htmlFor="stitch-select">Stitch:</label>
                  <select
                    id="stitch-select"
                    value={selectedStitch || ''}
                    onChange={(e) => setSelectedStitch(e.target.value || null)}
                    disabled={attaching !== null}
                  >
                    <option value="">Select a stitch...</option>
                    {stitches.map(stitch => (
                      <option key={stitch.id} value={stitch.id}>
                        {stitch.kind}: {stitch.title}
                      </option>
                    ))}
                  </select>
                </div>

                <button
                  className="attach-button"
                  onClick={handleAttach}
                  disabled={!selectedOrphan || !selectedStitch || attaching !== null}
                >
                  {attaching === selectedOrphan ? 'Attaching...' : 'Attach'}
                </button>
              </div>
            </div>
          )}

          {/* Orphans list */}
          <div className="orphans-list">
            <h3>Orphan Beads ({orphans.length})</h3>
            <table className="orphans-table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Title</th>
                  <th>Type</th>
                  <th>Priority</th>
                  <th>Created</th>
                  <th>Created By</th>
                </tr>
              </thead>
              <tbody>
                {orphans.map(orphan => {
                  const typeBadge = getTypeBadge(orphan.issue_type);
                  return (
                    <tr
                      key={orphan.id}
                      className={selectedOrphan === orphan.id ? 'orphan-row-selected' : ''}
                      onClick={() => setSelectedOrphan(orphan.id)}
                    >
                      <td><code className="orphan-id">{orphan.id}</code></td>
                      <td>{orphan.title}</td>
                      <td>
                        <span className={`badge ${typeBadge.className}`}>
                          {typeBadge.label}
                        </span>
                      </td>
                      <td>
                        <span className={`priority priority-${orphan.priority === 0 ? 'high' : orphan.priority === 1 ? 'medium' : 'low'}`}>
                          P{orphan.priority}
                        </span>
                      </td>
                      <td>{formatTimeAgo(orphan.created_at)}</td>
                      <td>{orphan.created_by}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </>
      )}
    </div>
  );
}
