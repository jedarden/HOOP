import { useState, useEffect, useCallback, useRef, TouchEvent } from 'react';
import { marked } from 'marked';

interface MorningBrief {
  id: string;
  generated_at: string;
  window_from: string;
  window_to: string;
  headline: string;
  markdown_content: string;
  draft_ids: string[];
  session_id?: string;
  status: 'running' | 'complete' | 'failed';
  error?: string;
}

interface MorningBriefData {
  id: string;
  headline: string;
  generated_at: string;
  draft_count: number;
  status: string;
}

function formatTimeAgo(timestamp: string): string {
  const now = new Date();
  const then = new Date(timestamp);
  const seconds = Math.floor((now.getTime() - then.getTime()) / 1000);

  if (seconds < 60) return `${seconds}s ago`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

function formatDate(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleDateString('en-US', {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
}

function getStatusBadge(status: string): { label: string; className: string } {
  switch (status) {
    case 'running':
      return { label: 'Generating...', className: 'status-running' };
    case 'complete':
      return { label: 'Complete', className: 'status-complete' };
    case 'failed':
      return { label: 'Failed', className: 'status-failed' };
    default:
      return { label: status, className: 'status-unknown' };
  }
}

// Swipe gesture hook for mobile (§21.1)
function useSwipeGestures(onSwipeLeft?: () => void, onSwipeRight?: () => void) {
  const touchStartRef = useRef<{ x: number; y: number } | null>(null);

  const handleTouchStart = useCallback((e: TouchEvent) => {
    touchStartRef.current = {
      x: e.touches[0].clientX,
      y: e.touches[0].clientY,
    };
  }, []);

  const handleTouchEnd = useCallback((e: TouchEvent) => {
    if (!touchStartRef.current) return;

    const touchEnd = {
      x: e.changedTouches[0].clientX,
      y: e.changedTouches[0].clientY,
    };

    const deltaX = touchEnd.x - touchStartRef.current.x;
    const deltaY = Math.abs(touchEnd.y - touchStartRef.current.y);

    // Only trigger if horizontal swipe and vertical movement is minimal
    const minSwipeDistance = 50;
    if (Math.abs(deltaX) > minSwipeDistance && deltaY < 50) {
      if (deltaX > 0 && onSwipeRight) {
        onSwipeRight();
      } else if (deltaX < 0 && onSwipeLeft) {
        onSwipeLeft();
      }
    }

    touchStartRef.current = null;
  }, [onSwipeLeft, onSwipeRight]);

  return { handleTouchStart, handleTouchEnd };
}

export default function MorningBriefTab() {
  const [briefs, setBriefs] = useState<MorningBrief[]>([]);
  const [latestBrief, setLatestBrief] = useState<MorningBrief | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedBrief, setExpandedBrief] = useState<string | null>(null);
  const [currentCardIndex, setCurrentCardIndex] = useState(0);

  // Mobile swipe gestures for card navigation (§21.1)
  const { handleTouchStart, handleTouchEnd } = useSwipeGestures(
    // Swipe left: next card
    () => {
      if (currentCardIndex < briefs.length - 1) {
        setCurrentCardIndex(currentCardIndex + 1);
        setExpandedBrief(briefs[currentCardIndex + 1]?.id ?? null);
      }
    },
    // Swipe right: previous card
    () => {
      if (currentCardIndex > 0) {
        setCurrentCardIndex(currentCardIndex - 1);
        setExpandedBrief(briefs[currentCardIndex - 1]?.id ?? null);
      }
    }
  );

  const fetchBriefs = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const response = await fetch('/api/agent/morning-brief/list');
      if (!response.ok) {
        throw new Error(`Failed to fetch morning briefs: ${response.statusText}`);
      }
      const data = await response.json();
      setBriefs(data);
      if (data.length > 0) {
        setLatestBrief(data[0]);
        setExpandedBrief(data[0].id);
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMsg);
      console.error('Error fetching morning briefs:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const checkStatus = useCallback(async () => {
    try {
      const response = await fetch('/api/agent/morning-brief/status');
      if (response.ok) {
        const data = await response.json();
        setIsRunning(data.running);
      }
    } catch (err) {
      console.error('Error checking morning brief status:', err);
    }
  }, []);

  const triggerBrief = useCallback(async () => {
    try {
      setError(null);
      const response = await fetch('/api/agent/morning-brief/trigger', {
        method: 'POST',
      });
      if (!response.ok) {
        throw new Error(`Failed to trigger morning brief: ${response.statusText}`);
      }
      const data = await response.json();
      if (data.status === 'already_running') {
        setIsRunning(true);
      } else {
        setIsRunning(true);
        // Brief started, refresh after a delay
        setTimeout(() => {
          fetchBriefs();
          checkStatus();
        }, 2000);
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMsg);
      console.error('Error triggering morning brief:', err);
    }
  }, [fetchBriefs, checkStatus]);

  useEffect(() => {
    fetchBriefs();
    checkStatus();

    // Set up WebSocket for real-time updates
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/ws`;
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      // Subscribe to morning brief updates
      ws.send(JSON.stringify({ type: 'subscribe', event_types: ['morning_brief'] }));
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.event_type === 'morning_brief' && data.morning_brief) {
          const briefData: MorningBriefData = data.morning_brief;
          setIsRunning(briefData.status === 'running');
          if (briefData.status === 'complete' || briefData.status === 'failed') {
            // Refresh the briefs list
            fetchBriefs();
          }
        }
      } catch (err) {
        console.error('Error parsing WebSocket message:', err);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    return () => {
      ws.close();
    };
  }, [fetchBriefs, checkStatus]);

  // Poll for status updates when running
  useEffect(() => {
    if (isRunning) {
      const interval = setInterval(() => {
        fetchBriefs();
        checkStatus();
      }, 5000);
      return () => clearInterval(interval);
    }
  }, [isRunning, fetchBriefs, checkStatus]);

  const renderMarkdown = (content: string) => {
    const html = marked(content);
    return { __html: html };
  };

  if (isLoading) {
    return (
      <div className="p-4">
        <div className="flex items-center justify-center h-64">
          <div className="text-gray-500">Loading morning brief...</div>
        </div>
      </div>
    );
  }

  // Mobile card view: show only current card
  const isMobileView = window.innerWidth < 768;
  const displayBriefs = isMobileView && briefs.length > 0
    ? [briefs[currentCardIndex]]
    : briefs;

  return (
    <div className="p-4">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold">Morning Brief</h2>
        <button
          onClick={triggerBrief}
          disabled={isRunning}
          className={`px-4 py-2 rounded ${
            isRunning
              ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
              : 'bg-blue-500 text-white hover:bg-blue-600'
          }`}
        >
          {isRunning ? 'Running...' : 'Generate Brief'}
        </button>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded text-red-700">
          {error}
        </div>
      )}

      {!latestBrief && !isLoading && (
        <div className="text-center py-8 text-gray-500">
          No morning briefs yet. Click "Generate Brief" to create the first one.
        </div>
      )}

      {latestBrief && (
        <div className="space-y-4">
          {/* Mobile card navigation indicators */}
          {isMobileView && briefs.length > 1 && (
            <div className="mobile-brief-nav flex items-center justify-between text-sm text-gray-500 mb-2">
              <span>Brief {currentCardIndex + 1} of {briefs.length}</span>
              <div className="flex gap-2">
                <button
                  onClick={() => setCurrentCardIndex(Math.max(0, currentCardIndex - 1))}
                  disabled={currentCardIndex === 0}
                  className="px-3 py-1 rounded bg-gray-100 disabled:opacity-50"
                >
                  ← Prev
                </button>
                <button
                  onClick={() => setCurrentCardIndex(Math.min(briefs.length - 1, currentCardIndex + 1))}
                  disabled={currentCardIndex === briefs.length - 1}
                  className="px-3 py-1 rounded bg-gray-100 disabled:opacity-50"
                >
                  Next →
                </button>
              </div>
            </div>
          )}

          {/* Brief cards with swipe gestures (§21.1) */}
          {displayBriefs.map((brief) => (
            <div
              key={brief.id}
              className="border rounded-lg overflow-hidden morning-brief-card"
              onTouchStart={handleTouchStart as any}
              onTouchEnd={handleTouchEnd as any}
            >
              <div className="bg-gray-50 px-4 py-3 border-b flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <h3 className="font-semibold">
                    {brief.id === latestBrief.id ? 'Latest Brief' : `Brief ${briefs.indexOf(brief) + 1}`}
                  </h3>
                  <span className={`badge ${getStatusBadge(brief.status).className}`}>
                    {getStatusBadge(brief.status).label}
                  </span>
                  <span className="text-sm text-gray-500">
                    {formatTimeAgo(brief.generated_at)}
                  </span>
                </div>
                {brief.status === 'complete' && brief.draft_ids.length > 0 && (
                  <span className="text-sm text-gray-500">
                    {brief.draft_ids.length} draft{brief.draft_ids.length !== 1 ? 's' : ''} created
                  </span>
                )}
              </div>
              <div className="p-4">
                {brief.status === 'running' && (
                  <div className="flex items-center justify-center py-8">
                    <div className="flex items-center gap-2">
                      <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500"></div>
                      <span className="text-gray-600">Generating morning brief...</span>
                    </div>
                  </div>
                )}
                {brief.status === 'failed' && (
                  <div className="text-red-600">
                    <p className="font-semibold">Failed to generate morning brief</p>
                    {brief.error && <p className="text-sm mt-1">{brief.error}</p>}
                  </div>
                )}
                {brief.status === 'complete' && (
                  <>
                    <div className="mb-4">
                      <h4 className="text-lg font-semibold mb-2 morning-brief-headline">{brief.headline}</h4>
                      <p className="text-sm text-gray-500 mb-4">
                        {formatDate(brief.generated_at)}
                      </p>
                      <div
                        className="prose prose-sm max-w-none morning-brief-content"
                        dangerouslySetInnerHTML={renderMarkdown(brief.markdown_content)}
                      />
                    </div>
                    {brief.draft_ids.length > 0 && (
                      <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded">
                        <p className="text-sm font-medium text-blue-800">
                          {brief.draft_ids.length} draft stitch{brief.draft_ids.length !== 1 ? 'es' : ''} created — check the Drafts tab to review
                        </p>
                      </div>
                    )}
                  </>
                )}
              </div>
            </div>
          ))}

          {/* Desktop-only history view */}
          {!isMobileView && briefs.length > 1 && (
            <div className="border rounded-lg overflow-hidden desktop-only">
              <div className="bg-gray-50 px-4 py-3 border-b">
                <h3 className="font-semibold">History</h3>
              </div>
              <div className="divide-y">
                {briefs.slice(1).map((brief) => (
                  <div key={brief.id} className="p-4">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <span className={`badge ${getStatusBadge(brief.status).className}`}>
                          {getStatusBadge(brief.status).label}
                        </span>
                        <span className="text-sm text-gray-500">
                          {formatTimeAgo(brief.generated_at)}
                        </span>
                      </div>
                      <button
                        onClick={() => setExpandedBrief(expandedBrief === brief.id ? null : brief.id)}
                        className="text-blue-500 hover:text-blue-600 text-sm"
                      >
                        {expandedBrief === brief.id ? 'Hide' : 'Show'}
                      </button>
                    </div>
                    {expandedBrief === brief.id && (
                      <div className="mt-3">
                        <p className="font-medium mb-2">{brief.headline}</p>
                        <div
                          className="prose prose-sm max-w-none"
                          dangerouslySetInnerHTML={renderMarkdown(brief.markdown_content)}
                        />
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
