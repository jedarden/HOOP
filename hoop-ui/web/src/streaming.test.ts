/**
 * Tests for the streamingContentAtom pattern (§B3 / arch-patterns "Streaming separation").
 *
 * Core invariant: streaming token buffers are separate from committed state.
 * Partial tokens must never appear in conversationsAtom or agentChatMessagesAtom.
 * On WS disconnect or turn.complete, buffers drop; committed state is untouched.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { createStore } from 'jotai';
import {
  streamingContentFamily,
  streamingActiveIdsAtom,
  setStreamingContentAction,
  clearStreamingContentAction,
  clearAllStreamingAction,
  conversationsAtom,
  agentInflightAtom,
  agentChatMessagesAtom,
  Conversation,
  AgentInflight,
} from './atoms';

// Minimal conversation fixture — only fields the streaming tests care about
function makeConversation(id: string, committedText = 'committed message'): Conversation {
  return {
    id,
    session_id: `sess-${id}`,
    provider: 'claude',
    kind: 'worker',
    cwd: '/tmp/test',
    title: `Conv ${id}`,
    messages: [{ role: 'assistant', content: committedText }],
    total_tokens: 100,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    complete: false,
    file_path: '/tmp/test.jsonl',
  };
}

describe('streamingContentFamily isolation', () => {
  let store: ReturnType<typeof createStore>;

  beforeEach(() => {
    store = createStore();
  });

  it('streaming content goes into buffer, not into conversationsAtom', () => {
    const conv = makeConversation('conv-1');
    store.set(conversationsAtom, [conv]);

    // Simulate streaming tokens arriving mid-turn
    store.set(setStreamingContentAction, { conversationId: 'conv-1', content: 'partial tok' });

    // Buffer holds the streaming content
    expect(store.get(streamingContentFamily('conv-1'))).toBe('partial tok');

    // Committed messages are unchanged — no partial token leaked in
    const conversations = store.get(conversationsAtom);
    expect(conversations[0].messages).toHaveLength(1);
    expect(conversations[0].messages[0].content).toBe('committed message');
  });

  it('mid-turn WS disconnect: no partial leak into committed store', () => {
    const conv = makeConversation('conv-1');
    store.set(conversationsAtom, [conv]);

    // Simulate partial tokens arriving mid-turn
    store.set(setStreamingContentAction, { conversationId: 'conv-1', content: 'partial partial partial' });
    expect(store.get(streamingContentFamily('conv-1'))).toBe('partial partial partial');

    // Simulate WS disconnect — ws.onclose calls clearAllStreamingAction
    store.set(clearAllStreamingAction);

    // Buffer is cleared
    expect(store.get(streamingContentFamily('conv-1'))).toBe('');

    // Committed state is still intact — no partial tokens written into it
    const conversations = store.get(conversationsAtom);
    expect(conversations[0].messages).toHaveLength(1);
    expect(conversations[0].messages[0].content).toBe('committed message');
    expect(conversations[0].messages[0].content).not.toContain('partial');
  });

  it('authoritative broadcast clears buffer for that conversation', () => {
    const conv = makeConversation('conv-1');
    store.set(conversationsAtom, [conv]);
    store.set(setStreamingContentAction, { conversationId: 'conv-1', content: 'streaming...' });
    expect(store.get(streamingContentFamily('conv-1'))).toBe('streaming...');

    // Simulate conversation_update (server sends authoritative completed message)
    const updated = makeConversation('conv-1', 'streaming...'); // now committed
    store.set(clearStreamingContentAction, 'conv-1');
    store.set(conversationsAtom, [updated]);

    // Buffer is gone
    expect(store.get(streamingContentFamily('conv-1'))).toBe('');
  });

  it('buffer isolation: one conversation streaming does not affect another', () => {
    const c1 = makeConversation('conv-1');
    const c2 = makeConversation('conv-2');
    store.set(conversationsAtom, [c1, c2]);

    store.set(setStreamingContentAction, { conversationId: 'conv-1', content: 'conv1 tokens' });

    // conv-1 has streaming content
    expect(store.get(streamingContentFamily('conv-1'))).toBe('conv1 tokens');
    // conv-2 buffer is untouched
    expect(store.get(streamingContentFamily('conv-2'))).toBe('');

    // Clearing conv-1 does not affect conv-2
    store.set(clearStreamingContentAction, 'conv-1');
    expect(store.get(streamingContentFamily('conv-1'))).toBe('');
    expect(store.get(streamingContentFamily('conv-2'))).toBe('');
  });

  it('streamingActiveIdsAtom tracks which conversations are streaming', () => {
    store.set(setStreamingContentAction, { conversationId: 'conv-a', content: 'a' });
    store.set(setStreamingContentAction, { conversationId: 'conv-b', content: 'b' });

    expect(store.get(streamingActiveIdsAtom).has('conv-a')).toBe(true);
    expect(store.get(streamingActiveIdsAtom).has('conv-b')).toBe(true);

    store.set(clearStreamingContentAction, 'conv-a');
    expect(store.get(streamingActiveIdsAtom).has('conv-a')).toBe(false);
    expect(store.get(streamingActiveIdsAtom).has('conv-b')).toBe(true);
  });

  it('clearAllStreamingAction sweeps all active buffers', () => {
    store.set(setStreamingContentAction, { conversationId: 'c1', content: 'tok1' });
    store.set(setStreamingContentAction, { conversationId: 'c2', content: 'tok2' });
    store.set(setStreamingContentAction, { conversationId: 'c3', content: 'tok3' });

    store.set(clearAllStreamingAction);

    expect(store.get(streamingContentFamily('c1'))).toBe('');
    expect(store.get(streamingContentFamily('c2'))).toBe('');
    expect(store.get(streamingContentFamily('c3'))).toBe('');
    expect(store.get(streamingActiveIdsAtom).size).toBe(0);
  });

  it('clearAllStreamingAction is a no-op when no streams are active', () => {
    // Should not throw or modify anything
    expect(() => store.set(clearAllStreamingAction)).not.toThrow();
    expect(store.get(streamingActiveIdsAtom).size).toBe(0);
  });

  it('latest streaming content replaces previous (full-text cursor semantics)', () => {
    store.set(setStreamingContentAction, { conversationId: 'conv-1', content: 'Hello' });
    store.set(setStreamingContentAction, { conversationId: 'conv-1', content: 'Hello, world' });
    store.set(setStreamingContentAction, { conversationId: 'conv-1', content: 'Hello, world!' });

    // Latest value wins — server sends full accumulated text
    expect(store.get(streamingContentFamily('conv-1'))).toBe('Hello, world!');
  });
});

describe('agent chat inflight isolation', () => {
  let store: ReturnType<typeof createStore>;

  beforeEach(() => {
    store = createStore();
  });

  it('inflight text does not appear in agentChatMessagesAtom', () => {
    const inflight: AgentInflight = {
      session_id: 'sess-abc',
      text: 'partial agent response...',
      tool_calls: [],
      started_at: Date.now(),
    };
    store.set(agentInflightAtom, inflight);
    store.set(agentChatMessagesAtom, []);

    // Inflight text is in agentInflightAtom
    expect(store.get(agentInflightAtom)?.text).toBe('partial agent response...');

    // Committed messages are empty — partial text not written there
    expect(store.get(agentChatMessagesAtom)).toHaveLength(0);
  });

  it('WS disconnect clears agentInflight without touching committed messages', () => {
    const committedMsg = {
      id: 'msg-1',
      role: 'assistant' as const,
      content: 'previous completed response',
      timestamp: Date.now() - 10000,
      session_id: 'sess-abc',
    };
    store.set(agentChatMessagesAtom, [committedMsg]);

    const inflight: AgentInflight = {
      session_id: 'sess-abc',
      text: 'mid-turn partial...',
      tool_calls: [],
      started_at: Date.now(),
    };
    store.set(agentInflightAtom, inflight);

    // Simulate ws.onclose: clear inflight, do not modify committed messages
    store.set(agentInflightAtom, null);

    // Inflight is gone
    expect(store.get(agentInflightAtom)).toBeNull();

    // Committed messages unchanged
    const msgs = store.get(agentChatMessagesAtom);
    expect(msgs).toHaveLength(1);
    expect(msgs[0].content).toBe('previous completed response');
    expect(msgs[0].content).not.toContain('partial');
  });

  it('turn_complete moves inflight to committed messages, clears buffer', () => {
    const inflight: AgentInflight = {
      session_id: 'sess-abc',
      text: 'completed response text',
      tool_calls: [],
      started_at: Date.now(),
    };
    store.set(agentInflightAtom, inflight);

    // Simulate turn_complete handler
    const finalMsg = {
      id: crypto.randomUUID(),
      role: 'assistant' as const,
      content: inflight.text,
      timestamp: Date.now(),
      session_id: inflight.session_id,
    };
    store.set(agentChatMessagesAtom, (prev) => [...prev, finalMsg]);
    store.set(agentInflightAtom, null);

    // Inflight is cleared
    expect(store.get(agentInflightAtom)).toBeNull();

    // Content promoted to committed store
    const msgs = store.get(agentChatMessagesAtom);
    expect(msgs).toHaveLength(1);
    expect(msgs[0].content).toBe('completed response text');
  });
});
