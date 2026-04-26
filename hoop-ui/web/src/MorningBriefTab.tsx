import { useState, useEffect, useCallback } from 'react';
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

export default function MorningBriefTab() {
  const [briefs, setBriefs] = useState<MorningBrief[]>([]);
  const [latestBrief, setLatestBrief] = useState<MorningBrief | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedBrief, setExpandedBrief] = useState<string | null>(null);

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
          {/* Latest Brief */}
          <div className="border rounded-lg overflow-hidden">
            <div className="bg-gray-50 px-4 py-3 border-b flex items-center justify-between">
              <div className="flex items-center gap-3">
                <h3 className="font-semibold">Latest Brief</h3>
                <span className={`badge ${getStatusBadge(latestBrief.status).className}`}>
                  {getStatusBadge(latestBrief.status).label}
                </span>
                <span className="text-sm text-gray-500">
                  {formatTimeAgo(latestBrief.generated_at)}
                </span>
              </div>
              {latestBrief.status === 'complete' && latestBrief.draft_ids.length > 0 && (
                <span className="text-sm text-gray-500">
                  {latestBrief.draft_ids.length} draft{latestBrief.draft_ids.length !== 1 ? 's' : ''} created
                </span>
              )}
            </div>
            <div className="p-4">
              {latestBrief.status === 'running' && (
                <div className="flex items-center justify-center py-8">
                  <div className="flex items-center gap-2">
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500"></div>
                    <span className="text-gray-600">Generating morning brief...</span>
                  </div>
                </div>
              )}
              {latestBrief.status === 'failed' && (
                <div className="text-red-600">
                  <p className="font-semibold">Failed to generate morning brief</p>
                  {latestBrief.error && <p className="text-sm mt-1">{latestBrief.error}</p>}
                </div>
              )}
              {latestBrief.status === 'complete' && (
                <>
                  <div className="mb-4">
                    <h4 className="text-lg font-semibold mb-2">{latestBrief.headline}</h4>
                    <p className="text-sm text-gray-500 mb-4">
                      {formatDate(latestBrief.generated_at)}
                    </p>
                    <div
                      className="prose prose-sm max-w-none"
                      dangerouslySetInnerHTML={renderMarkdown(latestBrief.markdown_content)}
                    />
                  </div>
                  {latestBrief.draft_ids.length > 0 && (
                    <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded">
                      <p className="text-sm font-medium text-blue-800">
                        {latestBrief.draft_ids.length} draft stitch{latestBrief.draft_ids.length !== 1 ? 'es' : ''} created — check the Drafts tab to review
                      </p>
                    </div>
                  )}
                </>
              )}
            </div>
          </div>

          {/* History */}
          {briefs.length > 1 && (
            <div className="border rounded-lg overflow-hidden">
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
