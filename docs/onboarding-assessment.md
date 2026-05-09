# HOOP Onboarding & Documentation Assessment

**Bead:** hoop-ttb.9
**Plan Reference:** §12 Onboarding & documentation
**Date:** 2026-05-09
**Status:** ✅ COMPLETE

## Overview

This document assesses the implementation of onboarding and documentation deliverables as specified in §12 of the canonical plan. Onboarding is a cross-cutting concern that delivers alongside each new surface, ensuring operators can progressively discover HOOP's capabilities without being overwhelmed on day one.

## Three Onboarding Surfaces (§12)

### 1. `hoop init` — Interactive CLI Wizard ✅

**Location:** `hoop-cli/src/init.rs`

**Implemented Stages:**
- ✅ Dependency check via `hoop audit` (br version, tmux, git, Tailscale, disk space, port availability)
- ✅ First project registration with scan preview
- ✅ Agent adapter setup (optional - Claude Code, Anthropic API, or ZAI)
- ✅ systemd user service install (optional)
- ✅ Health check + URL print

**Key Features:**
- Re-runnable and idempotent — each step can be skipped if already configured
- Target: under 5 minutes if tools are already installed
- Prints exact fix commands for any failed dependency checks

### 2. In-UI First-Run Experience ✅

**Location:** `hoop-ui/web/src/components/WelcomeTour.tsx`

**Implemented Elements:**
- ✅ Welcome overlay with concept explanations (Stitches, Patterns, Agent)
- ✅ Guided tour with soft highlights on key UI elements
- ✅ Three starter prompts:
  1. "Enable Tour Project" — spin up demo workspace with example Stitches
  2. "Dictate a first note" — voice note with transcription
  3. "Register another project" — add a new project to HOOP
  4. "Ask the agent something" — start a conversation with the AI
- ✅ Dismissable; re-openable from settings
- ✅ Keyboard shortcut (Cmd/Ctrl+K) emphasized

**Tour Steps:**
1. Welcome to HOOP (concept intro)
2. Your Projects (project cards overview)
3. Quick Actions (search palette)
4. Get Started (starter prompts)

### 3. Progressive Capability Introduction ✅

**Location:** `hoop-ui/web/src/components/OnboardingPromptBanner.tsx`, `hoop-daemon/src/api_onboarding.rs`

**Implemented Prompts:**
- ✅ "What's new" card on version upgrade
- ✅ Reflection Ledger empty after 30 days → propose rules
- ✅ 10+ Stitches share a theme → suggest creating a Pattern
- ✅ Agent never used → inline prompt on chat pane
- ✅ Mic never used → prompt near hotkey icon

**Backend API:**
- `GET /api/onboarding/prompts` — list eligible prompts
- `POST /api/onboarding/dismiss` — dismiss a prompt
- `POST /api/onboarding/enable` — globally enable/disable prompts
- `POST /api/onboarding/record-usage` — record feature usage
- `POST /api/onboarding/ack-version` — acknowledge current version

## Specific Onboarding Aids (§12)

### Explain-This Hover ✅

**Location:** `hoop-ui/web/src/components/ExplainThis.tsx`

**Features:**
- Central `UI_GLOSSARY` with one-sentence explanations for every non-obvious UI element
- Categories: navigation, stitches, agent, dictation, files, learning, settings, fleet, cost
- Tooltip or inline mode
- Keyboard-accessible
- ARIA-compliant

**Coverage:**
- Project Switcher, Stitch List, Pattern List, Search Palette
- Create Stitch, Filter Stitches, Stitch Draft, Link Bead
- Agent Chat, Morning Brief, Capacity Widget
- Dictation Widget, Dictation Hotkey
- File Browser, File Search
- Reflection Ledger, Reflection Proposal
- Settings Menu, Theme Toggle
- Fleet Status, Worker Timeline
- Cost Today, Cost Anomaly

### Dry-Run Mode for First Stitch Drafts ✅

**Location:** `hoop-ui/web/src/StitchDraftForm.tsx`, `hoop-daemon/src/api_draft_queue.rs`

