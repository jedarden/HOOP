import { useEffect, useRef } from 'react';
import { useSetAtom } from 'jotai';
import { workersAtom, beadsAtom, conversationsAtom, streamingContentAtom, wsConnectedAtom, configStatusAtom, WsEvent } from './atoms';

const WS_URL = `ws://${window.location.host}/ws`;

export function useWebSocket() {
  const setWorkers = useSetAtom(workersAtom);
  const setBeads = useSetAtom(beadsAtom);
  const setConversations = useSetAtom(conversationsAtom);
  const setStreamingContent = useSetAtom(streamingContentAtom);
  const setConnected = useSetAtom(wsConnectedAtom);
  const setConfigStatus = useSetAtom(configStatusAtom);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    let mounted = true;

    function connect() {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        return;
      }

      const ws = new WebSocket(WS_URL);
      wsRef.current = ws;

      ws.onopen = () => {
        if (!mounted) return;
        console.log('WebSocket connected');
        setConnected(true);
        // Clear any pending reconnect
        if (reconnectTimeoutRef.current) {
          clearTimeout(reconnectTimeoutRef.current);
          reconnectTimeoutRef.current = undefined;
        }
      };

      ws.onmessage = (event) => {
        if (!mounted) return;
        try {
          const data: WsEvent = JSON.parse(event.data);

          if (data.type === 'workers_snapshot' && data.workers) {
            setWorkers(data.workers);
          } else if (data.type === 'worker_update' && data.worker) {
            setWorkers((prev) => {
              const idx = prev.findIndex((w) => w.worker === data.worker!.worker);
              if (idx >= 0) {
                const updated = [...prev];
                updated[idx] = data.worker!;
                return updated;
              }
              return [...prev, data.worker!];
            });
          } else if (data.type === 'beads_snapshot' && data.beads) {
            setBeads(data.beads);
          } else if (data.type === 'conversations_snapshot' && data.conversations) {
            setConversations(data.conversations);
          } else if (data.type === 'conversation_update' && data.conversation) {
            setConversations((prev) => {
              const idx = prev.findIndex((c) => c.id === data.conversation!.id);
              if (idx >= 0) {
                const updated = [...prev];
                updated[idx] = data.conversation!;
                return updated;
              }
              return [...prev, data.conversation!];
            });
          } else if (data.type === 'streaming_content' && data.streaming) {
            setStreamingContent((prev) => {
              const next = new Map(prev);
              next.set(data.streaming!.conversation_id, {
                conversation_id: data.streaming!.conversation_id,
                content: data.streaming!.content,
                timestamp: data.streaming!.timestamp,
              });
              return next;
            });
          } else if (data.type === 'config_status' && data.config_status) {
            setConfigStatus(data.config_status);
          }
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      ws.onclose = () => {
        if (!mounted) return;
        console.log('WebSocket disconnected, reconnecting...');
        setConnected(false);
        wsRef.current = null;
        // Reconnect after 2 seconds
        reconnectTimeoutRef.current = setTimeout(() => {
          if (mounted) connect();
        }, 2000);
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    }

    connect();

    return () => {
      mounted = false;
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      wsRef.current?.close();
    };
  }, [setWorkers, setBeads, setConversations, setStreamingContent, setConnected, setConfigStatus]);
}
