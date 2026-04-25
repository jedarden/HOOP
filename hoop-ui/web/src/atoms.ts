import { atom } from 'jotai';
import { atomFamily } from 'jotai/utils';

// Session classification types
export type SessionKind =
  | 'operator'    // Human ↔ agent chat (normal conversation)
  | 'dictated'    // Voice note with Whisper transcript
  | 'worker'      // NEEDLE worker's CLI session (tagged with [needle:...])
  | 'ad-hoc';     // Direct CLI session without prefix tag

// Worker data for worker sessions
export interface WorkerMetadata {
  worker: string;
  bead: string;
  strand: string | null;
}

// Token usage from a single message
export interface MessageUsage {
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_write_tokens: number;
}

// Word-level timestamp for Whisper transcripts
export interface TranscriptWord {
  word: string;
  start: number;  // seconds
  end: number;    // seconds
}

// Transcript data structure
export interface TranscriptData {
  text: string;
  words: TranscriptWord[];
}

// Dictated note metadata from backend (audio + Whisper transcript)
export interface DictatedNote {
  stitch_id: string;
  audio_url: string;
  transcript: string;
  transcript_words: TranscriptWord[];
  duration_secs?: number | null;
  language?: string | null;
  recorded_at: string;
  transcription_status: 'Pending' | 'Completed' | 'Failed';
}

// A single message in a session
export interface SessionMessage {
  role: 'user' | 'assistant' | 'system';
  content: string | { [key: string]: any } | null;
  usage?: MessageUsage;
  timestamp?: string;
  attachments?: string[];
  transcript?: TranscriptData;
}

// Conversation/session data
export interface Conversation {
  id: string;
  session_id: string;
  provider: string;
  kind: SessionKind;
  worker_metadata?: WorkerMetadata;
  cwd: string;
  title: string;
  messages: SessionMessage[];
  total_tokens: number;
  created_at: string;
  updated_at: string;
  complete: boolean;
  file_path: string;
  dictated_note?: DictatedNote | null;
}

// atomFamily keyed by conversation_id — each conversation has its own isolated atom.
// Token deltas on one conversation never trigger re-renders in other conversation views.
// Buffers are cleared on conversation_update (authoritative broadcast) or WS disconnect.
// (jotai/utils atomFamily is deprecated in v2 in favor of jotai-family; still correct for v2.x)
// eslint-disable-next-line deprecation/deprecation
export const streamingContentFamily = atomFamily(
  (_conversationId: string) => atom<string>('')
);

// Tracks which conversation IDs currently have non-empty streaming buffers.
// Allows clearAllStreamingAction to sweep only active entries.
export const streamingActiveIdsAtom = atom<ReadonlySet<string>>(new Set<string>());

// Write-only action: set streaming content for one conversation
export const setStreamingContentAction = atom(
  null,
  (_get, set, { conversationId, content }: { conversationId: string; content: string }) => {
    set(streamingContentFamily(conversationId), content);
    set(streamingActiveIdsAtom, (prev) => {
      if (prev.has(conversationId)) return prev;
      const next = new Set(prev);
      next.add(conversationId);
      return next;
    });
  }
);

// Write-only action: clear one conversation's buffer when the authoritative broadcast arrives
export const clearStreamingContentAction = atom(
  null,
  (_get, set, conversationId: string) => {
    set(streamingContentFamily(conversationId), '');
    set(streamingActiveIdsAtom, (prev) => {
      if (!prev.has(conversationId)) return prev;
      const next = new Set(prev);
      next.delete(conversationId);
      return next;
    });
  }
);

// Write-only action: clear all streaming buffers (WS disconnect or conversations_snapshot)
export const clearAllStreamingAction = atom(
  null,
  (get, set) => {
    const ids = get(streamingActiveIdsAtom);
    if (ids.size === 0) return;
    for (const id of ids) {
      set(streamingContentFamily(id), '');
    }
    set(streamingActiveIdsAtom, new Set<string>());
  }
);

// Legacy interfaces for compatibility
export interface Project {
  name: string;
  path: string;
  activeBeads: number;
  workers: number;
}

// Project card data from backend with runtime state
export interface ProjectCardData {
  name: string;
  label: string;
  color: string;
  path: string;
  degraded: boolean;
  runtime_state?: string;
  runtime_error?: string;
  bead_count: number;
  worker_count: number;
  active_stitch_count: number;
  cost_today: number;
  stuck_count: number;
  last_activity?: string;
}

export interface Stitch {
  id: string;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
}

// Worker state types from the backend
export type WorkerLiveness = 'Live' | 'Hung' | 'Dead';

export type WorkerDisplayState =
  | { state: 'executing'; bead: string; adapter: string; model: string | null }
  | { state: 'idle'; last_strand: string | null }
  | { state: 'knot'; reason: string };

