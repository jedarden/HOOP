import { useEffect, useRef } from 'react';
import { useSetAtom } from 'jotai';
import { workersAtom, wsConnectedAtom, WsEvent } from './atoms';

const WS_URL = `ws://${window.location.host}/ws`;

export function useWebSocket() {
  const setWorkers = useSetAtom(workersAtom);
  const setConnected = useSetAtom(wsConnectedAtom);
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
  }, [setWorkers, setConnected]);
}
