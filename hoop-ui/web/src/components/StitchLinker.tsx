import { useState, useCallback, useEffect, useRef } from 'react';

interface StitchSearchResult {
  id: string;
  project: string;
  title: string;
  kind: string;
  created_at: string;
  last_activity_at: string;
}

interface CreateStitchLinkResponse {
  from_stitch_id: string;
  to_stitch_id: string;
  kind: string;
  created_at: string;
  warning?: string;
}

interface UndoState {
  linkId: string;
  targetTitle: string;
  remaining: number;
}

interface StitchLinkerProps {
  stitchId: string;
  stitchProject: string;
  onLinkCreated?: (toStitchId: string) => void;
}

export function StitchLinker({ stitchId, stitchProject, onLinkCreated }: StitchLinkerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [results, setResults] = useState<StitchSearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [creating, setCreating] = useState(false);
  const [undoState, setUndoState] = useState<UndoState | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const undoTimeoutRef = useRef<number | null>(null);

  // Focus search input when opened
  useEffect(() => {
    if (isOpen && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isOpen]);

  // Search stitches
  useEffect(() => {
    if (!searchQuery || searchQuery.length < 2) {
      setResults([]);
      return;
    }

    const abortController = new AbortController();
    const searchTimer = setTimeout(async () => {
      setLoading(true);
      try {
        const url = new URL('/api/stitches/search', window.location.origin);
        url.searchParams.set('q', searchQuery);
        url.searchParams.set('limit', '20');
        // Don't filter by project - allow cross-project linking

        const response = await fetch(url.toString(), {
          signal: abortController.signal,
        });
        if (response.ok) {
          const data = await response.json();
          // Filter out the current stitch
          const filtered = (data.results || []).filter((r: StitchSearchResult) => r.id !== stitchId);
          setResults(filtered);
        } else {
          setResults([]);
        }
      } catch (e) {
        if ((e as Error).name !== 'AbortError') {
          setResults([]);
        }
      } finally {
        setLoading(false);
      }
    }, 300);

    return () => {
      clearTimeout(searchTimer);
      abortController.abort();
    };
  }, [searchQuery, stitchId]);

  // Handle undo countdown
  useEffect(() => {
    if (!undoState) return;

    const timer = setInterval(() => {
      setUndoState(prev => {
        if (!prev) return null;
        const next = prev.remaining - 1;
        if (next <= 0) {
          return null;
        }
        return { ...prev, remaining: next };
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [undoState]);

  const createLink = useCallback(async (toStitchId: string, targetTitle: string) => {
    if (creating) return;

    setCreating(true);
    try {
      const response = await fetch(`/api/stitches/${encodeURIComponent(stitchId)}/links`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ to_stitch_id: toStitchId, kind: 'references' }),
      });

      if (response.ok) {
        const data: CreateStitchLinkResponse = await response.json();
        if (data.warning) {
          console.warn('Link warning:', data.warning);
        }
        setIsOpen(false);
        setSearchQuery('');
        setResults([]);

        // Set up undo state
        setUndoState({
          linkId: toStitchId,
          targetTitle,
          remaining: 10,
        });

        onLinkCreated?.(toStitchId);
      } else {
        const error = await response.text();
        console.error('Failed to create link:', error);
        alert(`Failed to create link: ${error}`);
      }
    } catch (e) {
      console.error('Failed to create link:', e);
      alert('Failed to create link - network error');
    } finally {
      setCreating(false);
    }
  }, [stitchId, creating, onLinkCreated]);

  const undoLink = useCallback(async () => {
    if (!undoState) return;

    try {
      const response = await fetch(`/api/stitches/${encodeURIComponent(stitchId)}/links/${encodeURIComponent(undoState.linkId)}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        setUndoState(null);
      } else {
        console.error('Failed to undo link');
      }
    } catch (e) {
      console.error('Failed to undo link:', e);
    }
  }, [stitchId, undoState]);

  const formatTimeAgo = (timestamp: string): string => {
    const now = new Date();
    const then = new Date(timestamp);
    const seconds = Math.floor((now.getTime() - then.getTime()) / 1000);

    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    return `${Math.floor(seconds / 3600)}h`;
  };

  const getKindLabel = (kind: string): string => {
    switch (kind) {
      case 'worker': return 'Worker';
      case 'operator': return 'Operator';
      case 'dictated': return 'Dictated';
      case 'ad-hoc': return 'Ad-hoc';
      case 'screen-capture': return 'Screen';
      default: return kind;
    }
  };

  return (
    <>
      <button
        className="stitch-linker-trigger"
        onClick={() => setIsOpen(true)}
        title="Link to another stitch"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path
            d="M7.5 3.5C7.5 3.22386 7.27614 3 7 3H3.5C3.22386 3 3 3.22386 3 3.5V7C3 7.27614 3.22386 7.5 3.5 7.5H4.5V9.5H3.5C3.22386 9.5 3 9.72386 3 10V13.5C3 13.7761 3.22386 14 3.5 14H7C7.27614 14 7.5 13.7761 7.5 13.5V12.5H9.5V13.5C9.5 13.7761 9.72386 14 10 14H13.5C13.7761 14 13.5 13.7761 13.5 13.5V10C13.5 9.72386 13.2761 9.5 13 9.5H12.5V7.5H13C13.2761 7.5 13.5 7.27614 13.5 7V3.5C13.5 3.22386 13.2761 3 13 3H9.5C9.22386 3 9 3.22386 9 3.5V4.5H7.5V3.5ZM7.5 5.5V7.5H4.5V5.5H7.5ZM9.5 5.5H12.5V7.5H9.5V5.5ZM9.5 9.5H12.5V12.5H9.5V9.5ZM7.5 12.5V9.5H4.5V12.5H7.5Z"
            fill="currentColor"
          />
        </svg>
        Link stitch
      </button>

      {isOpen && (
        <div className="stitch-linker-overlay" onClick={() => setIsOpen(false)}>
          <div
            className="stitch-linker-panel"
            onClick={e => e.stopPropagation()}
            role="dialog"
            aria-label="Link to another stitch"
            aria-modal="true"
          >
            <div className="stitch-linker-header">
              <h3>Link to another stitch</h3>
              <button
                className="stitch-linker-close"
                onClick={() => setIsOpen(false)}
                aria-label="Close"
              >
                ×
              </button>
            </div>

            <div className="stitch-linker-search">
              <input
                ref={searchInputRef}
                type="text"
                placeholder="Search by title or ID..."
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
                className="stitch-linker-input"
                disabled={creating}
              />
            </div>

            <div className="stitch-linker-results">
              {loading && (
                <div className="stitch-linker-loading">Searching...</div>
              )}

              {!loading && searchQuery.length >= 2 && results.length === 0 && (
                <div className="stitch-linker-empty">No stitches found</div>
              )}

              {!loading && searchQuery.length < 2 && (
                <div className="stitch-linker-hint">
                  Type at least 2 characters to search stitches across all projects.
                </div>
              )}

              {!loading && results.map(result => (
                <div
                  key={result.id}
                  className={[
                    'stitch-linker-result',
                    creating ? 'stitch-linker-result--disabled' : '',
                  ].join(' ')}
                  onClick={() => !creating && createLink(result.id, result.title)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={e => e.key === 'Enter' && !creating && createLink(result.id, result.title)}
                >
                  <div className="stitch-linker-result-title">{result.title}</div>
                  <div className="stitch-linker-result-meta">
                    <span className="stitch-linker-result-kind">{getKindLabel(result.kind)}</span>
                    <span className="stitch-linker-result-project">{result.project}</span>
                    <span className="stitch-linker-result-time">
                      {formatTimeAgo(result.last_activity_at)} ago
                    </span>
                  </div>
                  <div className="stitch-linker-result-id">{result.id}</div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {undoState && (
        <div className="stitch-link-undo-toast">
          <div className="stitch-link-undo-content">
            <span className="stitch-link-undo-message">
              Linked to <strong>{undoState.targetTitle}</strong>
            </span>
            <button
              className="stitch-link-undo-btn"
              onClick={undoLink}
              disabled={undoState.remaining <= 0}
            >
              Undo ({undoState.remaining}s)
            </button>
          </div>
        </div>
      )}
    </>
  );
}
