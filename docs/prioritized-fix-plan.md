# HOOP Prioritized Fix Plan

**Created:** 2026-07-02  
**Status:** Active  
**Context:** Phase 1 CI gate (bf-5mpcl) is open; 36 compilation errors block progress; security bug in backup pipeline.

---

## Executive Summary

HOOP has **36 compilation errors** (all OpenAPI ToSchema trait violations) and **~150 clippy warnings** (unused imports, variables, dead code) blocking Phase 1 CI gate. A **security bug** in backup encryption also exists but is independent of compilation.

**Critical Path:** Fix compilation → Fix clippy → Verify tests → Close Phase 1 CI gate

---

## Priority Matrix

| Priority | Issue Type | Blocking Phase 1? | Security Impact | Count |
|----------|------------|-------------------|-----------------|-------|
| **P0** | Compilation errors | Yes | None | 36 errors across 4 structs |
| **P1** | Non-interactive mode | Yes | None | 1 bead (bf-18rrg) |
| **P1-S** | Backup encryption bug | No | **HIGH** | 1 bead (bf-5lv71) |
| **P2** | Clippy warnings | Yes | None | ~150 warnings |
| **P2** | Dead code warnings | Yes | None | ~50 warnings |

---

## Phase 1 Exit Gates (Blocking All Phase 2+ Work)

Per plan §10, Phase 1 cannot close until ALL of these pass on the same commit:

1. ✅ `cargo test` — all unit + integration tests green
2. ❌ `cargo clippy -- -D warnings` — zero warnings
3. ❌ `hoop status --json | jq .` — non-interactive mode verified
4. ❌ Phase 1 success criteria have passing automated tests

**Current State:** Gates 2-4 are OPEN (failing). Gate 1 status unknown due to compilation failures.

---

## Detailed Fix Plan (Numbered by Execution Order)

### 🔴 PRIORITY 0: Fix Compilation Errors (Blocks Everything)

**Rationale:** Cannot run clippy, tests, or build binaries while compilation fails. These 36 errors are all `utoipa::ToSchema` trait violations on structs used in OpenAPI documentation.

**Bead:** `bf-2e233` (Fix 54 OpenAPI ToSchema compilation errors) — **CURRENTLY 36 ERRORS** (some may have been fixed already)

**Affected Files:**
1. `hoop-daemon/src/api_transcription.rs:19` — `ListJobsQuery`
2. `hoop-daemon/src/api_screen_capture.rs:34` — `CreateScreenCaptureRequest`
3. `hoop-daemon/src/api_screen_capture.rs:352` — `StartStreamingUploadRequest`
4. `hoop-daemon/src/api_screen_capture.rs:469` — `CompleteStreamingUploadRequest`

**Fix Approach (per bf-jmb87 analysis):**
```rust
// For each of the 4 structs, add the derive:
#[derive(ToSchema)]
pub struct ListJobsQuery {
    // existing fields...
}

#[derive(ToSchema)]
struct CreateScreenCaptureRequest {
    // existing fields...
}
// ... and so on for StartStreamingUploadRequest, CompleteStreamingUploadRequest
```

**Verification:**
```bash
nix-shell --run 'cargo check 2>&1 | grep "^error" | wc -l'
# Expected: 0
```

**Estimated Effort:** 30 minutes  
**Depends On:** Nothing  
**Unlocks:** Clippy, tests, binary builds

---

### 🟡 PRIORITY 1: Fix Non-Interactive Mode (Phase 1 Gate Requirement)

**Rationale:** Phase 1 exit gate §S6 requires `hoop status --json | jq .` to work without prompts. This is a core deliverable.

**Bead:** `bf-18rrg` (Make hoop status --json work non-interactively)

**Investigation Needed:**
- Why does `hoop status --json` currently prompt?
- Is it a tty detection issue? A missing `--yes` flag?
- Does the CLI struct lack a `--json` field?

**Fix Approach:**
1. Add `--json` flag to `hoop status` command struct
2. Ensure tty detection correctly identifies non-interactive mode
3. Verify output is valid JSON pipeable to `jq`

**Verification:**
```bash
./target/debug/hoop status --json | jq .
# Expected: valid JSON, exit 0
```

**Estimated Effort:** 1 hour  
**Depends On:** Compilation fixes (P0)  
**Unlocks:** Phase 1 gate requirement §S6

---

### 🟢 PRIORITY 1-S: Fix Backup Encryption Security Bug (Independent)

