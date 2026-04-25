import { useAtom } from 'jotai';
import { useEffect, useCallback, useRef } from 'react';
import { welcomeTourCompletedAtom, welcomeTourStepAtom, activeProjectNameAtom } from '../atoms';

const WELCOME_TOUR_STORAGE_KEY = 'hoop_welcome_tour_completed';

interface StarterPrompt {
  label: string;
  description: string;
  action: () => void;
}

const STARTER_PROMPTS: StarterPrompt[] = [
  {
    label: 'Dictate a first note',
    description: 'Record a voice note with transcription',
    action: () => {
      const event = new CustomEvent('hoop-start-dictation');
      window.dispatchEvent(event);
    },
  },
  {
    label: 'Register another project',
    description: 'Add a new project to HOOP',
    action: () => {
      const event = new CustomEvent('hoop-register-project');
      window.dispatchEvent(event);
    },
  },
  {
    label: 'Ask the agent something',
    description: 'Start a conversation with the AI',
    action: () => {
      const event = new CustomEvent('hoop-open-agent-chat');
      window.dispatchEvent(event);
    },
  },
];

const WELCOME_STEPS = [
  {
    title: 'Welcome to HOOP',
    content: (
      <>
        <p className="wt-intro">
          HOOP is your AI-powered development companion. Let's take a quick tour.
        </p>
        <div className="wt-concepts">
          <div className="wt-concept">
            <h4>Stitches</h4>
            <p>Conversations and work items — where you collaborate with AI agents to get things done.</p>
          </div>
          <div className="wt-concept">
            <h4>Patterns</h4>
            <p>Reusable workflows that span multiple projects and automate repetitive tasks.</p>
          </div>
        </div>
      </>
    ),
    highlight: null,
  },
  {
    title: 'Your Projects',
    content: (
      <p>
        Each card represents a project with its workers, active stitches, and costs.
        Click any project to dive in.
      </p>
    ),
    highlight: '.project-card-fleet',
  },
  {
    title: 'Quick Actions',
    content: (
      <p>
        Press <kbd>Cmd</kbd> + <kbd>K</kbd> (or <kbd>Ctrl</kbd> + <kbd>K</kbd>) to open the search palette.
        Search projects, beads, and conversations instantly.
      </p>
    ),
    highlight: null,
  },
  {
    title: 'Get Started',
    content: (
      <p>
        You're all set! Choose a quick start action below, or dismiss to explore on your own.
      </p>
    ),
    highlight: null,
  },
];

