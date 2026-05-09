# §12 Onboarding & Documentation — Verification Summary

## Task: hoop-ttb.9

**Date:** 2026-05-09
**Status:** ✅ Complete

## Overview

§12 is a cross-cutting onboarding and documentation concern that delivers alongside each new surface. All deliverables are implemented and verified.

## Closing Criteria Verification

### 1. `hoop init` interactive CLI wizard ✅

**Location:** `hoop-cli/src/init.rs`

**Five-stage wizard:**
1. **Dependency check** — Runs `hoop audit check` and reports failures with fix commands
2. **Project registration** — Offers `scan ~/` with preview of discovered bead workspaces
3. **Agent setup** — Optional configuration for Claude Code, Anthropic API, or ZAI adapter
4. **systemd install** — Writes `~/.config/systemd/user/hoop.service`
5. **Health check** — Starts daemon and verifies `/healthz` endpoint

**Verification:**
```bash
hoop init
```

Each stage can be skipped if already configured. Re-running is idempotent.

### 2. In-UI first-run experience ✅

**Location:** `hoop-ui/web/src/components/WelcomeTour.tsx`

**Features:**
- Welcome overlay with 4-step tour
- Highlights key UI elements (project cards, search palette)
- Starter prompts for quick actions (enable tour, dictate note, register project, ask agent)
- Persistent completion state (localStorage)
- Re-playable from settings (WelcomeTourTrigger component)
- Dismissible with Escape key or × button

**Verification:**
- Fresh install shows tour automatically
- "Show Tour" button in settings replays anytime
- All steps dismissable

### 3. Progressive capability introduction ✅

**Locations:**
- `hoop-ui/web/src/useOnboarding.ts` — Hook for onboarding prompts
- `hoop-ui/web/src/components/OnboardingPromptBanner.tsx` — Contextual banners
- `hoop-daemon/src/api_onboarding.rs` — Server-side prompt management

**Features:**
- Agent never used → inline prompt in chat pane
- Mic never used → prompt near dictation hotkey
- Reflection Ledger empty after 30 days → "start proposing rules?" prompt
- 10+ Stitches share theme → suggest creating Pattern
- What's-new banner on version upgrade (WhatsNewBanner component)

**Verification:**
```bash
curl http://localhost:3000/api/onboarding/prompts
```

### 4. Specific onboarding aids ✅

#### Explain-this hover component
**Location:** `hoop-ui/web/src/components/ExplainThis.tsx`

- Central `UI_GLOSSARY` with 20+ UI element definitions
- Tooltip and inline modes
- HOC wrapper (`withExplainThis`)
- Hook for custom implementations (`useExplanation`)

#### Dry-run mode for first Stitch drafts
**Location:** `hoop-ui/web/src/StitchDraftForm.tsx:719-723`

```typescript
const [isDryRun, setIsDryRun] = useState(() => {
  const hasCreatedStitchBefore = localStorage.getItem('hoop_has_created_stitch');
  return !hasCreatedStitchBefore;
});
```

- Enabled by default for first-time users
- "Preview Only (Dry-run Mode)" button text
- Operator gets comfortable before committing

#### Sample tour project
**Location:** `hoop-daemon/src/api_tour_project.rs`

- One-click demo workspace at `~/.hoop/tour/`
- Four example Stitches:
  - Voice note demo (dictated)
  - Agent chat demo (operator)
  - Linked beads demo (ad-hoc)
  - Cost anomaly demo (worker)
- Removable in one click
- Tour project card with purple accent

**Verification:**
```bash
curl -X POST http://localhost:3000/api/tour/enable
curl http://localhost:3000/api/tour/status
curl -X DELETE http://localhost:3000/api/tour/disable
```

#### Agent pre-priming
**Location:** `hoop-daemon/src/agent_context.rs`

- `ContextIndex` contains operator's actual data:
  - Project names and paths
  - Recent activity summary (closed Stitches)
  - Open Stitch titles
  - Active alerts
  - Fleet notifications
- Agent opens with personalized context like "I see 4 projects with 12 open Stitches across them; what's on your mind?"

### 5. Repository documentation ✅

**Files:**
- `README.md` — Quickstart, install, concepts cheat sheet (<30-min stranger setup)
- `AGENTS.md` — Repository guide for LLMs (terminology, non-goals, conventions)
- `docs/operations.md` — Systemd, backups, upgrades, migrations
- `docs/troubleshooting.md` — Common failures mapped to `hoop audit` output
- `docs/plan/plan.md` — Canonical implementation plan (13 sections)
- `docs/examples/README.md` — Configuration examples with common patterns

**Verification:**
```bash
# Stranger can install and run in <30 minutes
curl -sSL "https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64" -o /tmp/hoop
chmod +x /tmp/hoop
/tmp/hoop init
```

### 6. Concept one-pagers ✅

**Location:** `docs/concepts/`

**Files:**
- `stitches.md` — User-facing work unit, four kinds, lifecycle
- `patterns.md` — Grouping Stitches toward goals, multi-project
- `projects-workspaces.md` — Logical units vs physical repos
- `beads.md` — NEEDLE's internal unit, abstracted by Stitches
- `human-interface-agent.md` — Persistent LLM session, tool belt, Morning Brief
- `reflection-ledger.md` — Learned rules from repeated patterns
- `privacy.md` — Secret detection, redaction, per-surface coverage

**Verification:**
```bash
ls docs/concepts/
# beads.md  human-interface-agent.md  privacy.md  projects-workspaces.md  reflection-ledger.md  stitches.md  patterns.md
```

## Onboarding Principles (from plan §12)

All principles are followed:

- **Progressive, never front-loaded** — Each concept introduced only when relevant
- **Viewable opt-out, invisible opt-in** — All onboarding aids are dismissable; none gate functionality
- **Operator-specific, not generic** — Agent greeting uses operator's actual data
- **Re-playable** — Every tour/tutorial can be re-opened from settings
- **LLM-first documentation path** — AGENTS.md provides immediate value to LLM contributors

## Additional Verification Commands

```bash
# 1. Test init wizard (fresh install)
rm -rf ~/.hoop  # CAUTION: deletes all HOOP state
hoop init

# 2. Verify tour project
curl -X POST http://localhost:3000/api/tour/enable
curl http://localhost:3000/api/tour/status | jq

# 3. Check onboarding prompts
curl http://localhost:3000/api/onboarding/prompts | jq

# 4. Verify concept docs exist
ls -1 docs/concepts/

# 5. Test stranger setup time
time (
  HOOP_VERSION="1.0.0"
  curl -sSL "https://github.com/jedarden/HOOP/releases/download/v${HOOP_VERSION}/hoop-linux-x86_64" -o /tmp/hoop
  chmod +x /tmp/hoop
  /tmp/hoop init
)
```

## Documentation References

- **Plan §12:** `docs/plan/plan.md` lines 1062-1131
- **Operations verification:** `docs/operations.md` lines 1941-2121
- **AGENTS.md:** Repository guide for LLMs
- **README.md:** Quickstart and concepts

## Conclusion

All §12 closing criteria are met. HOOP provides progressive, operator-specific onboarding from first run through advanced feature discovery. The documentation enables <30-min stranger setup, and all onboarding aids are dismissable and re-playable.

**Status:** ✅ §12 Complete
