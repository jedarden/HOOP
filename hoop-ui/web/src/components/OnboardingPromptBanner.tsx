import { useOnboardingPrompt, useOnboarding } from '../useOnboarding';

interface OnboardingPromptBannerProps {
  promptId: string;
  onActionClick?: () => void;
}

export function OnboardingPromptBanner({ promptId, onActionClick }: OnboardingPromptBannerProps) {
  const { prompt, dismiss } = useOnboardingPrompt(promptId);

  if (!prompt) {
    return null;
  }

  const handleDismiss = async () => {
    await dismiss();
  };

  const handleActionClick = () => {
    if (prompt.action_url) {
      if (prompt.action_url.startsWith('/')) {
        window.location.hash = prompt.action_url;
      } else {
        window.open(prompt.action_url, '_blank');
      }
    }
    onActionClick?.();
  };

  return (
    <div className="onboarding-prompt-banner" role="alert" aria-live="polite">
      <div className="onboarding-prompt-content">
        <span className="onboarding-prompt-icon" aria-hidden="true">💡</span>
        <div className="onboarding-prompt-text">
          <h3 className="onboarding-prompt-title">{prompt.title}</h3>
          <p className="onboarding-prompt-message">{prompt.message}</p>
        </div>
        <div className="onboarding-prompt-actions">
          {prompt.action_label && prompt.action_url && (
            <button className="onboarding-prompt-action" onClick={handleActionClick}>
              {prompt.action_label}
            </button>
          )}
          <button className="onboarding-prompt-dismiss" onClick={handleDismiss} aria-label="Dismiss">
            ✕
          </button>
        </div>
      </div>
    </div>
  );
}

interface WhatsNewBannerProps {
  onAcknowledge?: () => void;
}

export function WhatsNewBanner({ onAcknowledge }: WhatsNewBannerProps) {
  const { promptsResponse, acknowledgeVersion } = useOnboarding();

  if (!promptsResponse) {
    return null;
  }

  const { hoop_version, last_seen_version } = promptsResponse;

  // Only show if we've upgraded and haven't acknowledged yet
  if (last_seen_version === hoop_version) {
    return null;
  }

  // Check if there's a whats_new prompt for this version
  const whatsNewPromptId = `whats_new_${hoop_version.replace('.', '_')}`;
  const { prompt, dismiss } = useOnboardingPrompt(whatsNewPromptId);

  if (!prompt) {
    return null;
  }

  const handleAcknowledge = async () => {
    await acknowledgeVersion();
    await dismiss();
    onAcknowledge?.();
  };

  const handleDismiss = async () => {
    await dismiss();
  };

  const handleActionClick = () => {
    if (prompt.action_url) {
      if (prompt.action_url.startsWith('/')) {
        window.location.hash = prompt.action_url;
      } else {
        window.open(prompt.action_url, '_blank');
      }
    }
  };

  return (
    <div className="whats-new-banner" role="alert" aria-live="polite">
      <div className="whats-new-content">
        <span className="whats-new-icon" aria-hidden="true">🎉</span>
        <div className="whats-new-text">
          <h3 className="whats-new-title">{prompt.title}</h3>
          <p className="whats-new-message">{prompt.message}</p>
        </div>
        <div className="whats-new-actions">
          {prompt.action_label && prompt.action_url && (
            <button className="whats-new-action" onClick={handleActionClick}>
              {prompt.action_label}
            </button>
          )}
          <button className="whats-new-ack" onClick={handleAcknowledge}>
            Got it
          </button>
          <button className="whats-new-dismiss" onClick={handleDismiss} aria-label="Dismiss">
            ✕
          </button>
        </div>
      </div>
    </div>
  );
}

// Inline prompt component (for smaller prompts like agent/mic intro)
interface InlinePromptProps {
  promptId: string;
  variant?: 'info' | 'tip';
  onActionClick?: () => void;
  onDismiss?: () => void;
}

export function InlinePrompt({ promptId, variant = 'tip', onActionClick, onDismiss }: InlinePromptProps) {
  const { prompt, dismiss } = useOnboardingPrompt(promptId);

  if (!prompt) {
    return null;
  }

  const handleDismiss = async () => {
    await dismiss();
    onDismiss?.();
  };

  const handleActionClick = () => {
    if (prompt.action_url) {
      window.location.hash = prompt.action_url;
    }
    onActionClick?.();
  };

  return (
    <div className={`inline-prompt inline-prompt--${variant}`} role="alert" aria-live="polite">
      <span className={`inline-prompt-icon inline-prompt-icon--${variant}`} aria-hidden="true">
        {variant === 'info' ? 'ℹ' : '💡'}
      </span>
      <div className="inline-prompt-text">
        <strong className="inline-prompt-title">{prompt.title}</strong>
        <span className="inline-prompt-message">{prompt.message}</span>
        {prompt.action_label && prompt.action_url && (
          <button className="inline-prompt-action-link" onClick={handleActionClick}>
            {prompt.action_label} →
          </button>
        )}
      </div>
      <button
        className="inline-prompt-dismiss"
        onClick={handleDismiss}
        aria-label="Dismiss"
        title="Dismiss"
      >
        ✕
      </button>
    </div>
  );
}
