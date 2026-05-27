import { useEffect, useRef } from 'react';
import { useSetAtom } from 'jotai';
import {
  workersAtom,
  beadsAtom,
  conversationsAtom,
  setStreamingContentAction,
  clearStreamingContentAction,
  clearAllStreamingAction,
  wsConnectedAtom,
  connectionStatusAtom,
  reconnectAttemptAtom,
  reconnectDelayAtom,
  configStatusAtom,
  projectCardsAtom,
  projectsReceivedAtom,
  capacityAtom,
  stitchCreatedAtom,
  agentSessionStatusAtom,
  agentInflightAtom,
  agentChatMessagesAtom,
  stuckAlertsAtom,
  saturationAlertsAtom,
  costAnomalyAlertsAtom,
  optimisticStubsAtom,
  WsEvent,
  AgentChatMessage,
  AgentToolCallInProgress,
} from './atoms';

const WS_URL = `ws://${window.location.host}/ws`;

// Exponential backoff sequence: 1s, 2s, 5s, 10s, 30s (max)
const BACKOFF_DELAYS = [1000, 2000, 5000, 10000, 30000];

export function useWebSocket() {
  const setWorkers = useSetAtom(workersAtom);
  const setBeads = useSetAtom(beadsAtom);
  const setConversations = useSetAtom(conversationsAtom);
  const dispatchSetStreaming = useSetAtom(setStreamingContentAction);
  const dispatchClearStreaming = useSetAtom(clearStreamingContentAction);
  const dispatchClearAllStreaming = useSetAtom(clearAllStreamingAction);
  const setConnected = useSetAtom(wsConnectedAtom);
  const setConnectionStatus = useSetAtom(connectionStatusAtom);
  const setReconnectAttempt = useSetAtom(reconnectAttemptAtom);
  const setReconnectDelay = useSetAtom(reconnectDelayAtom);
  const setConfigStatus = useSetAtom(configStatusAtom);
  const setProjectCards = useSetAtom(projectCardsAtom);
  const setProjectsReceived = useSetAtom(projectsReceivedAtom);
  const setCapacity = useSetAtom(capacityAtom);
  const setStitchCreated = useSetAtom(stitchCreatedAtom);
  const setAgentSessionStatus = useSetAtom(agentSessionStatusAtom);
  const setAgentInflight = useSetAtom(agentInflightAtom);
  const setAgentChatMessages = useSetAtom(agentChatMessagesAtom);
  const setStuckAlerts = useSetAtom(stuckAlertsAtom);
  const setSaturationAlerts = useSetAtom(saturationAlertsAtom);
  const setCostAnomalyAlerts = useSetAtom(costAnomalyAlertsAtom);
  const setOptimisticStubs = useSetAtom(optimisticStubsAtom);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const attemptRef = useRef<number>(0);

  // Track in-flight text and tool calls via ref so turn_complete can finalize without
  // needing to read the atom (which requires an extra subscription).
  const inflightRef = useRef<{
    session_id: string;
    text: string;
    tool_calls: AgentToolCallInProgress[];
    started_at: number;
  } | null>(null);

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
        setConnectionStatus('connected');
        setReconnectAttempt(0);
        setReconnectDelay(BACKOFF_DELAYS[0]);
        attemptRef.current = 0;
        if (reconnectTimeoutRef.current) {
          clearTimeout(reconnectTimeoutRef.current);
          reconnectTimeoutRef.current = undefined;
        }
      };

      ws.onmessage = (event) => {
        if (!mounted) return;
        try {
          const data: WsEvent = JSON.parse(event.data);

          if (data.type === 'init') {
            // Epoch-sync: wipe all atom stores on init (reconnect).
            // Server is the source of truth — client rebuilds from snapshot events.
            // Only optimistic stubs survive (they represent un-sent client mutations).
            setBeads([]);
            setConversations([]);
            setWorkers([]);
            setProjectCards([]);
            setCapacity([]);
            // Note: configStatusAtom is NOT reset on init — server sends current status separately
            // This preserves error banners across WebSocket reconnections (§17.5)
            setStitchCreated([]);
            setAgentSessionStatus(null);
            setAgentInflight(null);
            setStuckAlerts(new Map());
            // Note: agentChatMessagesAtom is NOT cleared — committed messages persist
            // Note: optimisticStubsAtom is NOT cleared — pending mutations survive
          } else if (data.type === 'workers_snapshot' && data.workers) {
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
            // Clear all streaming buffers before accepting the authoritative snapshot.
            // This prevents stale partial tokens from persisting across reconnects.
            dispatchClearAllStreaming();
            setConversations(data.conversations);
          } else if (data.type === 'conversation_update' && data.conversation) {
            // Clear this conversation's streaming buffer — the authoritative message
            // is now in the committed store. Buffer must not outlive its turn.
            dispatchClearStreaming(data.conversation.id);
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
            dispatchSetStreaming({
              conversationId: data.streaming.conversation_id,
              content: data.streaming.content,
            });
          } else if (data.type === 'config_status' && data.config_status) {
            setConfigStatus(data.config_status);
          } else if (data.type === 'projects_snapshot' && data.projects) {
            setProjectCards(data.projects);
            setProjectsReceived(true);
          } else if (data.type === 'capacity_snapshot' && data.capacity) {
            setCapacity(data.capacity);
          } else if (data.type === 'stitch_created' && data.stitch_created) {
            setStitchCreated((prev) => [...prev.slice(-49), data.stitch_created!]);
          } else if (data.type === 'agent_session' && data.agent_session) {
            const evt = data.agent_session;

            if (evt.type === 'session_spawned' || evt.type === 'session_reattached') {
              setAgentSessionStatus((prev) => ({
                active: true,
                enabled: true,
                session_id: evt.session_id,
                adapter: evt.adapter,
                model: evt.model,
                stitch_id: prev?.stitch_id ?? null,
                cost_usd: prev?.cost_usd ?? 0,
                input_tokens: prev?.input_tokens ?? 0,
                output_tokens: prev?.output_tokens ?? 0,
                turn_count: prev?.turn_count ?? 0,
                created_at: prev?.created_at ?? null,
                last_activity_at: new Date().toISOString(),
                age_secs: null,
              }));
              // Reset inflight on new session
              inflightRef.current = null;
              setAgentInflight(null);

            } else if (evt.type === 'text_delta') {
              const prev = inflightRef.current;
              const isCurrentSession = prev?.session_id === evt.session_id;
              inflightRef.current = {
                session_id: evt.session_id,
                text: (isCurrentSession ? prev!.text : '') + evt.text,
                tool_calls: isCurrentSession ? prev!.tool_calls : [],
                started_at: isCurrentSession ? prev!.started_at : Date.now(),
              };
              setAgentInflight({ ...inflightRef.current });

            } else if (evt.type === 'tool_use') {
              const prev = inflightRef.current;
              const newTool: AgentToolCallInProgress = {
                id: evt.id,
                name: evt.name,
                input: evt.input,
                status: 'pending',
              };
              inflightRef.current = {
                session_id: evt.session_id,
                text: prev?.session_id === evt.session_id ? prev!.text : '',
                tool_calls: [
                  ...(prev?.session_id === evt.session_id ? prev!.tool_calls : []),
                  newTool,
                ],
                started_at: prev?.started_at ?? Date.now(),
              };
              setAgentInflight({ ...inflightRef.current });

            } else if (evt.type === 'tool_result') {
              if (inflightRef.current?.session_id === evt.session_id) {
                const updatedTools = inflightRef.current.tool_calls.map((tc) =>
                  tc.id === evt.id
                    ? { ...tc, output: evt.output, is_error: evt.is_error, status: 'complete' as const }
                    : tc
                );
                inflightRef.current = { ...inflightRef.current, tool_calls: updatedTools };
                setAgentInflight({ ...inflightRef.current });
              }

            } else if (evt.type === 'turn_complete') {
              // Finalize the in-flight response as a completed assistant message
              if (inflightRef.current && (
                inflightRef.current.text.length > 0 ||
                inflightRef.current.tool_calls.length > 0
              )) {
                const finalMsg: AgentChatMessage = {
                  id: crypto.randomUUID(),
                  role: 'assistant',
                  content: inflightRef.current.text,
                  tool_calls: inflightRef.current.tool_calls.length > 0
                    ? inflightRef.current.tool_calls
                    : undefined,
                  timestamp: Date.now(),
                  session_id: inflightRef.current.session_id,
                };
                setAgentChatMessages((prev) => [...prev, finalMsg]);
              }
              inflightRef.current = null;
              setAgentInflight(null);

              // Update session cost/token counters
              setAgentSessionStatus((prev) => prev ? {
                ...prev,
                cost_usd: prev.cost_usd + evt.cost_usd,
                input_tokens: prev.input_tokens + evt.input_tokens,
                output_tokens: prev.output_tokens + evt.output_tokens,
                turn_count: prev.turn_count + 1,
                last_activity_at: new Date().toISOString(),
              } : null);

            } else if (evt.type === 'session_archived') {
              setAgentSessionStatus((prev) => prev ? {
                ...prev,
                active: false,
                session_id: null,
              } : null);
              inflightRef.current = null;
              setAgentInflight(null);

            } else if (evt.type === 'error') {
              console.error('Agent session error:', evt.message);
              inflightRef.current = null;
              setAgentInflight(null);
            }
          } else if (data.type === 'stuck_alert' && data.stuck_alert) {
            setStuckAlerts((prev) => {
              const updated = new Map(prev);
              updated.set(data.stuck_alert!.worker, data.stuck_alert!);
              return updated;
            });
          } else if (data.type === 'saturation_alert' && data.saturation_alert) {
            setSaturationAlerts((prev) => {
              const updated = new Map(prev);
              updated.set(data.saturation_alert!.alert_id, data.saturation_alert!);
              return updated;
            });
          } else if (data.type === 'cost_anomaly_alert' && data.cost_anomaly_alert) {
            setCostAnomalyAlerts((prev) => {
              const updated = new Map(prev);
              updated.set(data.cost_anomaly_alert!.alert_id, data.cost_anomaly_alert!);
              return updated;
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
        setConnectionStatus('disconnected');
        wsRef.current = null;
        // Clear all streaming buffers on disconnect — partial tokens must not
        // persist into the committed store when the connection is re-established.
        dispatchClearAllStreaming();
        // Clear agent inflight so partial turns don't linger in the UI.
        // Committed messages are never at risk (they are only written on turn_complete).
        inflightRef.current = null;
        setAgentInflight(null);

        // Calculate next backoff delay (capped at 30s)
        attemptRef.current = Math.min(attemptRef.current + 1, BACKOFF_DELAYS.length - 1);
        const delay = BACKOFF_DELAYS[attemptRef.current];
        setReconnectAttempt(attemptRef.current);
        setReconnectDelay(delay);

        reconnectTimeoutRef.current = setTimeout(() => {
          if (mounted) {
            setConnectionStatus('connecting');
            connect();
          }
        }, delay);
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
  }, [setWorkers, setBeads, setConversations, dispatchSetStreaming, dispatchClearStreaming, dispatchClearAllStreaming, setConnected, setConnectionStatus, setReconnectAttempt, setReconnectDelay, setConfigStatus, setProjectCards, setProjectsReceived, setCapacity, setStitchCreated, setAgentSessionStatus, setAgentInflight, setAgentChatMessages, setStuckAlerts, setSaturationAlerts, setCostAnomalyAlerts, setOptimisticStubs]);
}