export function WelcomeTour() {
  const [completed, setCompleted] = useAtom(welcomeTourCompletedAtom);
  const [step, setStep] = useAtom(welcomeTourStepAtom);
  const [activeProject] = useAtom(activeProjectNameAtom);
  const highlightRingRef = useRef<HTMLDivElement>(null);

  const handleDismiss = useCallback(() => {
    setCompleted(true);
    setStep(0);
    // Persist completion to localStorage
    try {
      localStorage.setItem(WELCOME_TOUR_STORAGE_KEY, 'true');
    } catch {}
    // Clean up highlight classes
    document.body.classList.remove('wt-tour-active');
    document.querySelectorAll('.wt-highlighted-element').forEach(el => {
      el.classList.remove('wt-highlighted-element');
    });
  }, [setCompleted, setStep]);

  const handleNext = useCallback(() => {
    if (step < WELCOME_STEPS.length - 1) {
      setStep(step + 1);
    } else {
      handleDismiss();
    }
  }, [step, setStep, handleDismiss]);

  const handleBack = useCallback(() => {
    if (step > 0) {
      setStep(step - 1);
    }
  }, [step, setStep]);

  const handleStarterPrompt = useCallback((action: () => void) => {
    handleDismiss();
    action();
  }, [handleDismiss]);

  // Close on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleDismiss();
      }
    };
    window.addEventListener('keydown', handleEscape);
    return () => window.removeEventListener('keydown', handleEscape);
  }, [handleDismiss]);

  // Handle highlighting
  useEffect(() => {
    const currentStep = WELCOME_STEPS[step];
    const highlightSelector = currentStep.highlight;

    // Clean up previous highlights
    document.body.classList.remove('wt-tour-active');
    document.querySelectorAll('.wt-highlighted-element').forEach(el => {
      el.classList.remove('wt-highlighted-element');
    });

    if (highlightSelector && highlightRingRef.current) {
      // Find the first matching element
      const targetElement = document.querySelector(highlightSelector);
      if (targetElement) {
        document.body.classList.add('wt-tour-active');
        targetElement.classList.add('wt-highlighted-element');

        // Position the highlight ring
        const rect = targetElement.getBoundingClientRect();
        const ring = highlightRingRef.current;
        ring.style.top = `${rect.top - 4}px`;
        ring.style.left = `${rect.left - 4}px`;
        ring.style.width = `${rect.width + 8}px`;
        ring.style.height = `${rect.height + 8}px`;
        ring.style.display = 'block';

        // Scroll into view if needed
        targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
      } else {
        highlightRingRef.current.style.display = 'none';
      }
    } else if (highlightRingRef.current) {
      highlightRingRef.current.style.display = 'none';
    }

    // Cleanup on unmount
    return () => {
      document.body.classList.remove('wt-tour-active');
      document.querySelectorAll('.wt-highlighted-element').forEach(el => {
        el.classList.remove('wt-highlighted-element');
      });
    };
  }, [step]);

  // Don't show if completed or if we're in a project detail view
  if (completed || activeProject) return null;

  const currentStep = WELCOME_STEPS[step];
  const isLastStep = step === WELCOME_STEPS.length - 1;
  const isFirstStep = step === 0;

  return (
    <>
      {/* Highlight ring for spotlight effect */}
      {currentStep.highlight && (
        <div ref={highlightRingRef} className="wt-highlight-ring" style={{ display: 'none' }} />
      )}

      {/* Highlight overlay (dimmed background) */}
      {currentStep.highlight && (
        <div className="wt-highlight-overlay" />
      )}

      {/* Tour panel */}
      <div className="wt-overlay" onClick={handleDismiss}>
        <div
          className="wt-panel"
          onClick={e => e.stopPropagation()}
          role="dialog"
          aria-label="Welcome tour"
          aria-modal="true"
        >
          <div className="wt-header">
            <h2 className="wt-title">{currentStep.title}</h2>
            <button
              className="wt-close"
              onClick={handleDismiss}
              aria-label="Close tour"
            >
              ×
            </button>
          </div>

          <div className="wt-content">
            {currentStep.content}
          </div>

          {isLastStep ? (
            <div className="wt-starter-prompts">
              {STARTER_PROMPTS.map((prompt, idx) => (
                <button
                  key={idx}
                  className="wt-starter-btn"
                  onClick={() => handleStarterPrompt(prompt.action)}
                >
                  <span className="wt-starter-label">{prompt.label}</span>
                  <span className="wt-starter-desc">{prompt.description}</span>
                </button>
              ))}
            </div>
          ) : (
            <div className="wt-footer">
              <div className="wt-step-indicator">
                {WELCOME_STEPS.map((_, idx) => (
                  <div
                    key={idx}
                    className={`wt-step-dot${idx === step ? ' wt-step-dot-active' : ''}${idx < step ? ' wt-step-dot-complete' : ''}`}
                  />
                ))}
              </div>
              <div className="wt-actions">
                {!isFirstStep && (
                  <button className="wt-btn wt-btn-secondary" onClick={handleBack}>
                    Back
                  </button>
                )}
                <button className="wt-btn wt-btn-primary" onClick={handleNext}>
                  {isLastStep ? 'Get Started' : 'Next'}
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </>
  );
}

export function WelcomeTourTrigger() {
  const [, setCompleted] = useAtom(welcomeTourCompletedAtom);
  const setStep = useAtom(welcomeTourStepAtom)[1];

  const handleRestart = useCallback(() => {
    setCompleted(false);
    setStep(0);
  }, [setCompleted, setStep]);

  return (
    <button
      className="wt-restart-trigger"
      onClick={handleRestart}
      title="Show welcome tour again"
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M8 2C4.68629 2 2 4.68629 2 8C2 11.3137 4.68629 14 8 14C11.3137 14 14 11.3137 14 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
        <path d="M14 2V8H8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
      </svg>
      Show Tour
    </button>
  );
}
