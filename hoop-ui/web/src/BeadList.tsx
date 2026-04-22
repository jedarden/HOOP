import { useAtomValue } from 'jotai';
import { useState, useMemo } from 'react';
import { beadsAtom, workersAtom, BeadData, BeadStatus, BeadType } from './atoms';

type SortField = 'title' | 'status' | 'priority' | 'issue_type' | 'created_at' | 'id';
type SortOrder = 'asc' | 'desc';

interface SortConfig {
  field: SortField;
  order: SortOrder;
}

function formatTimeAgo(timestamp: string): string {
  const now = new Date();
  const then = new Date(timestamp);
  const seconds = Math.floor((now.getTime() - then.getTime()) / 1000);

  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86400)}d`;
}

function getStatusBadge(status: BeadStatus): { label: string; className: string } {
  switch (status) {
    case 'open':
      return { label: 'Open', className: 'status-open' };
    case 'closed':
      return { label: 'Closed', className: 'status-closed' };
  }
}

function getTypeBadge(type: BeadType): { label: string; className: string } {
  switch (type) {
    case 'task':
      return { label: 'Task', className: 'type-task' };
    case 'bug':
      return { label: 'Bug', className: 'type-bug' };
    case 'epic':
      return { label: 'Epic', className: 'type-epic' };
    default:
      return { label: 'Unknown', className: 'type-unknown' };
  }
}

function BeadRow({ bead, workers, expertMode }: { bead: BeadData; workers: ReturnType<typeof useAtomValue<typeof workersAtom>>; expertMode: boolean }) {
  const statusBadge = getStatusBadge(bead.status);
  const typeBadge = getTypeBadge(bead.issue_type);

  // Find if any worker is executing this bead
  const assignedWorker = workers.find(w =>
    w.state.state === 'executing' && w.state.bead === bead.id
  );

  return (
    <tr className="bead-row">
      {expertMode && (
        <td className="bead-cell bead-cell-id">
          <code className="bead-id">{bead.id}</code>
        </td>
      )}
      <td className="bead-cell bead-cell-title">
        <span className="bead-title">{bead.title}</span>
        {bead.dependencies.length > 0 && (
          <span className="bead-deps" title={`Dependencies: ${bead.dependencies.join(', ')}`}>
            {' '}· {bead.dependencies.length} dep{bead.dependencies.length !== 1 ? 's' : ''}
          </span>
        )}
      </td>
      <td className="bead-cell bead-cell-status">
        <span className={`badge ${statusBadge.className}`}>{statusBadge.label}</span>
      </td>
      <td className="bead-cell bead-cell-type">
        <span className={`badge ${typeBadge.className}`}>{typeBadge.label}</span>
      </td>
      <td className="bead-cell bead-cell-worker">
        {assignedWorker ? (
          <span className="worker-name" title={`Executed by ${assignedWorker.worker}`}>
            {assignedWorker.worker}
          </span>
        ) : (
          <span className="worker-unassigned">—</span>
        )}
      </td>
      <td className="bead-cell bead-cell-priority">
        <span className={`priority priority-${bead.priority === 0 ? 'high' : bead.priority === 1 ? 'medium' : 'low'}`}>
          {bead.priority === 0 ? 'P0' : bead.priority === 1 ? 'P1' : `P${bead.priority}`}
        </span>
      </td>
      <td className="bead-cell bead-cell-age">
        {formatTimeAgo(bead.created_at)}
      </td>
      {expertMode && (
        <>
          <td className="bead-cell bead-cell-created-by">{bead.created_by}</td>
          <td className="bead-cell bead-cell-updated">{formatTimeAgo(bead.updated_at)}</td>
        </>
      )}
    </tr>
  );
}

export default function BeadList() {
  const beads = useAtomValue(beadsAtom);
  const workers = useAtomValue(workersAtom);
  const [sortConfig, setSortConfig] = useState<SortConfig>({ field: 'created_at', order: 'desc' });
  const [expertMode, setExpertMode] = useState(false);

  const sortedBeads = useMemo(() => {
    const sortable = [...beads];
    sortable.sort((a, b) => {
      let aVal: string | number;
      let bVal: string | number;

      switch (sortConfig.field) {
        case 'title':
          aVal = a.title.toLowerCase();
          bVal = b.title.toLowerCase();
          break;
        case 'status':
          aVal = a.status;
          bVal = b.status;
          break;
        case 'priority':
          aVal = a.priority;
          bVal = b.priority;
          break;
        case 'issue_type':
          aVal = a.issue_type;
          bVal = b.issue_type;
          break;
        case 'created_at':
          aVal = a.created_at;
          bVal = b.created_at;
          break;
        case 'id':
          aVal = a.id;
          bVal = b.id;
          break;
        default:
          return 0;
      }

      if (aVal < bVal) return sortConfig.order === 'asc' ? -1 : 1;
      if (aVal > bVal) return sortConfig.order === 'asc' ? 1 : -1;
      return 0;
    });
    return sortable;
  }, [beads, sortConfig]);

  function handleSort(field: SortField) {
    setSortConfig(prev => ({
      field,
      order: prev.field === field && prev.order === 'asc' ? 'desc' : 'asc'
    }));
  }

  function SortHeader({ field, children }: { field: SortField; children: React.ReactNode }) {
    const isActive = sortConfig.field === field;
    const order = isActive ? sortConfig.order : null;

    return (
      <th
        className={`bead-header bead-header-${field} ${isActive ? 'sorted' : ''}`}
        onClick={() => handleSort(field)}
      >
        <span className="header-content">
          {children}
          {isActive && (
            <span className={`sort-indicator ${order === 'asc' ? 'asc' : 'desc'}`}>
              {order === 'asc' ? '↑' : '↓'}
            </span>
          )}
        </span>
      </th>
    );
  }

  return (
    <section className="bead-section">
      <div className="bead-section-header">
        <h2>Beads ({beads.length})</h2>
        <label className="expert-toggle">
          <input
            type="checkbox"
            checked={expertMode}
            onChange={(e) => setExpertMode(e.target.checked)}
          />
          Expert Mode
        </label>
      </div>

      {beads.length === 0 ? (
        <div className="bead-empty">
          <p>No beads found. Beads will appear here as they are created.</p>
        </div>
      ) : (
        <div className="bead-table-container">
          <table className="bead-table">
            <thead>
              <tr>
                {expertMode && (
                  <SortHeader field="id">ID</SortHeader>
                )}
                <SortHeader field="title">Title</SortHeader>
                <SortHeader field="status">Status</SortHeader>
                <SortHeader field="issue_type">Type</SortHeader>
                <th className="bead-header bead-header-worker">Worker</th>
                <SortHeader field="priority">Priority</SortHeader>
                <SortHeader field="created_at">Age</SortHeader>
                {expertMode && (
                  <>
                    <th className="bead-header bead-header-created-by">Created By</th>
                    <th className="bead-header bead-header-updated">Updated</th>
                  </>
                )}
              </tr>
            </thead>
            <tbody>
              {sortedBeads.map(bead => (
                <BeadRow
                  key={bead.id}
                  bead={bead}
                  workers={workers}
                  expertMode={expertMode}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
