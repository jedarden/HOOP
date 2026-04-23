import { atom } from 'jotai';

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

// In-flight streaming content (separate atom for isolation)
export interface StreamingContent {
  conversation_id: string;
  content: string;
  timestamp: number;
}

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
}

// Config error details
export interface ConfigError {
  message: string;
  line: number;
  col: number;
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

// WebSocket event from backend
export interface WsEvent {
  type: 'worker_update' | 'workers_snapshot' | 'beads_snapshot' | 'conversations_snapshot' | 'conversation_update' | 'streaming_content' | 'config_status' | 'projects_snapshot' | 'capacity_snapshot';
  worker?: WorkerData;
  workers?: WorkerData[];
  beads?: BeadData[];
  conversations?: Conversation[];
  conversation?: Conversation;
  streaming?: { conversation_id: string; content: string; timestamp: number };
  projects?: ProjectCardData[];
  config_status?: ConfigStatus;
  capacity?: AccountCapacity[];
}

// Cost bucket from backend aggregation
export interface CostBucket {
  date: string;
  project: string;
  adapter: string;
  model: string;
  strand: string | null;
  usage: MessageUsage;
  request_count: number;
  cost_usd: number;
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
  last_activity_at: string;
  created_at: string;
  audio_filename: string;
}

// Atoms for state management
export const conversationsAtom = atom<Conversation[]>([]);
export const streamingContentAtom = atom<Map<string, StreamingContent>>(new Map());
export const selectedConversationIdAtom = atom<string | null>(null);
export const projectsAtom = atom<Project[]>([]);
export const projectCardsAtom = atom<ProjectCardData[]>([]);
export const stitchesAtom = atom<Stitch[]>([]);
export const workersAtom = atom<WorkerData[]>([]);
export const beadsAtom = atom<BeadData[]>([]);
export const beadEventsAtom = atom<Map<string, BeadEventFromEvents[]>>(new Map()); // bead_id -> events
export const wsConnectedAtom = atom<boolean>(false);
export const configStatusAtom = atom<ConfigStatus>({ valid: true });
export const capacityAtom = atom<AccountCapacity[]>([]);
export const costBucketsAtom = atom<CostBucket[]>([]);
export const dictatedNotesAtom = atom<Map<string, NoteSummary[]>>(new Map()); // project -> notes

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