**Rationale:** **SECURITY CRITICAL** — backup pipeline silently uploads UNENCRYPTED snapshots when age encryption fails (e.g., missing `HOOP_BACKUP_AGE_KEY`). This is a policy downgrade violation that exposes sensitive data (stitches, transcripts, audit logs).

**Bead:** `bf-5lv71` (Backup pipeline silently uploads UNENCRYPTED snapshot when age encryption fails)

**Bug Location:** `hoop-daemon/src/backup_pipeline.rs:175-190`

**Current Behavior:**
```rust
// When config.encryption == true and age_encrypt() fails:
warn!("Age encryption failed, uploading unencrypted: {}", e);
// PROCEEDS TO UPLOAD PLAINTEXT
```

**Required Fix:**
```rust
// When config.encryption == true and encryption fails:
// 1. FAIL the backup run (do not upload anything)
// 2. Surface E6-003 error code
// 3. Increment hoop_errors_total{subsystem="backup"}
// 4. Leave hoop_backup_last_success_timestamp untouched
// 5. Record failure in backup audit rows
```

**Test Cases Required:**
1. `encryption=true` + no age key → run fails, nothing uploaded
2. `encryption=true` + valid key → encrypted upload works
3. `encryption=false` → plaintext upload works

**Estimated Effort:** 2 hours  
**Depends On:** Nothing (independent of compilation)  
**Security Impact:** HIGH — violates explicit operator intent

---

### 🔵 PRIORITY 2: Fix Clippy Warnings (Phase 1 Gate Requirement)

**Rationale:** Phase 1 exit gate requires `cargo clippy -- -D warnings` to exit clean. Currently ~150 warnings (unused imports, unused variables, dead code).

**Beads:**
- `bf-19qea` (Clean up 70+ unused imports across workspace)
- `bf-1siii` (Fix unused_* warnings)
- `bf-1ygdz` (Fix dead_code warnings in data structures)
- `bf-2jpsl` (Fix dead_code warnings in functions and methods)
- `bf-171hj` (Run final clippy verification)

**Warning Categories:**
1. **Unused imports** (~70 occurrences) — auto-fixable with `cargo clippy --fix`
2. **Unused variables** (~33 occurrences) — prefix with `_` or remove
3. **Dead code** (~50 occurrences) — remove or add `#[allow(dead_code)]`
4. **Unnecessary mut** (~3 occurrences) — remove `mut` keyword

**Fix Approach:**
```bash
# Step 1: Auto-fix what clippy can
nix-shell --run 'cargo clippy --fix --allow-dirty -- -D warnings'

# Step 2: Manual cleanup for remaining warnings
nix-shell --run 'cargo clippy -- -D warnings 2>&1 | grep "^warning:"'

# Step 3: Verify clean
nix-shell --run 'cargo clippy -- -D warnings 2>&1 | grep "^error" | wc -l'
# Expected: 0
```

**Estimated Effort:** 2 hours  
**Depends On:** Compilation fixes (P0)  
**Unlocks:** Phase 1 gate requirement #2

---

### ⚪ PRIORITY 3: Verify Tests and Close Phase 1 CI Gate

**Rationale:** Once compilation and clippy are clean, run the full test suite to verify Phase 1 deliverables.

**Bead:** `bf-5mpcl` (Phase 1 CI gate)

**Verification Steps:**
```bash
# 1. All tests pass
nix-shell --run 'cargo test --workspace 2>&1 | tail -20'

# 2. Clippy clean
nix-shell --run 'cargo clippy --workspace -- -D warnings'

# 3. Non-interactive mode works
./target/debug/hoop status --json | jq .

# 4. Phase 1 success criteria have tests
# (Check testrepo/ fixture coverage)
```

**Estimated Effort:** 1 hour  
**Depends On:** P0, P1, P2  
**Unlocks:** Phase 2 work (currently BLOCKED by phase sequence lock)

---

## Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                    PRIORITY 0: Compilation                   │
│            (bf-2e233: Fix 36 ToSchema errors)                │
└───────────────────────────┬─────────────────────────────────┘
                            │
        ┌───────────────────┴───────────────────┐
        │                                       │
        ▼                                       ▼
┌───────────────────────┐           ┌──────────────────────────┐
│ PRIORITY 1:           │           │ PRIORITY 1-S:             │
│ Non-interactive mode  │           │ Security bug (backup)     │
│ (bf-18rrg)            │           │ (bf-5lv71) — INDEPENDENT  │
└───────────┬───────────┘           └──────────────────────────┘
            │
            ▼
