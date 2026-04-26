import { useState, useEffect, useCallback } from 'react';
import { useAtom } from 'jotai';
import { ScriptEntry, ScriptRunResponse, scriptExecutionAtom } from './atoms';

export interface ScriptsTabProps {
  projectName: string;
}

interface Banner {
  type: 'success' | 'error';
  message: string;
  details?: string;
}

// Format RFC3339 timestamp for display
function formatTimestamp(ts: string | undefined): string {
  if (!ts) return 'Never';
  try {
    const date = new Date(ts);
    return date.toLocaleString();
  } catch {
    return 'Invalid';
  }
}

export default function ScriptsTab({ projectName }: ScriptsTabProps) {
  const [scripts, setScripts] = useState<ScriptEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [execState, setExecState] = useAtom(scriptExecutionAtom);
  const [banner, setBanner] = useState<Banner | null>(null);

  // Fetch available scripts for this project
  const fetchScripts = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const url = `/api/scripts?project=${encodeURIComponent(projectName)}`;
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to fetch scripts: ${response.statusText}`);
      }
      const data: ScriptEntry[] = await response.json();
      setScripts(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, [projectName]);

  useEffect(() => {
    fetchScripts();
  }, [fetchScripts]);

  // Clear banner after 5 seconds
  useEffect(() => {
    if (banner) {
      const timer = setTimeout(() => setBanner(null), 5000);
      return () => clearTimeout(timer);
    }
  }, [banner]);

  // Run a script
  const runScript = useCallback(async (scriptName: string) => {
    setExecState({ running: true, scriptName, result: null, error: null });
    setBanner(null);

    try {
      const response = await fetch(`/api/scripts/${encodeURIComponent(scriptName)}/run`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          args: [],
          project: projectName,
        }),
      });

      if (!response.ok) {
        throw new Error(`Script execution failed: ${response.statusText}`);
      }

      const result: ScriptRunResponse = await response.json();
      setExecState({ running: false, scriptName, result, error: null });

      // Show banner based on exit code
      if (result.timed_out) {
        setBanner({
          type: 'error',
          message: `Script "${scriptName}" timed out after ${result.duration_ms / 1000}s`,
        });
      } else if (result.exit_code === 0) {
        setBanner({
          type: 'success',
          message: `Script "${scriptName}" completed successfully`,
          details: result.stdout || undefined,
        });
      } else {
        setBanner({
          type: 'error',
          message: `Script "${scriptName}" failed with exit code ${result.exit_code}`,
          details: result.stderr || result.stdout || undefined,
        });
      }
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Unknown error';
      setExecState({ running: false, scriptName, result: null, error: errorMsg });
      setBanner({
        type: 'error',
        message: `Failed to run script "${scriptName}"`,
        details: errorMsg,
      });
    }
  }, [projectName, setExecState]);

  const isRunning = execState.running && execState.scriptName !== null;

  return (
    <div className="scripts-tab">
      {/* Banner */}
      {banner && (
        <div className={`banner banner-${banner.type}`} role="alert">
          <div className="banner-content">
            <strong>{banner.message}</strong>
            {banner.details && <pre className="banner-details">{banner.details}</pre>}
          </div>
          <button
            className="banner-close"
            onClick={() => setBanner(null)}
            aria-label="Close banner"
          >
            ×
          </button>
        </div>
      )}

      {/* Header */}
      <div className="scripts-header">
        <h2>Operator Scripts</h2>
        <button
          className="btn-secondary"
          onClick={fetchScripts}
          disabled={loading}
          aria-label="Refresh scripts"
        >
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      </div>

      {/* Error state */}
      {error && (
        <div className="scripts-error" role="alert">
          <strong>Error loading scripts</strong>
          <span>{error}</span>
        </div>
      )}

      {/* Empty state */}
      {!loading && !error && scripts.length === 0 && (
        <div className="scripts-empty">
          <p>No scripts available for this project.</p>
          <p className="scripts-empty-hint">
            Scripts can be added to <code>~/.hoop/scripts/</code> with optional{' '}
            <code>.yml</code> manifests. See <code>hoop-cli(1)</code> for details.
          </p>
        </div>
      )}

      {/* Scripts list */}
      {!loading && !error && scripts.length > 0 && (
        <div className="scripts-list">
          {scripts.map(script => {
            const isThisRunning = isRunning && execState.scriptName === script.name;
            const manifest = script.manifest;
            const description = manifest?.description || 'No description';
            const scope = manifest?.scope || 'global';
            const timeout = manifest?.timeout_secs || 300;

            return (
              <div key={script.name} className="script-card">
                <div className="script-header">
                  <h3 className="script-name">{script.name}</h3>
                  <div className="script-badges">
                    {manifest?.schedule && (
                      <span className="badge badge-scheduled">Scheduled</span>
                    )}
                    {script.running && (
                      <span className="badge badge-running">Running</span>
                    )}
                    {scope === 'project' && (
                      <span className="badge badge-project">Project</span>
                    )}
                    {script.executable || (
                      <span className="badge badge-not-executable">Not executable</span>
                    )}
                  </div>
                </div>

                <p className="script-description">{description}</p>

                <div className="script-meta">
                  <span className="script-timeout">Timeout: {timeout}s</span>
                  {manifest?.projects && manifest.projects.length > 0 && (
                    <span className="script-projects">
                      Projects: {manifest.projects.join(', ')}
                    </span>
                  )}
                </div>

                {manifest?.schedule && (
                  <div className="script-schedule">
                    <div className="schedule-row">
                      <span className="schedule-label">Schedule:</span>
                      <code className="schedule-value">{manifest.schedule}</code>
                    </div>
                    {script.next_fire && (
                      <div className="schedule-row">
                        <span className="schedule-label">Next run:</span>
                        <span className="schedule-value">{formatTimestamp(script.next_fire)}</span>
                      </div>
                    )}
                    {script.last_fire && (
                      <div className="schedule-row">
                        <span className="schedule-label">Last run:</span>
                        <span className="schedule-value">{formatTimestamp(script.last_fire)}</span>
                      </div>
                    )}
                  </div>
                )}

                <button
                  className="btn-primary"
                  onClick={() => runScript(script.name)}
                  disabled={isThisRunning || !script.executable}
                  aria-label={`Run script ${script.name}`}
                >
                  {isThisRunning ? 'Running...' : 'Run Script'}
                </button>
              </div>
            );
          })}
        </div>
      )}

      {/* Execution result */}
      {execState.result && !execState.running && (
        <div className="script-result">
          <h3>Execution Result</h3>
          <div className="result-meta">
            <span>Duration: {execState.result.duration_ms}ms</span>
            <span>Status: {execState.result.status}</span>
          </div>

          {execState.result.stdout && (
            <div className="result-stdout">
              <h4>Output</h4>
              <pre>{execState.result.stdout}</pre>
            </div>
          )}

          {execState.result.stderr && (
            <div className="result-stderr">
              <h4>Error Output</h4>
              <pre>{execState.result.stderr}</pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