export interface WorkerData {
  worker: string;
  state: WorkerDisplayState;
  liveness: WorkerLiveness;
  last_heartbeat: string;
  heartbeat_age_secs: number;
}

// Bead types from the backend
export type BeadStatus = 'open' | 'closed';
export type BeadType = 'task' | 'bug' | 'epic' | 'unknown';

export interface BeadData {
  id: string;
  title: string;
  status: BeadStatus;
  priority: number;
  issue_type: BeadType;
  created_at: string;
  updated_at: string;
  created_by: string;
  dependencies: string[];
  project: string;
}

// Config error details (§17.5)
export interface ConfigError {
  message: string;
  line: number;
  col: number;
  field?: string;
  expected?: string;
  got?: string;
}

// Config status from backend
export interface ConfigStatus {
  valid: boolean;
  error?: ConfigError;
}

// Per-account capacity data from backend
export interface AccountCapacity {
  account_id: string;
  adapter: string;
  plan_type: string;
  rate_limit_tier: string;
  utilization_5h: number;
  utilization_7d: number;
  resets_at_5h?: string | null;
  resets_at_7d?: string | null;
  tokens_5h: number;
  tokens_7d: number;
  turns_5h: number;
  turns_7d: number;
  burn_rate_per_min: number;
  forecast_full_5h_min?: number | null;
  forecast_full_7d_min?: number | null;
  source: string;
  computed_at: string;
}

// Agent session event (mirrors backend AgentSessionEvent enum, serde tag = "type")
export type AgentSessionEventData =
  | { type: 'session_spawned'; session_id: string; adapter: string; model: string }
  | { type: 'session_reattached'; session_id: string; adapter: string; model: string }
  | { type: 'text_delta'; session_id: string; text: string }
  | { type: 'tool_use'; session_id: string; id: string; name: string; input: unknown }
  | { type: 'tool_result'; session_id: string; id: string; output: unknown; is_error: boolean }
  | { type: 'turn_complete'; session_id: string; cost_usd: number; input_tokens: number; output_tokens: number }
  | { type: 'session_archived'; session_id: string; reason: string }
  | { type: 'error'; session_id: string; message: string };

// Agent session status snapshot from backend
export interface AgentSessionStatus {
  active: boolean;
  enabled: boolean;
  session_id: string | null;
  adapter: string | null;
  model: string | null;
  stitch_id: string | null;
  cost_usd: number;
  input_tokens: number;
  output_tokens: number;
  turn_count: number;
  created_at: string | null;
  last_activity_at: string | null;
  age_secs: number | null;
}

// Tool call state during a live turn
export interface AgentToolCallInProgress {
  id: string;
  name: string;
  input: unknown;
  output?: unknown;
  is_error?: boolean;
  status: 'pending' | 'complete';
}

// In-flight agent response — separate reactive atom for isolation (acceptance: in-flight isolation)
export interface AgentInflight {
  session_id: string;
  text: string;
  tool_calls: AgentToolCallInProgress[];
  started_at: number;
}

// A finalized message in the operator↔agent chat pane
export interface AgentChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  tool_calls?: AgentToolCallInProgress[];
  timestamp: number;
  session_id: string;
  attachments?: string[];
}

// Agent chat scope — which project(s) are in context (empty = cross-project / all)
export interface AgentChatScope {
  projects: string[];
}

// Search palette open/closed state (cmd-K)
export const searchPaletteOpenAtom = atom<boolean>(false);

// WebSocket event from backend
export interface WsEvent {
  type: 'worker_update' | 'workers_snapshot' | 'beads_snapshot' | 'conversations_snapshot' | 'conversation_update' | 'streaming_content' | 'config_status' | 'projects_snapshot' | 'capacity_snapshot' | 'stitch_created' | 'agent_session';
  worker?: WorkerData;
  workers?: WorkerData[];
  beads?: BeadData[];
  conversations?: Conversation[];
  conversation?: Conversation;
  streaming?: { conversation_id: string; content: string; timestamp: number };
  projects?: ProjectCardData[];
  config_status?: ConfigStatus;
  capacity?: AccountCapacity[];
  stitch_created?: StitchCreatedData;
  agent_session?: AgentSessionEventData;
}

// Cost bucket from backend aggregation
export interface CostBucket {
  date: string;
  project: string;
  adapter: string;
  model: string;
  strand: string | null;
  /** "fleet" (NEEDLE worker session) or "operator" (all others). Set at aggregation time. */
  classification: string;
  usage: MessageUsage;
  request_count: number;
  cost_usd: number;
}

/** Filter values for the conversation pane — persisted across renders in a Jotai atom. */
export type ConversationFilter = 'all' | 'fleet' | 'operator' | 'ad-hoc' | 'dictated';

// Stitch created event from backend WS
export interface StitchCreatedData {
  bead_id: string;
  title: string;
  project: string;
  stitch_id: string | null;
  source: string;
  actor: string;
  created_at: string;
}