┌───────────────────────┐
│ PRIORITY 2:           │
│ Clippy warnings       │
│ (bf-19qea, etc.)      │
└───────────┬───────────┘
            │
            ▼
┌───────────────────────┐
│ PRIORITY 3:           │
│ Verify tests +        │
│ Close Phase 1 gate    │
│ (bf-5mpcl)            │
└───────────────────────┘
```

---

## Execution Timeline (Recommended)

| Priority | Task | Estimated Time | Run Order |
|----------|------|----------------|------------|
| **P0** | Fix ToSchema compilation errors | 30 min | 1st |
| **P1-S** | Fix backup encryption bug | 2 hours | Can run in parallel with P0 |
| **P1** | Fix non-interactive mode | 1 hour | 2nd (after P0) |
| **P2** | Fix clippy warnings | 2 hours | 3rd (after P0) |
| **P3** | Verify tests and close Phase 1 | 1 hour | 4th (after P0+P1+P2) |

**Total Critical Path:** ~4.5 hours  
**With Parallel (P0 + P1-S):** ~2.5 hours wall time

---

## Phase Sequence Lock Warning

**⚠️ CRITICAL:** Per plan §10 and AGENTS.md, **DO NOT implement Phase 2+ features until Phase 1 CI gate (bf-5mpcl) passes.**

The phase sequence lock is strict:
- Phase N may not begin until Phase N-1 exit gates pass
- No partial phase completion
- Deliverables move intact, not half-finished

**Current Phase 1 Status:**
- ❌ Compilation fails (36 errors)
- ❌ Clippy warnings (~150)
- ❌ Non-interactive mode unverified
- ❌ Tests unverified

**Phase 2–7 Code Status:** EXISTS BUT UNVERIFIED. Do not trust code in later phases until Phase 1 gates pass.

---

## Bead Dependency Mapping

| Bead ID | Title | Priority | Blocks / Blocked By |
|---------|-------|----------|-------------------|
| bf-2e233 | Fix 54 OpenAPI ToSchema errors | P2 | BLOCKS: bf-5mpcl, bf-171hj, all clippy beads |
| bf-5lv71 | Backup encryption security bug | P2 | BLOCKS: None (independent) |
| bf-18rrg | Non-interactive mode | P1 | BLOCKS: bf-5mpcl |
| bf-19qea | Clean up 70+ unused imports | P2 | BLOCKED BY: bf-2e233 |
| bf-1siii | Fix unused_* warnings | P2 | BLOCKED BY: bf-2e233 |
| bf-1ygdz | Fix dead_code warnings | P2 | BLOCKED BY: bf-2e233 |
| bf-2jpsl | Fix dead_code in functions | P2 | BLOCKED BY: bf-2e233 |
| bf-171hj | Run final clippy verification | P2 | BLOCKED BY: bf-19qea, bf-1siii, bf-1ygdz, bf-2jpsl |
| bf-5mpcl | Phase 1 CI gate | P1 | BLOCKED BY: bf-2e233, bf-18rrg, bf-171hj |

---

## Acceptance Criteria for Full Fix

When all priorities are complete, the following must pass:

```bash
# 1. Compilation clean
nix-shell --run 'cargo check 2>&1 | grep "^error" | wc -l'
# Output: 0

# 2. Clippy clean
nix-shell --run 'cargo clippy --workspace -- -D warnings 2>&1 | grep "^error" | wc -l'
# Output: 0

# 3. Non-interactive mode works
./target/debug/hoop status --json | jq . > /dev/null && echo $?
# Output: 0

# 4. Backup encryption fails securely
# (Test case: encryption=true + no age key → run fails, nothing uploaded)

# 5. All tests pass
nix-shell --run 'cargo test --workspace 2>&1 | tail -5'
# Output: test result: ok. 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## Related Documentation

- **Plan:** `docs/plan/plan.md` — Phase 1 deliverables and exit gates (§10, §S6)
- **Phase 1 Gap Analysis:** `docs/phase1-gap-analysis.md` — Detailed gap analysis
- **AGENTS.md:** — Phase sequence lock warning, non-goals, vocabulary guard
- **Build Notes:** `notes/bf-jmb87.md` — ToSchema compilation error analysis

---

## Change History

| Date | Change | Author |
|------|--------|--------|
| 2026-07-02 | Initial plan created | bf-fwva2 task |

---

**Next Action:** Execute PRIORITY 0 (Fix ToSchema errors) → `bf claim bf-2e233`
