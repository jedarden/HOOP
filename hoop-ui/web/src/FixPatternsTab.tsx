import { useState, useEffect, useCallback } from 'react';

interface FixPattern {
  id: string;
  name: string;
  keywords: string;
  recommended_fix_template_md: string;
  example_source_stitches: string[];
  created_at: string;
  applied_count: number;
}

interface PatternExport {
  id: string;
  name: string;
  signature_vector: number[];
  keywords: string;
  recommended_fix_template_md: string;
  example_source_stitches: string[];
  created_at: string;
  applied_count: number;
}

interface PatternsExportResponse {
  patterns: PatternExport[];
  exported_at: string;
  version: string;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

export default function FixPatternsTab() {
  const [patterns, setPatterns] = useState<FixPattern[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showImportModal, setShowImportModal] = useState(false);
  const [importJson, setImportJson] = useState('');
  const [importResult, setImportResult] = useState<{ imported: number; skipped: number; ids: string[] } | null>(null);
  const [importError, setImportError] = useState<string | null>(null);

  const loadPatterns = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const resp = await fetch('/api/fix-patterns');
      if (!resp.ok) throw new Error('Failed to load patterns');
      const data = await resp.json();
      setPatterns(data.patterns || []);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load patterns');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadPatterns();
  }, [loadPatterns]);

  const handleExport = useCallback(async () => {
    try {
      const resp = await fetch('/api/fix-patterns/export');
      if (!resp.ok) throw new Error('Failed to export patterns');
      const data: PatternsExportResponse = await resp.json();
      const json = JSON.stringify(data, null, 2);
      // Trigger download
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `fix-patterns-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to export patterns');
    }
  }, []);

  const handleImport = useCallback(async () => {
    setImportError(null);
    setImportResult(null);
    try {
      const parsed = JSON.parse(importJson);
      const resp = await fetch('/api/fix-patterns/import', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(parsed),
      });
      if (!resp.ok) throw new Error('Failed to import patterns');
      const result = await resp.json();
      setImportResult(result);
      // Reload patterns after successful import
      await loadPatterns();
    } catch (e) {
      setImportError(e instanceof Error ? e.message : 'Failed to import patterns');
    }
  }, [importJson, loadPatterns]);

  const handleDelete = useCallback(async (id: string, name: string) => {
    if (!confirm(`Delete pattern "${name}"?`)) return;
    try {
      const resp = await fetch(`/api/fix-patterns/${id}`, { method: 'DELETE' });
      if (!resp.ok) throw new Error('Failed to delete pattern');
      await loadPatterns();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete pattern');
    }
  }, [loadPatterns]);

  return (
    <div className="fix-patterns-tab">
      <div className="fix-patterns-header">
        <h3>Fix Patterns Library</h3>
        <div className="fix-patterns-actions">
          <button onClick={handleExport} className="fix-patterns-action-btn" disabled={patterns.length === 0}>
            📤 Export JSON
          </button>
          <button onClick={() => setShowImportModal(true)} className="fix-patterns-action-btn">
            📥 Import JSON
          </button>
        </div>
      </div>

      {error && <div className="fix-pattern-error-banner">{error}</div>}

      <div className="fix-patterns-content">
        {loading ? (
          <div className="fix-patterns-loading">Loading patterns...</div>
        ) : patterns.length === 0 ? (
          <div className="fix-patterns-empty">
            <p>No fix patterns found.</p>
            <p className="fix-patterns-hint">
              Tag stitches as patterns from the Stitches tab to build your library.
            </p>
          </div>
        ) : (
          <div className="fix-patterns-list">
            {patterns.map(pattern => (
              <div key={pattern.id} className="fix-pattern-card">
                <div className="fix-pattern-card-header">
                  <h4 className="fix-pattern-name">{pattern.name}</h4>
                  <div className="fix-pattern-card-actions">
                    <span className="fix-pattern-usage">{pattern.applied_count} uses</span>
                    <button
                      onClick={() => handleDelete(pattern.id, pattern.name)}
                      className="fix-pattern-delete-btn"
                      title="Delete pattern"
                    >
                      🗑️
                    </button>
                  </div>
                </div>
                {pattern.keywords && (
                  <div className="fix-pattern-keywords">
                    {pattern.keywords.split(',').map((kw, i) => (
                      <span key={i} className="fix-pattern-keyword">{kw.trim()}</span>
                    ))}
                  </div>
                )}
                <div className="fix-pattern-template">
                  <h5>Recommended Fix:</h5>
                  <pre>{pattern.recommended_fix_template_md}</pre>
                </div>
                {pattern.example_source_stitches.length > 0 && (
                  <div className="fix-pattern-examples">
                    <h5>Example Stitches:</h5>
                    <ul>
                      {pattern.example_source_stitches.map(stitchId => (
                        <li key={stitchId}>
                          <code>{stitchId}</code>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
                <div className="fix-pattern-meta">
                  <span>Created {formatDate(pattern.created_at)}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {showImportModal && (
        <div className="fix-pattern-import-modal">
          <div className="fix-pattern-import-content">
            <div className="fix-pattern-import-header">
              <h4>Import Fix Patterns</h4>
              <button onClick={() => setShowImportModal(false)} className="close-btn">×</button>
            </div>
            <div className="fix-pattern-import-body">
              <p>Paste the JSON export of fix patterns below:</p>
              <textarea
                value={importJson}
                onChange={e => setImportJson(e.target.value)}
                placeholder='{"patterns": [...], "exported_at": "...", "version": "..."}'
                rows={10}
                className="fix-pattern-import-textarea"
              />
              {importError && <div className="fix-pattern-import-error">{importError}</div>}
              {importResult && (
                <div className="fix-pattern-import-success">
                  Imported {importResult.imported} patterns, skipped {importResult.skipped}.
                </div>
              )}
            </div>
            <div className="fix-pattern-import-actions">
              <button
                onClick={handleImport}
                disabled={!importJson.trim()}
                className="fix-pattern-import-btn"
              >
                Import
              </button>
              <button onClick={() => setShowImportModal(false)} className="fix-pattern-cancel-btn">
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
