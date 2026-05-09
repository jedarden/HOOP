# Phase 5 Implementation Status: The human-interface agent (v0.5)

**Status:** Substantially Complete (2026-05-09)

## Overview

Phase 5 implements the human-interface agent — a persistent, cross-project conversation partner that reviews artifacts, answers questions, and drafts Stitches. The agent becomes the operator's primary interface to HOOP.

## Completed Components

### 1. Agent Session Management (hoop-ttb.6.2) ✅
**File:** `hoop-daemon/src/agent_session.rs`

- Persistent agent session that survives `systemctl restart hoop`
- Session persistence to `fleet.db`
- Attach-on-restart capability
- Adapter switch support (archives old session, starts fresh)

### 2. Agent Adapter Abstraction (hoop-ttb.6.1) ✅
**File:** `hoop-daemon/src/agent_adapter.rs`

- LLM-agnostic adapter interface
- Claude Code adapter (default)
- Anthropic API adapter
- ZAI+GLM adapter
- Pluggable architecture for future adapters

### 3. MCP Server (hoop-ttb.6.3) ✅
**Files:** `hoop-mcp/src/*.rs`

- Unix socket-based MCP server
- Read APIs: projects, beads, Stitches, files, conversations
- One write tool: `create_stitch` (routes through draft queue)
- Forbidden worker actions enforced at compile-time

### 4. Agent Context — Lazy Fetch (hoop-ttb.6.4) ✅
**File:** `hoop-daemon/src/agent_context.rs`

- Thin index by default
- Details fetched via tool calls on demand
- System prompt budget enforcement (4KB default)
- Reflection Ledger injection

### 5. Agent Chat Pane (hoop-ttb.6.5) ✅
**File:** `hoop-ui/web/src/AgentChatPane.tsx`

- Operator ↔ agent chat UI
- Multimodal input (text + file attachments)
- Real-time streaming
- Cross-project by design
- Tool call visualization

### 6. Draft Queue — Preview Flow (hoop-ttb.6.6) ✅
**File:** `hoop-daemon/src/api_draft_queue.rs`

- Agent-drafted Stitches route through preview queue
- No direct submits (read-first default)
- Operator approval required before bead creation
- Dedup detection with false-positive reporting

### 7. Fleet Notification Channel (hoop-ttb.6.7) ✅
**File:** `hoop-daemon/src/fleet_notifications.rs`

- StitchBeadsClosed notifications
- ConvoyComplete notifications
- CapacityAlert notifications (5-hour threshold)
- BeadCreatedByHoop notifications
- In-memory ring buffer (20 notifications)
- Agent decides whether to escalate to operator

### 8. Morning Brief (hoop-ttb.6.9) ✅
**Files:** `hoop-daemon/src/morning_brief.rs`, `hoop-daemon/src/api_morning_brief.rs`

- Autonomous daily briefing
- What closed, what failed, what's stuck, what's anomalous
- Pre-drafted Stitches (always unsubmitted)
- **One headline** — single priority for today
- Configurable schedule (default 07:00)

### 9. Cross-Project Stitch Propagation (hoop-ttb.6.10) ✅
**File:** `hoop-daemon/src/cross_project_propagation.rs`

- Sibling project detection
- Lexical + file-path similarity scoring
- Surfaced as: "you just closed X in project A. The same pattern exists in B, C..."
- Always preview; operator accepts per-project or all-at-once

### 10. Reflection Ledger Schema (hoop-ttb.6.11) ✅
**File:** `hoop-daemon/src/fleet.rs` (reflection_ledger table)

- `reflection_ledger` table in fleet.db
- Scope model: global / project / pattern
- Status: proposed / approved / rejected / archived
- Source stitch tracking
- Application count tracking

### 11. Reflection Detector (hoop-ttb.6.12) ✅
**File:** `hoop-daemon/src/reflection_detector.rs`

- Scans closed operator Stitches for repeated patterns
- Detects: corrections, preferences, negatives, approvals
- Configurable scan window (default 30 days)
- Minimum occurrences threshold (default 3)
- Proposes entries to Reflection Ledger

