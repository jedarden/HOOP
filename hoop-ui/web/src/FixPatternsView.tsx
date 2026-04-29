import { useState, useEffect, useCallback } from 'react';

interface FixPattern {
  id: string;
  name: string;
  signature_vector: number[];
  keywords: string;
  recommended_fix_template_md: string;
  example_source_stitches: string[];
  created_at: string;
  applied_count: number;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function FixPatternList({
  patterns,
  loading,
  error,
  onSelect,
}: {
  patterns: FixPattern[];
  loading: boolean;
  error: string | null;
  onSelect: (id: string) => void;
}) {
  if (loading) {
    return <div className="fix-patterns-loading">Loading fix patterns...</div>;
  }

  if (error) {
    return <div className="fix-pattern-error-banner">{error}</div>;
  }

  if (patterns.length === 0) {
    return (
      <div className="fix-patterns-empty">
        <p>No fix patterns found.</p>
        <p className="fix-patterns-hint">
          Tag stitches as patterns from the Stitches tab to build your library.
        </p>
      </div>
    );
  }

  return (
    <div className="fix-patterns-list">
      {patterns.map((pattern) => (
        <div
          key={pattern.id}
          className="fix-pattern-card"
          onClick={() => onSelect(pattern.id)}
          style={{ cursor: 'pointer' }}
        >
          <div className="fix-pattern-card-header">
            <h4 className="fix-pattern-name">{pattern.name}</h4>
            <div className="fix-pattern-card-actions">
              <span className="fix-pattern-usage">{pattern.applied_count} uses</span>
              <span className="fix-pattern-card-arrow">→</span>
            </div>
          </div>
          {pattern.keywords && (
            <div className="fix-pattern-keywords">
              {pattern.keywords.split(',').map((kw, i) => (
                <span key={i} className="fix-pattern-keyword">
                  {kw.trim()}
                </span>
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
                {pattern.example_source_stitches.map((stitchId) => (
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
  );
}

function FixPatternDetail({
  patternId: _patternId,
  loading,
  error,
  pattern,
  onBack,
}: {
  patternId: string;
  loading: boolean;
  error: string | null;
  pattern: FixPattern | null;
  onBack: () => void;
}) {
  if (loading) {
    return <div className="fix-patterns-loading">Loading pattern details...</div>;
  }

  if (error) {
    return <div className="fix-pattern-error-banner">{error}</div>;
  }

  if (!pattern) {
    return <div className="fix-patterns-empty">Pattern not found</div>;
  }

  return (
    <div className="fix-pattern-detail-view">
      <button className="fix-pattern-back-btn" onClick={onBack}>
        ← All Fix Patterns
      </button>

      <div className="fix-pattern-detail-card">
        <div className="fix-pattern-detail-header">
          <h2 className="fix-pattern-detail-name">{pattern.name}</h2>
          <div className="fix-pattern-detail-meta">
            <span className="fix-pattern-detail-usage">
              {pattern.applied_count} uses
            </span>
            <span className="fix-pattern-detail-date">
              Created {formatDate(pattern.created_at)}
            </span>
          </div>
        </div>

        {pattern.keywords && (
          <div className="fix-pattern-detail-keywords">
            <h3>Keywords</h3>
            <div className="fix-pattern-keywords">
              {pattern.keywords.split(',').map((kw, i) => (
                <span key={i} className="fix-pattern-keyword">
                  {kw.trim()}
                </span>
              ))}
            </div>
          </div>
        )}

        <div className="fix-pattern-detail-template">
          <h3>Recommended Fix</h3>
          <pre className="fix-pattern-detail-template-content">
            {pattern.recommended_fix_template_md}
          </pre>
        </div>

        {pattern.example_source_stitches.length > 0 && (
          <div className="fix-pattern-detail-examples">
            <h3>Example Stitches</h3>
            <ul>
              {pattern.example_source_stitches.map((stitchId) => (
                <li key={stitchId}>
                  <code>{stitchId}</code>
                </li>
              ))}
            </ul>
          </div>
        )}

        <div className="fix-pattern-detail-actions">
          <button
            className="fix-pattern-delete-btn"
            onClick={() => {
              if (confirm(`Delete pattern "${pattern.name}"?`)) {
                fetch(`/api/fix-patterns/${pattern.id}`, { method: 'DELETE' })
                  .then((resp) => {
                    if (resp.ok) {
                      onBack();
                    } else {
                      throw new Error('Failed to delete pattern');
                    }
                  })
                  .catch((e) =>
                    alert(e instanceof Error ? e.message : 'Failed to delete pattern')
                  );
              }
            }}
          >
            Delete Pattern
          </button>
        </div>
      </div>
    </div>
  );
}

interface FixPatternsViewProps {
  patternId?: string;
}

export default function FixPatternsView({ patternId }: FixPatternsViewProps) {
  const [patterns, setPatterns] = useState<FixPattern[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [detailPattern, setDetailPattern] = useState<FixPattern | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);

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

  const loadPatternDetail = useCallback(async (id: string) => {
    setDetailLoading(true);
    setDetailError(null);
    try {
      const resp = await fetch(`/api/fix-patterns/${id}`);
      if (!resp.ok) throw new Error('Failed to load pattern');
      const data = await resp.json();
      setDetailPattern(data);
    } catch (e) {
      setDetailError(e instanceof Error ? e.message : 'Failed to load pattern');
    } finally {
      setDetailLoading(false);
    }
  }, []);

  useEffect(() => {
    loadPatterns();
  }, [loadPatterns]);

  useEffect(() => {
    if (patternId) {
      loadPatternDetail(patternId);
    } else {
      setDetailPattern(null);
    }
  }, [patternId, loadPatternDetail]);

  const handleSelect = useCallback((id: string) => {
    window.location.hash = `#/fix-patterns/${id}`;
  }, []);

  const handleBack = useCallback(() => {
    window.location.hash = '#/fix-patterns';
  }, []);

  if (patternId) {
    return (
      <FixPatternDetail
        patternId={patternId}
        loading={detailLoading}
        error={detailError}
        pattern={detailPattern}
        onBack={handleBack}
      />
    );
  }

  return (
    <div className="fix-patterns-view">
      <div className="fix-patterns-header">
        <h2>Fix Patterns Library</h2>
        <p className="fix-patterns-subtitle">
          Reusable fix templates for common code issues. Tag stitches as patterns
          from the Stitches tab to build your library.
        </p>
      </div>
      <FixPatternList
        patterns={patterns}
        loading={loading}
        error={error}
        onSelect={handleSelect}
      />
    </div>
  );
}
