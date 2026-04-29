import { useState, useCallback, useEffect } from 'react';

interface PatternTaggerProps {
  stitchId: string;
  stitchTitle: string;
  disabled?: boolean;
}

interface FixPattern {
  id: string;
  name: string;
  keywords: string;
  recommended_fix_template_md: string;
  example_source_stitches: string[];
  created_at: string;
  applied_count: number;
}

interface CreatePatternRequest {
  name: string;
  signature_vector: number[];
  keywords: string;
  recommended_fix_template_md: string;
  example_source_stitches: string[];
}

export default function PatternTagger({ stitchId, stitchTitle, disabled }: PatternTaggerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [patterns, setPatterns] = useState<FixPattern[]>([]);
  const [selectedPatternId, setSelectedPatternId] = useState<string>('');
  const [newPatternName, setNewPatternName] = useState('');
  const [keywords, setKeywords] = useState('');
  const [template, setTemplate] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const loadPatterns = useCallback(async () => {
    if (isOpen) {
      setIsLoading(true);
      setError(null);
      try {
        const resp = await fetch('/api/fix-patterns');
        if (!resp.ok) throw new Error('Failed to load patterns');
        const data = await resp.json();
        setPatterns(data.patterns || []);
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to load patterns');
      } finally {
        setIsLoading(false);
      }
    }
  }, [isOpen]);

  useEffect(() => {
    loadPatterns();
  }, [loadPatterns]);

  const handleSave = useCallback(async () => {
    setError(null);
    setSuccess(null);
    setIsSaving(true);

    try {
      if (selectedPatternId) {
        // Add this stitch as an example to existing pattern
        const resp = await fetch(`/api/fix-patterns/${selectedPatternId}`);
        if (!resp.ok) throw new Error('Failed to load pattern');
        const pattern = await resp.json();

        const updatedExamples = [...pattern.example_source_stitches, stitchId];
        const updateResp = await fetch(`/api/fix-patterns/${selectedPatternId}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            example_source_stitches: updatedExamples,
          }),
        });

        if (!updateResp.ok) throw new Error('Failed to update pattern');
        setSuccess(`Added to pattern: ${pattern.name}`);
      } else if (newPatternName.trim()) {
        // Create new pattern with this stitch as example
        const signature = computeSignature(stitchTitle);
        const req: CreatePatternRequest = {
          name: newPatternName.trim(),
          signature_vector: signature,
          keywords: keywords.trim(),
          recommended_fix_template_md: template.trim(),
          example_source_stitches: [stitchId],
        };

        const createResp = await fetch('/api/fix-patterns', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(req),
        });

        if (!createResp.ok) throw new Error('Failed to create pattern');
        const data = await createResp.json();
        setSuccess(`Created pattern: ${newPatternName.trim()}`);
        setSelectedPatternId(data.id);
      } else {
        setError('Please select a pattern or enter a name for a new pattern');
        setIsSaving(false);
        return;
      }

      // Reload patterns and reset form
      await loadPatterns();
      setNewPatternName('');
      setKeywords('');
      setTemplate('');
      setSelectedPatternId('');
      setTimeout(() => setIsOpen(false), 1500);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save pattern');
    } finally {
      setIsSaving(false);
    }
  }, [selectedPatternId, newPatternName, keywords, template, stitchId, stitchTitle, loadPatterns]);

  // Simple hash-based signature computation (matches backend)
  function computeSignature(title: string): number[] {
    const VECTOR_SIZE = 8;
    let hash = 0;
    for (let i = 0; i < title.length; i++) {
      const char = title.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32bit integer
    }
    const signature: number[] = [];
    for (let i = 0; i < VECTOR_SIZE; i++) {
      const byte = (hash >> (i * 8)) & 0xff;
      signature.push(byte / 255.0);
    }
    return signature;
  }

  if (!isOpen) {
    return (
      <button
        className="fix-pattern-tagger-btn"
        onClick={() => setIsOpen(true)}
        disabled={disabled}
        title="Tag as fix pattern"
      >
        🏷️ Tag as Pattern
      </button>
    );
  }

  return (
    <div className="fix-pattern-tagger-panel">
      <div className="fix-pattern-tagger-header">
        <h4>Tag as Fix Pattern</h4>
        <button onClick={() => setIsOpen(false)} className="close-btn">×</button>
      </div>

      {error && <div className="fix-pattern-error">{error}</div>}
      {success && <div className="fix-pattern-success">{success}</div>}

      {isLoading ? (
        <div className="fix-pattern-loading">Loading patterns...</div>
      ) : (
        <div className="fix-pattern-tagger-content">
          <div className="fix-pattern-field">
            <label>Existing Pattern</label>
            <select
              value={selectedPatternId}
              onChange={(e) => setSelectedPatternId(e.target.value)}
              disabled={isSaving || !!newPatternName}
            >
              <option value="">-- Select existing pattern --</option>
              {patterns.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name} ({p.applied_count} uses)
                </option>
              ))}
            </select>
          </div>

          <div className="fix-pattern-divider">OR</div>

          <div className="fix-pattern-field">
            <label>New Pattern Name</label>
            <input
              type="text"
              value={newPatternName}
              onChange={(e) => setNewPatternName(e.target.value)}
              placeholder="e.g., Unwrap Option Panic"
              disabled={isSaving || !!selectedPatternId}
            />
          </div>

          <div className="fix-pattern-field">
            <label>Keywords</label>
            <input
              type="text"
              value={keywords}
              onChange={(e) => setKeywords(e.target.value)}
              placeholder="comma-separated keywords"
              disabled={isSaving || !!selectedPatternId}
            />
          </div>

          <div className="fix-pattern-field">
            <label>Recommended Fix (Markdown)</label>
            <textarea
              value={template}
              onChange={(e) => setTemplate(e.target.value)}
              placeholder="Describe the fix pattern..."
              rows={4}
              disabled={isSaving || !!selectedPatternId}
            />
          </div>

          <div className="fix-pattern-actions">
            <button
              onClick={handleSave}
              disabled={isSaving || disabled || (!selectedPatternId && !newPatternName.trim())}
              className="save-btn"
            >
              {isSaving ? 'Saving...' : 'Save Pattern'}
            </button>
            <button
              onClick={() => setIsOpen(false)}
              disabled={isSaving}
              className="cancel-btn"
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
