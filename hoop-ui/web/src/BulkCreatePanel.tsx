/**
 * BulkCreatePanel.tsx
 *
 * Phase 4 deliverable #5: Bulk draft creation UI
 *
 * Allows operators to paste a bullet list or markdown document,
 * preview parsed drafts, and submit selected drafts as a batch.
 *
 * Features:
 * - Parse markdown/bullet lists into draft previews
 * - Edit individual draft titles and descriptions
 * - Select/deselect drafts for creation
 * - Enforce 50-draft hard cap
 * - Show success/failure results
 */

import { useState, useCallback } from "react";
import { useAtom } from "jotai";

// API types
interface ParsedDraft {
  index: number;
  title: string;
  description: string;
  kind: string;
  priority?: number;
  labels: string[];
}

interface BulkDraftItem {
  title: string;
  description: string;
  kind: string;
  priority?: number;
  labels: string[];
}

interface ParseBulkResponse {
  drafts: ParsedDraft[];
  count: number;
  exceeds_limit: boolean;
  limit: number;
}

interface SubmitBulkResponse {
  draft_ids: string[];
  created: number;
  failed: number;
  errors: string[];
}

interface BulkCreatePanelProps {
  project: string;
  onSuccess?: (draftIds: string[]) => void;
  onClose?: () => void;
}

type ViewState =
  | "input"
  | "preview"
  | "submitting"
  | "result"
  | "error";

