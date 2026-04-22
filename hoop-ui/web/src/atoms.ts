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

// A single message in a session
export interface SessionMessage {
  role: 'user' | 'assistant' | 'system';
  content: string | { [key: string]: any } | null;
  usage?: MessageUsage;
  timestamp?: string;
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

// WebSocket event from backend
export interface WsEvent {
  type: 'worker_update' | 'workers_snapshot' | 'beads_snapshot' | 'conversations_snapshot' | 'conversation_update' | 'streaming_content';
  worker?: WorkerData;
  workers?: WorkerData[];
  beads?: BeadData[];
  conversations?: Conversation[];
  conversation?: Conversation;
  streaming?: { conversation_id: string; content: string; timestamp: number };
}

// Atoms for state management
export const conversationsAtom = atom<Conversation[]>([]);
export const streamingContentAtom = atom<Map<string, StreamingContent>>(new Map());
export const selectedConversationIdAtom = atom<string | null>(null);
export const projectsAtom = atom<Project[]>([]);
export const stitchesAtom = atom<Stitch[]>([]);
export const workersAtom = atom<WorkerData[]>([]);
export const beadsAtom = atom<BeadData[]>([]);
export const wsConnectedAtom = atom<boolean>(false);

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
