import { useEffect, useRef } from 'react';
import { useAtomValue } from 'jotai';
import { activeProjectNameAtom, selectedConversationIdAtom, presenceVisibilityAtom } from './atoms';

interface PresenceHeartbeatOptions {
  enabled?: boolean;
  interval?: number; // milliseconds, default 15000 (15s)
}

export function usePresenceHeartbeat({ enabled = true, interval = 15000 }: PresenceHeartbeatOptions = {}) {
  const activeProject = useAtomValue(activeProjectNameAtom);
  const selectedConversationId = useAtomValue(selectedConversationIdAtom);
  const visibility = useAtomValue(presenceVisibilityAtom);
  const heartbeatTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (!enabled) return;

    // Send initial presence immediately
    const sendPresence = async () => {
      const body = {
        project: activeProject || null,
        stitch_id: selectedConversationId || null,
        visibility,
      };

      try {
        const response = await fetch('/api/presence', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(body),
        });
        if (!response.ok) {
          console.warn('Failed to send presence heartbeat:', await response.text());
        }
      } catch (e) {
        console.warn('Failed to send presence heartbeat:', e);
      }
    };

    // Send initial presence
    sendPresence();

    // Set up heartbeat interval
    heartbeatTimerRef.current = setInterval(() => {
      sendPresence();
    }, interval);

    // Clean up on unmount or when dependencies change
    return () => {
      if (heartbeatTimerRef.current) {
        clearInterval(heartbeatTimerRef.current);
        heartbeatTimerRef.current = null;
      }

      // Remove presence when leaving
      fetch('/api/presence', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          project: activeProject || null,
          stitch_id: selectedConversationId || null,
        }),
      }).catch(() => {
        // Best-effort cleanup, ignore errors
      });
    };
  }, [activeProject, selectedConversationId, visibility, enabled, interval]);
}
