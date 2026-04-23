import { useEffect, useRef } from 'react';
import { useSetAtom } from 'jotai';
import {
  workersAtom,
  beadsAtom,
  conversationsAtom,
  streamingContentAtom,
  wsConnectedAtom,
  configStatusAtom,
  projectCardsAtom,
  capacityAtom,
  stitchCreatedAtom,
  agentSessionStatusAtom,
  agentInflightAtom,
  agentChatMessagesAtom,
  WsEvent,
  AgentChatMessage,
  AgentToolCallInProgress,
} from './atoms';

const WS_URL = `ws://${window.location.host}/ws`;

export function useWebSocket() {
  const setWorkers = useSetAtom(workersAtom);
  const setBeads = useSetAtom(beadsAtom);
  const setConversations = useSetAtom(conversationsAtom);
  const setStreamingContent = useSetAtom(streamingContentAtom);
  const setConnected = useSetAtom(wsConnectedAtom);
  const setConfigStatus = useSetAtom(configStatusAtom);
  const setProjectCards = useSetAtom(projectCardsAtom);
  const setCapacity = useSetAtom(capacityAtom);
  const setStitchCreated = useSetAtom(stitchCreatedAtom);
  const setAgentSessionStatus = useSetAtom(agentSessionStatusAtom);
  const setAgentInflight = useSetAtom(agentInflightAtom);
  const setAgentChatMessages = useSetAtom(agentChatMessagesAtom);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

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
          } else if (data.type === 'projects_snapshot' && data.projects) {
            setProjectCards(data.projects);
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
  }, [setWorkers, setBeads, setConversations, setStreamingContent, setConnected, setConfigStatus, setProjectCards, setCapacity, setStitchCreated, setAgentSessionStatus, setAgentInflight, setAgentChatMessages]);
}
