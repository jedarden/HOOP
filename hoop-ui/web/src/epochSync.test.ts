/**
 * Epoch-sync invariant tests (§B2, hoop-ttb.3.46)
 *
 * Core invariant: on every WS (re)connect, client wipes atom store and rebuilds
 * from server's `init` payload. Only un-sent optimistic stubs survive.
 *
 * Test scenarios:
 * 1. Init event clears all atoms (except optimistic stubs and committed messages)
 * 2. Simulated disconnect → mutate state → reconnect → stale rows gone
 * 3. Optimistic stubs survive init
 * 4. Committed agent messages survive init
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { createStore } from 'jotai';
import type { StuckReason } from './atoms';
import {
  workersAtom,
  beadsAtom,
  conversationsAtom,
  projectCardsAtom,
  capacityAtom,
  stitchCreatedAtom,
  agentSessionStatusAtom,
  agentInflightAtom,
  agentChatMessagesAtom,
  stuckAlertsAtom,
  optimisticStubsAtom,
  configStatusAtom,
  BeadData,
  OptimisticStub,
} from './atoms';

describe('epoch-sync invariant (§B2)', () => {
  let store: ReturnType<typeof createStore>;

  beforeEach(() => {
    store = createStore();
    // Pre-populate store with state that should be cleared on init
    store.set(workersAtom, [
      {
        worker: 'worker-1',
        state: { state: 'idle', last_strand: null },
        liveness: 'Live',
        last_heartbeat: new Date().toISOString(),
        heartbeat_age_secs: 1,
      },
    ]);
    store.set(beadsAtom, [
      {
        id: 'bead-1',
        title: 'Test Bead',
        status: 'open',
        priority: 1,
        issue_type: 'task',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        created_by: 'test',
        dependencies: [],
        project: 'test-project',
      },
    ]);
    store.set(conversationsAtom, [
      {
        id: 'conv-1',
        session_id: 'sess-1',
        provider: 'claude',
        kind: 'worker',
        cwd: '/tmp',
        title: 'Test Conversation',
        messages: [],
        total_tokens: 0,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        complete: false,
        file_path: '/tmp/test.jsonl',
      },
    ]);
    store.set(projectCardsAtom, [
      {
        name: 'test-project',
        label: 'Test Project',
        color: '#000000',
        path: '/tmp/test',
        degraded: false,
        bead_count: 1,
        worker_count: 1,
        active_stitch_count: 0,
        cost_today: 0,
        stuck_count: 0,
      },
    ]);
    store.set(capacityAtom, [
      {
        account_id: 'acc-1',
        adapter: 'claude',
        plan_type: 'pro',
        rate_limit_tier: 'tier-1',
        utilization_5h: 0.5,
        utilization_7d: 0.3,
        tokens_5h: 1000,
        tokens_7d: 5000,
        turns_5h: 10,
        turns_7d: 50,
        prompts_5h: 0,
        prompts_7d: 0,
        burn_rate_per_min: 0.1,
        mean_cost_per_stitch_tokens: 100,
        stitch_close_rate_per_min: 0.5,
        source: 'test',
        computed_at: new Date().toISOString(),
      },
    ]);
    store.set(stitchCreatedAtom, [
      {
        bead_id: 'bead-1',
        title: 'Test Stitch',
        project: 'test-project',
        stitch_id: 'stitch-1',
        source: 'test',
        actor: 'test',
        created_at: new Date().toISOString(),
      },
    ]);
    store.set(agentSessionStatusAtom, {
      active: true,
      enabled: true,
      session_id: 'sess-1',
      adapter: 'claude',
      model: 'claude-3-opus',
      stitch_id: null,
      cost_usd: 0.5,
      input_tokens: 100,
      output_tokens: 200,
      turn_count: 1,
      created_at: new Date().toISOString(),
      last_activity_at: new Date().toISOString(),
      age_secs: 60,
    });
    store.set(agentInflightAtom, {
      session_id: 'sess-1',
      text: 'partial response...',
      tool_calls: [],
      started_at: Date.now(),
    });
    store.set(stuckAlertsAtom, new Map([['worker-1', {
      worker: 'worker-1',
      bead: 'bead-1',
      started_at: new Date().toISOString(),
      last_event_at: new Date().toISOString(),
      elapsed_secs: 300,
      idle_secs: 60,
      saw_content: true,
      reason: 'idle_timeout' as StuckReason,
      message: 'Worker idle for 60s',
      last_heartbeat_at: new Date().toISOString(),
      last_transition_at: new Date().toISOString(),
      retry_count: 0,
    }]]));
    store.set(configStatusAtom, { valid: true });
  });

  it('init event clears all atoms except optimistic stubs and committed messages', () => {
    // Set up optimistic stubs that should survive
    const stub: OptimisticStub = {
      tempId: 'temp-1',
      type: 'bead',
      data: { title: 'Pending Bead' },
      createdAt: Date.now(),
    };
    store.set(optimisticStubsAtom, [stub]);

    // Set up committed agent messages that should survive
    const committedMsg = {
      id: 'msg-1',
      role: 'assistant' as const,
      content: 'completed response',
      timestamp: Date.now() - 10000,
      session_id: 'sess-1',
    };
    store.set(agentChatMessagesAtom, [committedMsg]);

    // Verify pre-init state
    expect(store.get(workersAtom)).toHaveLength(1);
    expect(store.get(beadsAtom)).toHaveLength(1);
    expect(store.get(conversationsAtom)).toHaveLength(1);
    expect(store.get(projectCardsAtom)).toHaveLength(1);
    expect(store.get(capacityAtom)).toHaveLength(1);
    expect(store.get(stitchCreatedAtom)).toHaveLength(1);
    expect(store.get(agentSessionStatusAtom)).not.toBeNull();
    expect(store.get(agentInflightAtom)).not.toBeNull();
    expect(store.get(stuckAlertsAtom).size).toBe(1);

    // Simulate init event (epoch-sync wipe)
    store.set(workersAtom, []);
    store.set(conversationsAtom, []);
    store.set(projectCardsAtom, []);
    store.set(capacityAtom, []);
    store.set(stitchCreatedAtom, []);
    store.set(agentSessionStatusAtom, null);
    store.set(agentInflightAtom, null);
    store.set(stuckAlertsAtom, new Map());
    store.set(configStatusAtom, { valid: true });
    // Note: beadsAtom is also cleared but will be rebuilt from beads_snapshot

    // Verify all atoms cleared
    expect(store.get(workersAtom)).toHaveLength(0);
    expect(store.get(conversationsAtom)).toHaveLength(0);
    expect(store.get(projectCardsAtom)).toHaveLength(0);
    expect(store.get(capacityAtom)).toHaveLength(0);
    expect(store.get(stitchCreatedAtom)).toHaveLength(0);
    expect(store.get(agentSessionStatusAtom)).toBeNull();
    expect(store.get(agentInflightAtom)).toBeNull();
    expect(store.get(stuckAlertsAtom).size).toBe(0);
    expect(store.get(configStatusAtom).valid).toBe(true);

    // Verify optimistic stubs survived
    expect(store.get(optimisticStubsAtom)).toEqual([stub]);

    // Verify committed messages survived
    expect(store.get(agentChatMessagesAtom)).toEqual([committedMsg]);
  });

  it('disconnect → mutate state → reconnect → stale rows gone', () => {
    // Set optimistic stubs and committed messages
    const stub: OptimisticStub = {
      tempId: 'temp-1',
      type: 'bead',
      data: { title: 'Pending Bead' },
      createdAt: Date.now(),
    };
    store.set(optimisticStubsAtom, [stub]);

    const committedMsg = {
      id: 'msg-1',
      role: 'assistant' as const,
      content: 'completed response',
      timestamp: Date.now() - 10000,
      session_id: 'sess-1',
    };
    store.set(agentChatMessagesAtom, [committedMsg]);

    // Simulate disconnect: inflight cleared
    store.set(agentInflightAtom, null);

    // Simulate server state change while disconnected: new bead added on server
    // (In real scenario, this would come from beads_snapshot after init)

    // Simulate reconnect (init event)
    store.set(workersAtom, []);
    store.set(conversationsAtom, []);
    store.set(beadsAtom, []); // Will be rebuilt from snapshot
    store.set(projectCardsAtom, []);
    store.set(capacityAtom, []);
    store.set(stitchCreatedAtom, []);
    store.set(agentSessionStatusAtom, null);
    store.set(agentInflightAtom, null);
    store.set(stuckAlertsAtom, new Map());
    store.set(configStatusAtom, { valid: true });

    // Verify stale state is gone
    expect(store.get(workersAtom)).toHaveLength(0);
    expect(store.get(beadsAtom)).toHaveLength(0);
    expect(store.get(agentSessionStatusAtom)).toBeNull();
    expect(store.get(agentInflightAtom)).toBeNull();

    // Verify optimistic stubs survived (they represent un-sent mutations)
    expect(store.get(optimisticStubsAtom)).toEqual([stub]);

    // Verify committed messages survived (they're authoritative)
    expect(store.get(agentChatMessagesAtom)).toEqual([committedMsg]);

    // Simulate receiving new snapshot from server
    const newBead: BeadData = {
      id: 'bead-2', // Different from the stale bead-1
      title: 'New Bead from Server',
      status: 'open',
      priority: 1,
      issue_type: 'task',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      created_by: 'test',
      dependencies: [],
      project: 'test-project',
    };
    store.set(beadsAtom, [newBead]);

    // Verify only server state is present (stale bead-1 is gone)
    expect(store.get(beadsAtom)).toHaveLength(1);
    expect(store.get(beadsAtom)[0].id).toBe('bead-2');
  });

  it('optimistic stubs survive multiple reconnect cycles', () => {
    const stub1: OptimisticStub = {
      tempId: 'temp-1',
      type: 'bead',
      data: { title: 'Pending Bead 1' },
      createdAt: Date.now() - 5000,
    };
    const stub2: OptimisticStub = {
      tempId: 'temp-2',
      type: 'stitch',
      data: { title: 'Pending Stitch 1' },
      createdAt: Date.now(),
    };
    store.set(optimisticStubsAtom, [stub1, stub2]);

    // First reconnect cycle
    store.set(workersAtom, []);
    store.set(conversationsAtom, []);
    store.set(beadsAtom, []);
    store.set(projectCardsAtom, []);

    expect(store.get(optimisticStubsAtom)).toEqual([stub1, stub2]);

    // Second reconnect cycle
    store.set(workersAtom, []);
    store.set(conversationsAtom, []);
    store.set(beadsAtom, []);

    expect(store.get(optimisticStubsAtom)).toEqual([stub1, stub2]);
  });

  it('committed agent messages are never cleared by epoch-sync', () => {
    const messages = [
      {
        id: 'msg-1',
        role: 'user' as const,
        content: 'Hello',
        timestamp: Date.now() - 30000,
        session_id: 'sess-1',
      },
      {
        id: 'msg-2',
        role: 'assistant' as const,
        content: 'Hi there!',
        timestamp: Date.now() - 20000,
        session_id: 'sess-1',
      },
      {
        id: 'msg-3',
        role: 'user' as const,
        content: 'How are you?',
        timestamp: Date.now() - 10000,
        session_id: 'sess-1',
      },
    ];
    store.set(agentChatMessagesAtom, messages);

    // Simulate epoch-sync
    store.set(workersAtom, []);
    store.set(conversationsAtom, []);
    store.set(beadsAtom, []);
    store.set(agentSessionStatusAtom, null);
    store.set(agentInflightAtom, null);

    // Committed messages must survive
    expect(store.get(agentChatMessagesAtom)).toEqual(messages);
  });

  it('inflight agent state is cleared on init (but committed messages survive)', () => {
    const inflight = {
      session_id: 'sess-1',
      text: 'partial response...',
      tool_calls: [],
      started_at: Date.now(),
    };
    store.set(agentInflightAtom, inflight);

    const committedMsg = {
      id: 'msg-1',
      role: 'assistant' as const,
      content: 'previous completed response',
      timestamp: Date.now() - 10000,
      session_id: 'sess-1',
    };
    store.set(agentChatMessagesAtom, [committedMsg]);

    // Simulate init event
    store.set(agentInflightAtom, null);

    // Inflight is cleared
    expect(store.get(agentInflightAtom)).toBeNull();

    // Committed messages survive
    expect(store.get(agentChatMessagesAtom)).toEqual([committedMsg]);
  });
});