export function BulkCreatePanel({
  project,
  onSuccess,
  onClose,
}: BulkCreatePanelProps) {
  const [view, setView] = useState<ViewState>("input");
  const [input, setInput] = useState("");
  const [defaultKind, setDefaultKind] = useState("task");
  const [defaultPriority, setDefaultPriority] = useState<number | undefined>(
    undefined
  );
  const [defaultLabels, setDefaultLabels] = useState<string[]>([]);
  const [overrideLimit, setOverrideLimit] = useState(false);

  const [parsedDrafts, setParsedDrafts] = useState<ParsedDraft[]>([]);
  const [selectedDrafts, setSelectedDrafts] = useState<Set<number>>(
    new Set()
  );
  const [editedDrafts, setEditedDrafts] = useState<Map<number, ParsedDraft>>(
    new Map()
  );

  const [submitResult, setSubmitResult] = useState<SubmitBulkResponse | null>(
    null
  );
  const [error, setError] = useState<string | null>(null);

  const handleParse = useCallback(async () => {
    if (!input.trim()) {
      setError("Please enter some content to parse");
      setView("error");
      return;
    }

    try {
      const response = await fetch("/api/bulk/parse", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          project,
          content: input,
          kind: defaultKind,
          priority: defaultPriority,
          labels: defaultLabels,
          override_limit: overrideLimit,
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        if (response.status === 403) {
          setError(
            `Limit exceeded: ${errorText}\n\nCheck "Override limit" to proceed.`
          );
        } else {
          setError(`Failed to parse: ${errorText}`);
        }
        setView("error");
        return;
      }

      const data: ParseBulkResponse = await response.json();
      setParsedDrafts(data.drafts);

      // Select all drafts by default
      setSelectedDrafts(new Set(data.drafts.map((_, i) => i)));
      setEditedDrafts(new Map());

      setView("preview");
    } catch (e) {
      setError(`Network error: ${e}`);
      setView("error");
    }
  }, [
    input,
    project,
    defaultKind,
    defaultPriority,
    defaultLabels,
    overrideLimit,
  ]);

  const handleSubmit = useCallback(async () => {
    if (selectedDrafts.size === 0) {
      setError("No drafts selected for creation");
      setView("error");
      return;
    }

    setView("submitting");

    // Build the drafts to submit, applying any edits
    const draftsToSubmit: BulkDraftItem[] = [];
    for (const index of selectedDrafts) {
      const edited = editedDrafts.get(index);
      const original = parsedDrafts[index];
      draftsToSubmit.push({
        title: edited?.title || original.title,
        description: edited?.description || original.description,
        kind: edited?.kind || original.kind,
        priority: edited?.priority ?? original.priority,
        labels: edited?.labels ?? original.labels,
      });
    }

    try {
      const response = await fetch("/api/bulk/submit", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          project,
          drafts: draftsToSubmit,
          override_limit: overrideLimit,
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        setError(`Failed to submit: ${errorText}`);
        setView("error");
        return;
      }

      const data: SubmitBulkResponse = await response.json();
      setSubmitResult(data);

      if (data.created > 0 && onSuccess) {
        onSuccess(data.draft_ids);
      }

      setView("result");
    } catch (e) {
      setError(`Network error: ${e}`);
      setView("error");
    }
  }, [
    selectedDrafts,
    editedDrafts,
    parsedDrafts,
    project,
    overrideLimit,
    onSuccess,
  ]);

  const toggleDraftSelection = (index: number) => {
    const newSelected = new Set(selectedDrafts);
    if (newSelected.has(index)) {
      newSelected.delete(index);
    } else {
      newSelected.add(index);
    }
    setSelectedDrafts(newSelected);
  };

  const toggleAllSelection = () => {
    if (selectedDrafts.size === parsedDrafts.length) {
      setSelectedDrafts(new Set());
    } else {
      setSelectedDrafts(new Set(parsedDrafts.map((_, i) => i)));
    }
  };

  const updateDraft = (index: number, updates: Partial<ParsedDraft>) => {
    const draft = parsedDrafts[index];
    const updated = { ...draft, ...updates };
    const newEdited = new Map(editedDrafts);
    newEdited.set(index, updated);
    setEditedDrafts(newEdited);
  };

  const reset = () => {
    setInput("");
    setParsedDrafts([]);
    setSelectedDrafts(new Set());
    setEditedDrafts(new Map());
    setSubmitResult(null);
    setError(null);
    setView("input");
  };

  const close = () => {
    reset();
    onClose?.();
  };

  return (
    <div className="bulk-create-panel">
      {view === "input" && (
        <>
          <div className="bulk-create-preview-header">
            <h3>Bulk Create Drafts</h3>
            <button onClick={close} className="close-button">
              ×
            </button>
          </div>

          <div className="bulk-create-row">
            <div className="bulk-create-half">
              <label className="bulk-create-field">Project</label>
              <input
                type="text"
                value={project}
                disabled
                className="bulk-create-textarea"
              />
            </div>
            <div className="bulk-create-half">
              <label className="bulk-create-field">Default Kind</label>
              <select
                value={defaultKind}
                onChange={(e) => setDefaultKind(e.target.value)}
                className="bulk-create-textarea"
              >
                <option value="task">Task</option>
                <option value="fix">Fix</option>
                <option value="review">Review</option>
                <option value="investigation">Investigation</option>
                <option value="genesis">Genesis</option>
              </select>
            </div>
          </div>

          <div className="bulk-create-field">
            <label className="bulk-create-field">Input (Markdown or Bullet List)</label>
            <textarea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder="## Task One&#10;Description for task one&#10;&#10;## Task Two&#10;Description for task two&#10;&#10;Or use bullet lists:&#10;- Task one&#10;- Task two&#10;- Task three"
              className="bulk-create-textarea"
              rows={15}
            />
            <div className="bulk-create-hint">
              Supports: Markdown headers (##), bullet lists (-), numbered lists
              (1.), task lists (- [ ])
            </div>
          </div>

          <div className="bulk-create-row">
            <label className="bulk-create-checkbox-label">
              <input
                type="checkbox"
                checked={overrideLimit}
                onChange={(e) => setOverrideLimit(e.target.checked)}
              />
              Override 50-draft limit (requires confirmation)
            </label>
          </div>

          <div className="bulk-create-preview-actions">
            <button onClick={close} className="secondary-button">
              Cancel
            </button>
            <button onClick={handleParse} className="primary-button">
              Parse & Preview
            </button>
          </div>
        </>
      )}

      {view === "preview" && (
        <>
          <div className="bulk-create-preview-header">
            <h3>Preview Drafts ({parsedDrafts.length})</h3>
            <button onClick={reset} className="close-button">
              ×
            </button>
          </div>

          <div className="bulk-create-preview-actions">
            <span className="bulk-create-selected-count">
              {selectedDrafts.size} of {parsedDrafts.length} selected
            </span>
            <button onClick={toggleAllSelection} className="secondary-button">
              {selectedDrafts.size === parsedDrafts.length
                ? "Deselect All"
                : "Select All"}
            </button>
          </div>

          <div className="bulk-create-drafts-list">
            {parsedDrafts.map((draft, index) => {
              const edited = editedDrafts.get(index);
              const displayDraft = edited || draft;
              const isSelected = selectedDrafts.has(index);

              return (
                <div
                  key={index}
                  className={`bulk-create-draft-card ${
                    isSelected ? "selected" : ""
                  } ${edited ? "edited" : ""}`}
                >
                  <div className="bulk-create-draft-checkbox">
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => toggleDraftSelection(index)}
                    />
                  </div>
                  <div className="bulk-create-draft-content">
                    <div className="bulk-create-draft-header">
                      <span className="draft-index">#{draft.index}</span>
                      <select
                        value={displayDraft.kind}
                        onChange={(e) =>
                          updateDraft(index, { kind: e.target.value })
                        }
                        className="bulk-create-draft-kind"
                      >
                        <option value="task">Task</option>
                        <option value="fix">Fix</option>
                        <option value="review">Review</option>
                        <option value="investigation">Investigation</option>
                        <option value="genesis">Genesis</option>
                      </select>
                    </div>
                    <input
                      type="text"
                      value={displayDraft.title}
                      onChange={(e) =>
                        updateDraft(index, { title: e.target.value })
                      }
                      className="bulk-create-draft-title-input"
                      placeholder="Draft title"
                    />
                    <textarea
                      value={displayDraft.description}
                      onChange={(e) =>
                        updateDraft(index, { description: e.target.value })
                      }
                      className="bulk-create-draft-desc-input"
                      placeholder="Description (optional)"
                      rows={2}
                    />
                  </div>
                </div>
              );
            })}
          </div>

          <div className="bulk-create-preview-actions">
            <button onClick={reset} className="secondary-button">
              Back
            </button>
            <button
              onClick={handleSubmit}
              className="primary-button"
              disabled={selectedDrafts.size === 0}
            >
              Create {selectedDrafts.size} Draft{selectedDrafts.size !== 1 ? "s" : ""}
            </button>
          </div>
        </>
      )}

      {view === "submitting" && (
        <div className="bulk-create-submitting">
          <div className="bulk-create-spinner" />
          <p>Creating {selectedDrafts.size} draft{selectedDrafts.size !== 1 ? "s" : ""}...</p>
        </div>
      )}

      {view === "result" && submitResult && (
        <>
          <div className="bulk-create-preview-header">
            <h3>Results</h3>
            <button onClick={close} className="close-button">
              ×
            </button>
          </div>

          <div className="bulk-create-summary">
            <div
              className={`bulk-create-summary-item ${
                submitResult.created > 0 ? "success" : ""
              }`}
            >
              <span className="bulk-create-summary-label">Created</span>
              <span className="bulk-create-summary-value">
                {submitResult.created}
              </span>
            </div>
            <div
              className={`bulk-create-summary-item ${
                submitResult.failed > 0 ? "error" : ""
              }`}
            >
              <span className="bulk-create-summary-label">Failed</span>
              <span className="bulk-create-summary-value">
                {submitResult.failed}
              </span>
            </div>
          </div>

          {submitResult.errors.length > 0 && (
            <div className="bulk-create-failures">
              <h4>Errors</h4>
              {submitResult.errors.map((error, i) => (
                <div key={i} className="bulk-create-failure-item">
                  <span>{error}</span>
                </div>
              ))}
            </div>
          )}

          <div className="bulk-create-preview-actions">
            <button onClick={close} className="primary-button">
              Done
            </button>
          </div>
        </>
      )}

      {view === "error" && (
        <>
          <div className="bulk-create-preview-header">
            <h3>Error</h3>
            <button onClick={close} className="close-button">
              ×
            </button>
          </div>

          <div className="bulk-create-failures">
            <p style={{ whiteSpace: "pre-wrap" }}>{error}</p>
          </div>

          <div className="bulk-create-preview-actions">
            <button onClick={reset} className="secondary-button">
              Back
            </button>
            <button onClick={close} className="primary-button">
              Close
            </button>
          </div>
        </>
      )}
    </div>
  );
}

export default BulkCreatePanel;
