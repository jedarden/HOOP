/**
 * Secrets Warning Banner (§18)
 *
 * Shows a warning banner when secrets are detected in transcripts or attachments.
 * Operator can choose to:
 * - Redact in place (replace secrets with [REDACTED])
 * - Proceed anyway (keep the content as is)
 *
 * This component fetches findings on-demand for a specific stitch_id.
 */

import { useEffect, useState } from 'react';

interface SecretFinding {
  pattern_name: string;
  match_start: number;
  match_len: number;
  matched_text: string;
}

interface SecretsWarningBannerProps {
  /** The stitch_id to check for findings */
  stitchId: string | null;
  /** Callback when user chooses "Proceed anyway" */
  onProceed?: () => void;
  /** Callback when user chooses "Redact" */
  onRedact?: () => void;
  /** Optional: manually set dismissed state */
  dismissed?: boolean;
  /** Optional: callback when banner is dismissed */
  onDismiss?: () => void;
}

export function SecretsWarningBanner({
  stitchId,
  onProceed,
  onRedact,
  dismissed = false,
  onDismiss,
}: SecretsWarningBannerProps) {
  const [findings, setFindings] = useState<SecretFinding[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [internalDismissed, setInternalDismissed] = useState(false);

  // Fetch findings when stitchId changes
  useEffect(() => {
    if (!stitchId || dismissed || internalDismissed) {
      setFindings([]);
      return;
    }

    setLoading(true);
    setError(null);

    fetch(`/api/dictated-notes/${stitchId}/findings`)
      .then(async (res) => {
        if (!res.ok) {
          throw new Error(`HTTP ${res.status}: ${res.statusText}`);
        }
        const data: SecretFinding[] = await res.json();
        setFindings(data);
      })
      .catch((e) => {
        console.error('Failed to fetch secrets findings:', e);
        setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => {
        setLoading(false);
      });
  }, [stitchId, dismissed, internalDismissed]);

  // Don't show if loading, no findings, or dismissed
  if (loading || findings.length === 0 || dismissed || internalDismissed) {
    return null;
  }

  // Show error state but still allow proceeding
  if (error) {
    return (
      <div className="secrets-warning-banner secrets-warning-error" role="alert">
        <div className="banner-content">
          <span className="banner-icon">⚠</span>
          <span className="banner-text">
            Failed to check for secrets: {error}
          </span>
          <button
            className="banner-dismiss"
            onClick={() => {
              setInternalDismissed(true);
              onDismiss?.();
            }}
            aria-label="Dismiss warning"
          >
            ✕
          </button>
        </div>
      </div>
    );
  }

  // Group findings by pattern_name
  const findingsByPattern = findings.reduce((acc, f) => {
    if (!acc[f.pattern_name]) {
      acc[f.pattern_name] = [];
    }
    acc[f.pattern_name].push(f);
    return acc;
  }, {} as Record<string, SecretFinding[]>);

  const patternCount = Object.keys(findingsByPattern).length;

  const handleRedact = () => {
    setInternalDismissed(true);
    onRedact?.();
  };

  const handleProceed = () => {
    setInternalDismissed(true);
    onProceed?.();
  };

  const handleDismiss = () => {
    setInternalDismissed(true);
    onDismiss?.();
  };

  return (
    <div className="secrets-warning-banner" role="alert">
      <div className="banner-content">
        <span className="banner-icon">🔒</span>
        <span className="banner-text">
          <strong>Secrets detected</strong> — {findings.length} potential secret{findings.length !== 1 ? 's' : ''} found
          ({patternCount} pattern{patternCount !== 1 ? 's' : ''})
        </span>
        <div className="banner-actions">
          {onRedact && (
            <button
              className="banner-action banner-action-redact"
              onClick={handleRedact}
              title="Redact secrets from transcript"
            >
              Redact
            </button>
          )}
          <button
            className="banner-action banner-action-proceed"
            onClick={handleProceed}
            title="I understand the risks, proceed anyway"
          >
            Proceed
          </button>
          <button
            className="banner-dismiss"
            onClick={handleDismiss}
            aria-label="Dismiss warning"
            title="Hide this warning"
          >
            ✕
          </button>
        </div>
      </div>

      {/* Expandable details section */}
      <details className="banner-details">
        <summary>View details</summary>
        <div className="banner-findings">
          {Object.entries(findingsByPattern).map(([pattern, items]) => (
            <div key={pattern} className="finding-group">
              <strong className="finding-pattern">{pattern}</strong>
              <ul className="finding-list">
                {items.map((f, idx) => (
                  <li key={idx} className="finding-item">
                    <code className="finding-match">
                      {f.matched_text.length > 50
                        ? f.matched_text.slice(0, 50) + '...'
                        : f.matched_text}
                    </code>
                    <span className="finding-position">at position {f.match_start}</span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </details>
    </div>
  );
}
