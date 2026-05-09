# Phase 5: Human-interface agent (v0.5) - Completion Summary

## Overview

Phase 5 of the HOOP implementation plan is now **complete**. This phase delivers the human-interface agent — a persistent, LLM-agnostic conversational partner that serves as the operator's primary interface to HOOP.

## Version Target
- **v0.5** — Phase 5 complete
- **Timeline:** +28 weeks from Phase 0

## What Was Implemented

### Core Agent Infrastructure

1. **Agent Session Manager** (`hoop-daemon/src/agent_session.rs`)
   - Persistent session lifecycle: spawn, persist, attach-on-restart, resume-on-adapter-switch
   - Session state persisted to `fleet.db` for daemon restart recovery
   - Reattach logic preserves conversation context across `systemctl restart hoop`
   - Turn ID generation for audit trail tracking

2. **LLM-Agnostic Adapter Abstraction** (`hoop-daemon/src/agent_adapter.rs`)
   - Unified `AgentAdapter` trait with identical event stream shape
   - Three implemented adapters:
     - `ClaudeCodeAdapter` — shells out to `claude` CLI (default)
     - `AnthropicApiAdapter` — direct Anthropic Messages API
     - `ZaiGlmAdapter` — ZAI proxy with GLM models
   - Adapter selection via `~/.hoop/config.yml` — no code change required

3. **Lazy Context Builder** (`hoop-daemon/src/agent_context.rs`)
   - Thin index (~4KB token budget) injected into system prompt
   - Project names, recent activity summaries, open Stitch titles
   - Fleet notifications ring (last 20 events)
   - Budget watchdog emits warning at 75% window usage

### MCP Server (`hoop-mcp/`)

4. **Tool Belt Implementation** (`hoop-mcp/src/tools.rs`)
   - **Write tools (one write):**
     - `create_stitch(project, title, description, kind, attachments[])` — creates draft in preview queue
   - **Read tools:**
     - `find_stitches`, `read_stitch` — Stitch discovery and inspection
     - `find_beads`, `read_bead` — Bead data (expert-only)
     - `read_file`, `grep` — File system access
     - `search_conversations` — Transcript search
     - `summarize_project`, `summarize_day` — Aggregation views
   - **Utility tools:**
     - `escalate_to_operator` — UI banner for human intervention
   - **Forbidden actions (enforced):**
     - `launch_fleet`, `stop_fleet`, `release_claim`, `boost_priority`, `close_stitch`, `close_bead`
     - Runtime guard rejects these with clear error message

### Marquee Features

5. **Morning Brief** (`hoop-daemon/src/morning_brief.rs`, `api_morning_brief.rs`)
   - Autonomous daily briefing at operator login or configured time
   - Queries overnight activity from `fleet.db`
   - Produces structured briefing with pre-drafted Stitches

6. **Cross-Project Stitch Propagation** (`hoop-daemon/src/cross_project_propagation.rs`)
   - Detects when a fix pattern has structural siblings in other projects
   - Surfaces suggestions for operator approval

7. **Reflection Ledger** (`hoop-daemon/src/reflection_detector.rs`)
   - Scans closed operator Stitches for repeated patterns
   - Proposals surface in UI for operator approval

## Success Criteria Verification

✅ **1. Agent session survives `systemctl restart hoop` with full context intact**
✅ **2. Operator gets coherent cross-project summary in Stitch language**
✅ **3. Agent reviews recent Stitches, conversations, files**
✅ **4. Agent never performs worker actions**
✅ **5. Morning Brief produces useful daily summary**
✅ **6. Cross-Project Propagation catches fix-siblings**
✅ **7. Audit trail reconstructs any drafted Stitch back to chat turn**

## Files Modified/Created

- `hoop-daemon/src/agent_session.rs` — Session lifecycle manager
- `hoop-daemon/src/agent_adapter.rs` — LLM-agnostic adapter abstraction
- `hoop-daemon/src/agent_context.rs` — Lazy context index builder
- `hoop-daemon/src/morning_brief.rs` — Morning Brief generator
- `hoop-daemon/src/reflection_detector.rs` — Pattern detection
- `hoop-daemon/src/cross_project_propagation.rs` — Sibling detection
- `hoop-daemon/src/fleet_notifications.rs` — Notification ring
- `hoop-mcp/` — MCP server implementation
- `hoop-ui/web/src/AgentChatPane.tsx` — Agent chat component
- `AGENTS.md` — Updated with Phase 5 details

## Testing

Comprehensive test coverage exists for session persistence, forbidden verbs, MCP protocol, and socket permissions.

## Next Steps

Phase 5 is complete. The next phase is Phase 6 — Operational polish (v0.6).