**Features:**
- Preview panel with rendered markdown + computed dependency graph
- Cost estimate from historical similar Stitches
- Duration p50/p90 estimates
- Risk pattern matching
- File conflict detection
- Similar Stitches reference list

### Sample Stitches Tour Project ✅

**Location:** `hoop-daemon/src/api_tour_project.rs`

**Features:**
- One-click spin-up of demo workspace (`~/.hoop/tour/`)
- Four example Stitches:
  1. Voice Note Demo (dictated)
  2. Agent Chat Demo (operator)
  3. Linked Beads Demo (ad-hoc)
  4. Cost Anomaly Demo (worker)
- Removable in one click
- Appears as purple project card "HOOP Tour"

**API Endpoints:**
- `POST /api/tour/enable` — create tour project
- `DELETE /api/tour/disable` — remove tour project
- `GET /api/tour/status` — check tour status

### Agent Pre-Priming ✅

**Location:** `hoop-daemon/src/agent_context.rs`

**Features:**
- Agent's first message references operator's actual data
- Lazy context includes: project names, recent activity summaries, open Stitch titles, alerts
- Full details fetched via MCP tools on demand

## Repository Documentation (§12)

### ✅ README.md at repo root
**Location:** `/home/coding/HOOP/README.md`

**Contents:**
- Combined install + run + concepts + quickstart
- Under-30-min path for a human visitor
- Prerequisites (br, tmux, git, CLI adapters, NEEDLE, API key)
- Quick install and install-from-source instructions
- Requirements table with verified versions
- Documentation map with links to AGENTS.md, CHANGELOG.md, operations.md, troubleshooting.md, plan.md

### ✅ AGENTS.md at repo root
**Location:** `/home/coding/HOOP/AGENTS.md`

**Contents:**
- LLM-facing guide summarizing scope, non-goals, terminology, conventions
- Points at the plan as authoritative
- Keeps LLMs from re-introducing removed vocabulary (Mayor, polecat, Gas Town)
- Prevents proposing disallowed features (worker steering, capacity enforcement)

### ✅ docs/plan/plan.md
**Location:** `/home/coding/HOOP/docs/plan/plan.md`

**Contents:**
- Canonical, detailed implementation plan
- 13 sections covering vision, principles, architecture, data flows, phased roadmap
- What the README links to for depth

### ✅ docs/concepts/ — one-page-per-concept docs
**Location:** `/home/coding/HOOP/docs/concepts/`

**Concepts Documented:**
- ✅ beads.md — NEEDLE's internal execution unit
- ✅ stitches.md — HOOP's user-facing unit (conversations and work items)
- ✅ patterns.md — Optional, operator-curated grouping of Stitches toward a goal
- ✅ projects-workspaces.md — Project = logical unit; Workspace = single repo
- ✅ human-interface-agent.md — Persistent LLM session as operator's primary interface
- ✅ reflection-ledger.md — Learned-rules store from repeated patterns
- ✅ privacy.md — Privacy model and data handling

### ✅ docs/operations.md
**Location:** `/home/coding/HOOP/docs/operations.md`

**Contents:**
- systemd user service management
- Log viewing and rotation
- Upgrade procedures
- Schema migrations
- Tailscale routing
- Backup and disaster recovery

### ✅ docs/troubleshooting.md
**Location:** `/home/coding/HOOP/docs/troubleshooting.md`

**Contents:**
- Common failures mapped to `hoop audit` output
- Recovery steps for each failure
- Audit severity levels (critical, warning, info)

## Phase-by-Phase Onboarding Deliverables

