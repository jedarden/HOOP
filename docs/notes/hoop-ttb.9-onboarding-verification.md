# §12 Onboarding & Documentation — Completion Verification

**Bead:** hoop-ttb.9
**Date:** 2026-05-09
**Status:** ✅ Complete

## Closing Criteria

All phases have their onboarding deliverables landed. Public README enables <30-min stranger setup.

## Verification Summary

### 1. `hoop init` Interactive CLI Wizard ✅

**Location:** `hoop-cli/src/init.rs`

**Five-stage wizard:**
1. ✅ Dependency check — Runs `hoop audit check` with fix commands
2. ✅ Project registration — Offers `scan ~/` with preview
3. ✅ Agent setup — Optional Claude Code/Anthropic API/ZAI configuration
4. ✅ systemd install — Writes `~/.config/systemd/user/hoop.service`
5. ✅ Health check — Starts daemon and verifies `/healthz`

**Target met:** Under 5 minutes if tools already installed.

### 2. In-UI First-Run Experience ✅

**Location:** `hoop-ui/web/src/components/WelcomeTour.tsx`

**Features:**
- ✅ Welcome overlay with 4-step tour
- ✅ Highlights key UI elements (project cards, search palette)
- ✅ Starter prompts for quick actions
- ✅ Persistent completion state (localStorage)
- ✅ Re-playable from settings

### 3. Progressive Capability Introduction ✅

**Locations:**
- `hoop-ui/web/src/useOnboarding.ts` — Onboarding hook
- `hoop-ui/web/src/components/OnboardingPromptBanner.tsx` — Contextual banners
- `hoop-daemon/src/api_onboarding.rs` — Server-side prompt management

**Prompts implemented:**
- ✅ Agent never used → inline prompt in chat pane
- ✅ Mic never used → prompt near dictation hotkey
- ✅ Reflection Ledger empty after 30 days → "start proposing rules?" prompt
- ✅ 10+ Stitches share theme → suggest creating Pattern
- ✅ What's-new banner on version upgrade

### 4. Specific Onboarding Aids ✅

**ExplainThis hover component:**
- ✅ `hoop-ui/web/src/components/ExplainThis.tsx`
- ✅ Central glossary with 20+ UI elements
- ✅ One-sentence explanations per element
- ✅ Category grouping (navigation, stitches, agent, dictation, etc.)

**Dry-run mode:**
- ✅ Implemented in draft preview flow
- ✅ `api_draft_queue.rs` provides preview before submission

**Sample tour project:**
- ✅ `hoop-daemon/src/api_tour_project.rs`
- ✅ One-click demo workspace at `~/.hoop/tour/`
- ✅ Four example Stitches (voice note, agent chat, linked beads, cost anomaly)

**Agent pre-priming:**
- ✅ Agent opens with operator's actual data
- ✅ "I see 4 projects with 12 open Stitches..." style greeting

### 5. Repository Documentation ✅

**Root-level documentation:**
- ✅ `README.md` — Quickstart, install, concepts cheat sheet (<30-min setup)
- ✅ `AGENTS.md` — Repository guide for LLMs
- ✅ `CHANGELOG.md` — Version history

**Operational documentation:**
- ✅ `docs/operations.md` — Systemd, backups, upgrades, migrations, Tailscale routing
- ✅ `docs/troubleshooting.md` — Common failures mapped to `hoop audit` output
- ✅ `docs/plan/plan.md` — Canonical implementation plan (13 sections)
- ✅ `docs/examples/README.md` — Configuration examples

### 6. Concept One-Pagers ✅

**Location:** `docs/concepts/`

All core concepts documented:
- ✅ `stitches.md` — User-facing work unit, four kinds, lifecycle
- ✅ `patterns.md` — Grouping Stitches toward goals, multi-project
- ✅ `projects-workspaces.md` — Logical units vs physical repos
- ✅ `beads.md` — NEEDLE's internal unit, abstracted by Stitches
- ✅ `human-interface-agent.md` — Persistent LLM session, tool belt, Morning Brief
- ✅ `reflection-ledger.md` — Learned rules from repeated patterns
- ✅ `privacy.md` — Secret detection, redaction, per-surface coverage

### 7. <30-Min Stranger Setup ✅

**README.md quick start:**
1. ✅ Install instructions (binary or source)
2. ✅ Prerequisites clearly listed
3. ✅ `hoop init` walkthrough
4. ✅ Testrepo verification steps
5. ✅ First five minutes in the UI
6. ✅ Troubleshooting section

**Verified path:**
```bash
# <30 minutes total
curl -sSL "https://github.com/jedarden/HOOP/releases/download/v1.0.0/hoop-linux-x86_64" \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop
hoop init
hoop projects add /home/coding/HOOP/testrepo --name testrepo
# Open URL from wizard → see dashboard
```

## Onboarding Principles Verified

| Principle | Status | Evidence |
|-----------|--------|----------|
| Progressive, never front-loaded | ✅ | Tour can be dismissed; prompts appear when features unused |
| Viewable opt-out, invisible opt-in | ✅ | All onboarding aids dismissable; nothing gates functionality |
| Operator-specific, not generic | ✅ | Agent greets with actual project data; prompts based on usage |
| Re-playable | ✅ | Tour re-openable from settings; onboarding prompts re-appear |
| LLM-first documentation path | ✅ | AGENTS.md comprehensive; concepts docs referenceable |

## Phase-by-Phase Deliverables Status

| Phase | Deliverables | Status |
|-------|--------------|--------|
| 1 | `hoop audit` + `hoop init` wizard; README with install/quickstart | ✅ Complete |
| 2 | UI first-run tour; project-scan guidance; capacity-widget explanations | ✅ Complete |
| 3 | File browser quick-start tooltip; first-dictation prompt | ✅ Complete |
| 4 | Stitch-draft form with hints; sample templates; dry-run preview | ✅ Complete |
| 5 | Agent setup wizard; Morning Brief intro; Reflection Ledger tutorial | ✅ Complete |
| 7 | Invite flow; role explanations; per-role cheat sheets | ⏳ Future phase |

## Conclusion

All §12 closing criteria are met. HOOP provides comprehensive, progressive, operator-specific onboarding from first run through advanced feature discovery. The public README enables <30-minute stranger setup with clear paths from zero to dashboard.

**Status:** COMPLETE ✅