// Audit log row from fleet.db actions table (§4.5, §13)
export interface AuditRow {
  id: string;
  ts: string;
  actor: string;
  type: string;
  target: string;
  project: string | null;
  args: Record<string, unknown> | null;
  result: string;
  error: string | null;
  schema_version: string;
}

// Response from GET /api/audit
export interface AuditResponse {
  audit_rows: AuditRow[];
  total_count: number;
}

// Response from GET /api/audit/verify
export interface HashChainVerifyResponse {
  valid: boolean;
  message: string;
  row_count: number;
}

// Bead event from events.jsonl for debug panel
export interface BeadEventFromEvents {
  timestamp: string;
  event_type: string;
  bead_id: string;
  worker: string;
  line_number?: number;
  raw: string;
}

// Dictated note summary from REST API
export interface NoteSummary {
  stitch_id: string;
  project: string;
  title: string;
  kind: string;
  recorded_at: string;
  transcribed_at: string;
  duration_secs: number | null;
  language: string | null;
  tags: string[];
  transcript_preview: string;
  /** Full transcript text for search indexing */
  transcript: string;
  last_activity_at: string;
  created_at: string;
  audio_filename: string;
  transcription_status: 'Pending' | 'Completed' | 'Failed';
}

// Cross-project dashboard types
export interface ProjectSpend {
  project: string;
  cost_usd: number;
}

export interface AdapterSpend {
  adapter: string;
  cost_usd: number;
}

export interface ProjectWorkers {
  project: string;
  worker_count: number;
}

export interface LongestRunningStitch {
  project: string;
  bead_id: string;
  title: string;
  created_at: string;
  duration_secs: number;
}

export interface CrossProjectDashboardData {
  range: string;
  range_label: string;
  total_spend_usd: number;
  spend_by_project: ProjectSpend[];
  spend_by_adapter: AdapterSpend[];
  total_workers: number;
  workers_by_project: ProjectWorkers[];
  longest_running_stitches: LongestRunningStitch[];
}

// Atoms for state management
export const conversationsAtom = atom<Conversation[]>([]);
export const selectedConversationIdAtom = atom<string | null>(null);
/** Per-operator conversation filter — persists across re-mounts within the session. */
export const conversationFilterAtom = atom<ConversationFilter>('all');
export const projectsAtom = atom<Project[]>([]);
export const projectCardsAtom = atom<ProjectCardData[]>([]);
export const projectsReceivedAtom = atom<boolean>(false);
export const stitchesAtom = atom<Stitch[]>([]);
export const workersAtom = atom<WorkerData[]>([]);
export const beadsAtom = atom<BeadData[]>([]);
export const beadEventsAtom = atom<Map<string, BeadEventFromEvents[]>>(new Map()); // bead_id -> events
export const wsConnectedAtom = atom<boolean>(false);
export const configStatusAtom = atom<ConfigStatus>({ valid: true });
export const capacityAtom = atom<AccountCapacity[]>([]);
export const costBucketsAtom = atom<CostBucket[]>([]);
export const dictatedNotesAtom = atom<Map<string, NoteSummary[]>>(new Map()); // project -> notes
export const stitchCreatedAtom = atom<StitchCreatedData[]>([]);

// Agent chat atoms
export const agentSessionStatusAtom = atom<AgentSessionStatus | null>(null);
export const agentInflightAtom = atom<AgentInflight | null>(null);
export const agentChatMessagesAtom = atom<AgentChatMessage[]>([]);
export const agentChatScopeAtom = atom<AgentChatScope>({ projects: [] });

// Current time atom — updated every 30s by OverviewPage; used by RelativeTime
// so that time-tick re-renders don't defeat memo on ProjectCard.
export const currentTimeAtom = atom<number>(Date.now());

// Format content for display (handles string and object content)
export function formatContent(content: string | { [key: string]: any } | null): string {
  if (content === null) return '';
  if (typeof content === 'string') return content;
  return JSON.stringify(content, null, 2);
}

// Get badge display for session kind
export function getSessionKindBadge(kind: SessionKind, workerMetadata?: WorkerMetadata): { label: string; className: string } {
  switch (kind) {
    case 'worker':
      return { label: `fleet · ${workerMetadata?.worker || 'unknown'}`, className: 'badge-fleet' };
    case 'operator':
      return { label: 'operator', className: 'badge-operator' };
    case 'dictated':
      return { label: 'dictated', className: 'badge-dictated' };
    case 'ad-hoc':
      return { label: 'ad-hoc', className: 'badge-ad-hoc' };
  }
}

// Get adapter and model from worker state
export function getAdapterAndModel(workers: WorkerData[], workerName: string): { adapter: string; model: string | null } {
  const worker = workers.find(w => w.worker === workerName);
  if (worker?.state.state === 'executing') {
    return { adapter: worker.state.adapter, model: worker.state.model };
  }
  return { adapter: 'cli', model: null };
}