| Phase | Deliverable | Status | Evidence |
|-------|-------------|--------|----------|
| 1 | `hoop audit` + `hoop init` wizard (minimum viable) | ✅ | `hoop-cli/src/init.rs`, `hoop-daemon/src/audit.rs` |
| 1 | README at repo root with install + quickstart flow | ✅ | `/home/coding/HOOP/README.md` |
| 2 | UI first-run tour | ✅ | `hoop-ui/web/src/components/WelcomeTour.tsx` |
| 2 | Project-scan guidance | ✅ | `init.rs` stage 2 offers `scan ~/` with preview |
| 2 | Capacity-widget explanations | ✅ | `ExplainThis.tsx` includes "capacity" entry |
| 3 | File browser quick-start tooltip | ✅ | `ExplainThis.tsx` includes "file-browser" entry |
| 3 | First-dictation prompt near mic hotkey | ✅ | `OnboardingPromptBanner.tsx` MicIntro prompt |
| 4 | Stitch-draft form with inline field hints | ✅ | `StitchDraftForm.tsx` with ExplainThis wrappers |
| 4 | Sample templates library | ✅ | `hoop-daemon/src/template_library.rs` |
| 4 | Dry-run preview | ✅ | `api_preview.rs` "What Will This Take?" endpoint |
| 5 | Agent setup wizard | ✅ | `init.rs` stage 3 (optional agent adapter setup) |
| 5 | Morning Brief self-introduction on first run | ✅ | `morning_brief.rs` includes introductory message |
| 5 | Reflection Ledger first-proposal tutorial | ✅ | `api_reflection_ledger.rs` with proposal UI |

## Onboarding Principles (§12)

### ✅ Progressive, never front-loaded
- No concept dump on day one
- Each feature introduced when relevant
- Starter prompts for immediate action

### ✅ Viewable opt-out, invisible opt-in
- All onboarding aids are dismissable
- None gate functionality
- "Don't bug me" global setting in `api_onboarding.rs`

### ✅ Operator-specific, not generic
- Welcome messages reference operator's actual data
- Agent's first message uses actual project names and counts
- Personalized from second zero

### ✅ Re-playable
- Every tour, tutorial, and introduction can be re-opened from settings
- `WelcomeTourTrigger` component allows restart
- Prompts respect dismissal but can be reset

### ✅ LLM-first documentation path
- `AGENTS.md` provides LLM-facing guide
- Repo immediately useful to contributor LLM starting fresh
- Terminology enforced to prevent re-introduction of removed concepts

## Closing Criteria

All closing criteria from §12 are met:

✅ All phases have their onboarding deliverables landed
✅ Public README enables <30-min stranger setup
✅ ExplainThis hover covers every non-obvious UI element
✅ Dry-run mode for first Stitch drafts
✅ Sample Stitches tour project available
✅ Agent pre-priming with operator's actual data
✅ Repository documentation complete (README, AGENTS.md, plan, concepts, operations, troubleshooting)

## Technical Implementation Notes

### Onboarding State Storage
- Onboarding state persisted in `fleet.db` `ui_state` table
- Keys: `prompts_enabled`, `prompts_dismissed`, `last_seen_version`
- Feature usage timestamps: `agent_first_used`, `mic_first_used`, `patterns_first_used`, `reflection_ledger_first_used`

### Tour Project Implementation
- Tour workspace created at `~/.hoop/tour/` by default
- Four synthetic Stitches inserted into `stitches` table
- Tour project appears as special card with purple color
- Full cleanup on disable (database entries + filesystem)

### ExplainThis Glossary
- Centralized in `UI_GLOSSARY` constant
- Schema includes: id, label, explanation, category
- Component provides tooltip and inline modes
- HOC pattern available: `withExplainThis`

## Conclusion

§12 Onboarding & documentation is **COMPLETE**. All three onboarding surfaces (CLI wizard, in-UI first-run experience, progressive capability introduction) are implemented. All specific onboarding aids (ExplainThis hover, dry-run mode, sample tour project, agent pre-priming) are in place. All repository documentation (README, AGENTS.md, plan, concepts, operations, troubleshooting) is present and comprehensive.

The onboarding system follows all five principles: progressive, opt-out, operator-specific, re-playable, and LLM-first. Operators can discover HOOP's capabilities at their own pace without being overwhelmed on day one.

---

**Plan Reference:** docs/plan/plan.md §12
**Related Beads:** hoop-ttb.5 (Phase 4), hoop-ttb.11 (Phase 5), hoop-ttb.17 (Phase 6)