### 12. Reflection Ledger API (hoop-ttb.6.13) ✅
**File:** `hoop-daemon/src/api_reflection_ledger.rs`

- GET /api/reflections/proposals — list pending proposals
- POST /api/reflections/{id}/approve — approve a proposal
- POST /api/reflections/{id}/reject — reject a proposal
- GET /api/reflections — list approved reflections

### 13. Reflection Ledger Injection (hoop-ttb.6.14) ✅
**File:** `hoop-daemon/src/agent_context.rs`

- Approved rules injected into every new agent session
- Part of lazy-context index
- Per-session audit (which rules applied to which turn)

### 14. Agent Audit Trail (hoop-ttb.6.15) ✅
**Files:** `hoop-daemon/src/agent_session.rs`, `hoop-daemon/tests/agent_turn_audit_trail.rs`

- Actor format: `hoop:agent:<session>`
- Turn ID tracking
- Audit row includes: session_id, adapter, model, turn_id
- Stitch reconstructable back to origin turn

### 15. Forbidden Agent Tools (hoop-ttb.6.8) ✅
**Files:** `hoop-mcp/tests/compile_fail_*.rs`

- No `launch_fleet`, `stop_fleet`, `release_claim`, `boost_priority`, `close_stitch`, `close_bead`
- Compile-time enforcement via Rust type system
- UI tests verify forbidden actions are blocked

## Tests Implemented

### Integration Tests
- `agent_turn_audit_trail.rs` — Agent audit trail verification
- `create_stitch_no_auto_submit.rs` — Draft queue invariant
- `fleet_notifications_integration.rs` — Notification ring
- `draft_queue_invariants.rs` — Draft queue property tests

### Compile-Fail Tests (MCP Server)
- `invoke_br_depend_forbidden.rs`
- `invoke_br_release_forbidden.rs`
- `invoke_br_claim_forbidden.rs`
- `invoke_br_update_forbidden.rs`
- `invoke_br_close_raw_forbidden.rs`
- `invoke_br_write_forbidden_under_create_only.rs`

## Remaining Work

### hoop-ttb.6.2.2 — Adapter Failover Tests
- hoop-ttb.6.2.2.1: Implement Anthropic 5xx mock
- hoop-ttb.6.2.2.2: Test daemon survives 5xx without crash
- hoop-ttb.6.2.2.3: Test config.yml adapter=zai hot-reload
- hoop-ttb.6.2.2.4: Assert old session archived as closed Stitch
- hoop-ttb.6.2.2.5: Assert Reflection Ledger continuity

### hoop-ttb.6.3.1 — MCP Socket Protocol Test
- Daemon ↔ hoop-mcp socket protocol contract test
- Schema-fixture round-trip verification

### hoop-ttb.6.13.1 — Reflection Proposal Audit
- Audit rows on reflection proposal approve / edit / reject

### hoop-ttb.6.1.1 — Per-Adapter Reasoning Effort
- Enum validator at WS boundary

## Phase 5 Success Criteria Status

| Criterion | Status | Notes |
|-----------|--------|-------|
| Agent session survives systemctl restart | ✅ | Session persistence to fleet.db |
| Cross-project summary in Stitch language | ✅ | Lazy context + project summaries |
| Agent reviews artifacts and drafts Stitches | ✅ | Full MCP tool belt |
| Agent never performs worker actions | ✅ | Compile-time enforcement |
| Morning Brief produces useful daily summary | ✅ | Configured by default |
| Cross-Project Propagation catches siblings | ✅ | Similarity-based detection |
| Agent audit trail reconstructable | ✅ | Turn ID tracking |

## Architecture Notes

- **Lazy Context:** Agent receives thin index; fetches details via tools
- **Read-First Default:** All drafts route through preview queue
- **LLM-Agnostic:** Adapter abstraction supports Claude / Anthropic / ZAI / future
- **Agent-Off Switch:** HOOP fully functional without agent
- **Audit Trail:** Every agent-drafted Stitch carries `actor: hoop:agent:<session>`

## References

- Plan §6 Phase 5
- Plan §1.6 (Stitches and Patterns)
- Plan §4.7 (Reflection Ledger schema)
- AGENTS.md (LLM-facing guide)
