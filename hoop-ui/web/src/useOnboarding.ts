import { useEffect, useCallback } from 'react';
import { useAtom } from 'jotai';
import {
  onboardingPromptsAtom,
  onboardingPromptsLoadedAtom,
  onboardingPromptsErrorAtom,
  OnboardingPrompt,
  OnboardingPromptsResponse,
} from './atoms';

// Fetch onboarding prompts from the server
async function fetchOnboardingPrompts(): Promise<OnboardingPromptsResponse | null> {
  try {
    const response = await fetch('/api/onboarding/prompts');
    if (!response.ok) {
      console.warn('Failed to fetch onboarding prompts:', response.statusText);
      return null;
    }
    return await response.json();
  } catch (e) {
    console.warn('Failed to fetch onboarding prompts:', e);
    return null;
  }
}

// Dismiss a prompt
async function dismissPromptOnServer(promptId: string): Promise<boolean> {
  try {
    const response = await fetch('/api/onboarding/dismiss', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt_id: promptId }),
    });
    if (!response.ok) {
      console.warn('Failed to dismiss prompt:', response.statusText);
      return false;
    }
    return true;
  } catch (e) {
    console.warn('Failed to dismiss prompt:', e);
    return false;
  }
}

// Enable/disable prompts globally
async function setPromptsEnabledOnServer(enabled: boolean): Promise<boolean> {
  try {
    const response = await fetch('/api/onboarding/enable', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(enabled),
    });
    if (!response.ok) {
      console.warn('Failed to set prompts enabled:', response.statusText);
      return false;
    }
    return true;
  } catch (e) {
    console.warn('Failed to set prompts enabled:', e);
    return false;
  }
}

// Record feature usage
async function recordFeatureUsageOnServer(feature: string): Promise<boolean> {
  try {
    const response = await fetch('/api/onboarding/record-usage', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ feature }),
    });
    if (!response.ok) {
      console.warn('Failed to record feature usage:', response.statusText);
      return false;
    }
    return true;
  } catch (e) {
    console.warn('Failed to record feature usage:', e);
    return false;
  }
}

// Acknowledge current version
async function acknowledgeVersionOnServer(): Promise<boolean> {
  try {
    const response = await fetch('/api/onboarding/ack-version', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) {
      console.warn('Failed to acknowledge version:', response.statusText);
      return false;
    }
    return true;
  } catch (e) {
    console.warn('Failed to acknowledge version:', e);
    return false;
  }
}

// Hook for onboarding prompts
export function useOnboarding() {
  const [promptsResponse, setPromptsResponse] = useAtom(onboardingPromptsAtom);
  const [isLoaded, setIsLoaded] = useAtom(onboardingPromptsLoadedAtom);
  const [error, setError] = useAtom(onboardingPromptsErrorAtom);

  // Fetch prompts on mount
  useEffect(() => {
    let mounted = true;

    fetchOnboardingPrompts().then((response) => {
      if (!mounted) return;

      if (response) {
        setPromptsResponse(response);
        setIsLoaded(true);
      } else {
        setError('Failed to load prompts');
        setIsLoaded(true);
      }
    });

    return () => {
      mounted = false;
    };
  }, [setPromptsResponse, setIsLoaded, setError]);

  // Refetch prompts (e.g., after dismissing or recording usage)
  const refetch = useCallback(() => {
    fetchOnboardingPrompts().then((response) => {
      if (response) {
        setPromptsResponse(response);
        setError(null);
      } else {
        setError('Failed to load prompts');
      }
    });
  }, [setPromptsResponse, setError]);

  // Dismiss a prompt
  const dismissPrompt = useCallback(
    async (promptId: string) => {
      const success = await dismissPromptOnServer(promptId);
      if (success) {
        // Refetch to get updated state
        refetch();
      }
      return success;
    },
    [refetch]
  );

  // Enable/disable prompts globally
  const setPromptsEnabled = useCallback(
    async (enabled: boolean) => {
      const success = await setPromptsEnabledOnServer(enabled);
      if (success) {
        refetch();
      }
      return success;
    },
    [refetch]
  );

  // Record feature usage (to prevent future intro prompts)
  const recordFeatureUsage = useCallback(
    async (feature: 'agent' | 'mic' | 'patterns' | 'reflection_ledger') => {
      return await recordFeatureUsageOnServer(feature);
    },
    []
  );

  // Acknowledge current version (after showing what's new)
  const acknowledgeVersion = useCallback(async () => {
    const success = await acknowledgeVersionOnServer();
    if (success) {
      refetch();
    }
    return success;
  }, [refetch]);

  // Get active prompts (not dismissed, if prompts are enabled)
  const activePrompts = promptsResponse?.prompts_enabled
    ? promptsResponse?.prompts ?? []
    : [];

  return {
    promptsResponse,
    activePrompts,
    isLoaded,
    error,
    dismissPrompt,
    setPromptsEnabled,
    recordFeatureUsage,
    acknowledgeVersion,
    refetch,
  };
}

// Hook for a specific prompt type
export function useOnboardingPrompt(promptId: string) {
  const { activePrompts, dismissPrompt } = useOnboarding();

  const prompt = activePrompts.find((p) => p.id === promptId);

  const dismiss = useCallback(async () => {
    if (prompt) {
      return await dismissPrompt(prompt.id);
    }
    return false;
  }, [prompt, dismissPrompt]);

  return { prompt, dismiss };
}
