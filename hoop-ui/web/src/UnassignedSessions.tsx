import { useState, useEffect, useCallback } from 'react';
import { useAtomValue } from 'jotai';
import { projectCardsAtom } from './atoms';

interface UnassignedSession {
  id: string;
  provider: string;
  kind: string;
  cwd: string;
  title: string;
  message_count: number;
  total_tokens: number;
  created_at: string;
  updated_at: string;
  complete: boolean;
}

interface UnassignedSessionsResponse {
  sessions: UnassignedSession[];
  total_count: number;
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);

  if (diffMins < 1) return 'just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  return date.toLocaleDateString();
}

function formatTokens(tokens: number): string {
  if (tokens < 1000) return tokens.toString();
  return `${(tokens / 1000).toFixed(1)}k`;
}

function getKindBadge(kind: string): { label: string; className: string } {
  switch (kind) {
    case 'worker':
      return { label: 'Fleet', className: 'badge-fleet' };
    case 'operator':
      return { label: 'Operator', className: 'badge-operator' };
    case 'dictated':
      return { label: 'Dictated', className: 'badge-dictated' };
    case 'ad-hoc':
      return { label: 'Ad-hoc', className: 'badge-ad-hoc' };
    default:
      return { label: kind, className: 'badge-ad-hoc' };
  }
}

function getProviderBadge(provider: string): { label: string; className: string } {
  switch (provider) {
    case 'claude':
      return { label: 'Claude', className: 'badge-claude' };
    case 'codex':
      return { label: 'Codex', className: 'badge-codex' };
    case 'gemini':
      return { label: 'Gemini', className: 'badge-gemini' };
    case 'opencode':
      return { label: 'OpenCode', className: 'badge-opencode' };
    case 'aider':
      return { label: 'Aider', className: 'badge-aider' };
    default:
      return { label: provider, className: 'badge-opencode' };
  }
}

export default function UnassignedSessions() {
  const projectCards = useAtomValue(projectCardsAtom);
  const [sessions, setSessions] = useState<UnassignedSession[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [assigning, setAssigning] = useState<Set<string>>(new Set());
  const [ignoring, setIgnoring] = useState<Set<string>>(new Set());

  const fetchSessions = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch('/api/unassigned');
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data: UnassignedSessionsResponse = await response.json();
      setSessions(data.sessions);
      setTotalCount(data.total_count);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSessions();
    const id = setInterval(fetchSessions, 30_000);
    return () => clearInterval(id);
  }, [fetchSessions]);

  const handleAssign = async (sessionId: string, projectName: string) => {
    setAssigning((prev) => new Set(prev).add(sessionId));

    try {
      const response = await fetch(`/api/unassigned/${sessionId}/assign`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ project: projectName }),
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      setSessions((prev) => prev.filter((s) => s.id !== sessionId));
      setTotalCount((prev) => Math.max(0, prev - 1));
    } catch (e) {
      console.error('Failed to assign session:', e);
      alert(`Failed to assign: ${e instanceof Error ? e.message : 'Unknown error'}`);
    } finally {
      setAssigning((prev) => {
        const next = new Set(prev);
        next.delete(sessionId);
        return next;
      });
    }
  };

  const handleIgnore = async (sessionId: string) => {
    if (!confirm('Ignore this session permanently?')) return;

    setIgnoring((prev) => new Set(prev).add(sessionId));

    try {
      const response = await fetch(`/api/unassigned/${sessionId}/ignore`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      setSessions((prev) => prev.filter((s) => s.id !== sessionId));
      setTotalCount((prev) => Math.max(0, prev - 1));
    } catch (e) {
      console.error('Failed to ignore session:', e);
      alert(`Failed to ignore: ${e instanceof Error ? e.message : 'Unknown error'}`);
    } finally {
      setIgnoring((prev) => {
        const next = new Set(prev);
        next.delete(sessionId);
        return next;
      });
    }
  };

  const projects = projectCards.map((p) => ({ name: p.name, label: p.label }));

  return (
    <div className="unassigned-sessions-view">
      <div className="unassigned-header">
        <div>
          <h2>Unassigned Sessions</h2>
          <p className="unassigned-description">
            Sessions discovered outside registered projects. Assign them to a project or ignore permanently.
          </p>
        </div>
        <div className="unassigned-count">
          {loading && !sessions.length ? 'Loading...' : `${totalCount} unassigned`}
        </div>
      </div>

      {error && <div className="unassigned-error">{error}</div>}

      <div className="unassigned-content">
        {sessions.length === 0 && !loading ? (
          <div className="unassigned-empty">
            <p>No unassigned sessions found.</p>
            <p className="unassigned-empty-hint">
              Sessions from CLI directories that don't match any registered project will appear here.
            </p>
          </div>
        ) : (
          <div className="unassigned-list">
            {sessions.map((session, index) => {
              const kindBadge = getKindBadge(session.kind);
              const providerBadge = getProviderBadge(session.provider);
              const isAssigning = assigning.has(session.id);
              const isIgnoring = ignoring.has(session.id);

              return (
                <div key={session.id} className="unassigned-item">
                  <div className="unassigned-item-header">
                    <span className={`badge ${providerBadge.className} badge-sm`}>
                      {providerBadge.label}
                    </span>
                    <span className={`badge ${kindBadge.className} badge-sm`}>
                      {kindBadge.label}
                    </span>
                    <span className="unassigned-item-time">
                      {formatTimestamp(session.updated_at)}
                    </span>
                  </div>

                  <h4 className="unassigned-item-title">{session.title || session.cwd}</h4>

                  <div className="unassigned-item-meta">
                    <span className="unassigned-item-path" title={session.cwd}>
                      {session.cwd}
                    </span>
                    <span className="unassigned-item-tokens">
                      {formatTokens(session.total_tokens)} tokens
                    </span>
                    <span className="unassigned-item-messages">
                      {session.message_count} messages
                    </span>
                  </div>

                  {!session.complete && (
                    <div className="unassigned-item-status">
                      <span className="status-dot live" />
                      <span>live</span>
                    </div>
                  )}

                  <div className="unassigned-item-index">#{index + 1}</div>

                  <div className="unassigned-actions">
                    <select
                      className="unassigned-assign-select"
                      disabled={isAssigning || isIgnoring}
                      value=""
                      onChange={(e) => {
                        if (e.target.value) {
                          handleAssign(session.id, e.target.value);
                          e.target.value = '';
                        }
                      }}
                    >
                      <option value="" disabled>
                        Assign to project...
                      </option>
                      {projects.map((p) => (
                        <option key={p.name} value={p.name}>
                          {p.label || p.name}
                        </option>
                      ))}
                    </select>

                    <button
                      className="unassigned-ignore-btn"
                      disabled={isAssigning || isIgnoring}
                      onClick={() => handleIgnore(session.id)}
                    >
                      {isIgnoring ? 'Ignoring...' : 'Ignore'}
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
