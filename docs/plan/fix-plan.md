# HOOP Compilation Issues — Prioritized Fix Plan

**Date:** 2026-07-02
**Status:** Phase 1 blocked on compilation errors
**Umbrella Bead:** bf-5mpcl (Phase 1 CI gate)

---

## Executive Summary

**Total Open Issues:** 69 beads (P0-P2)
**Blocking Compilation:** 22 errors (14 trait bound + 8 remaining)
**Non-blocking Warnings:** 74 clippy warnings

**Critical Path:** Fix compilation errors → Pass `cargo build` → Fix clippy warnings → Pass `cargo clippy -- -D warnings` → Pass `cargo test`

---

## Fix Priorities by Tier

### TIER 0: Unblock Compilation (MUST FIX FIRST)

These fixes are **required** before any other work can proceed. All Tier 0 fixes block the Phase 1 CI gate.

| Priority | Issue | Files | Error Count | Fix Complexity | Dependencies |
|----------|-------|-------|-------------|----------------|--------------|
| **0-A** | Missing `ToSchema` derives | api_transcription.rs, api_screen_capture.rs | 14 | LOW (1 line each) | None |
| **0-B** | Function arity mismatches | lib.rs, secrets_scanner.rs | 2 | LOW (rename/call fix) | None |
| **0-C** | Missing struct field | config_resolver.rs, lib.rs | 2 | MEDIUM (add field) | None |
| **0-D** | Async/await mismatch | lib.rs | 2 | MEDIUM (add `.await`) | None |
| **0-E** | Missing function | secrets_scanner.rs, lib.rs | 2 | LOW (add function) | None |

**Estimated Time:** 1-2 hours
**Blocks:** All Tier 1 and Tier 2 work

---

#### TIER 0-A: Trait Bound Violations (14 errors)

**Pattern:** OpenAPI `utoipa` code generation requires structs to derive `ToSchema`

| Struct | Location | Line | Fix |
|--------|----------|------|-----|
| `ListJobsQuery` | hoop-daemon/src/api_transcription.rs | 19 | `#[derive(ToSchema)]` |
| `CreateScreenCaptureRequest` | hoop-daemon/src/api_screen_capture.rs | 34 | `#[derive(ToSchema)]` |
| `StartStreamingUploadRequest` | hoop-daemon/src/api_screen_capture.rs | 352 | `#[derive(ToSchema)]` |
| `CompleteStreamingUploadRequest` | hoop-daemon/src/api_screen_capture.rs | 469 | `#[derive(ToSchema)]` |

**Fix:**
```rust
// Add to each struct
#[derive(Deserialize, Serialize, ToSchema)]  // Add ToSchema here
pub struct StructName {
    // fields...
}
```

**Verification:**
```bash
cargo check 2>&1 | grep -c "error\[E"
# Should drop from 22 to 8
```

---

#### TIER 0-B: Function Arity Mismatches (2 errors)

**Issue:** Functions called with wrong number of arguments

| Location | Issue |
|----------|-------|
| lib.rs (TBD from `cargo check`) | Function called with 2 args but takes 1 |
| lib.rs (TBD from `cargo check`) | Function called with 2 args but takes 1 |

**Fix:**
1. Run `cargo check 2>&1 | grep -A8 'error\[E0061\]'` to get exact locations
2. Adjust call sites to match function signatures

---

#### TIER 0-C: Missing Struct Field (2 errors)

**Issue:** `ResolvedConfig` missing `redaction` field referenced by `lib.rs`

| Location | Issue |
|----------|-------|
| config_resolver.rs | Missing `redaction: bool` field |
| lib.rs:1980, lib.rs:2024 | References `config.redaction` |

**Fix:**
```rust
// hoop-daemon/src/config_resolver.rs
pub struct ResolvedConfig {
    // ... existing fields
    pub redaction: bool,  // Add this field
}
```

**Verification:** Ensure all references to `config.redaction` are valid.

---

#### TIER 0-D: Async/Await Mismatch (2 errors)

**Issue:** Expected Future, found `()`

| Location | Issue |
|----------|-------|
| lib.rs (TBD from `cargo check`) | Missing `.await` on async call |
| lib.rs (TBD from `cargo check`) | Missing `.await` on async call |

**Fix:**
1. Run `cargo check 2>&1 | grep -A8 'error\[E0277\]'` to get exact locations
2. Add `.await` to async function calls

---

#### TIER 0-E: Missing Function (2 errors)

**Issue:** `update_per_project_patterns` not found in `secrets_scanner`

| Location | Issue |
|----------|-------|
| lib.rs:1980 | Calls `update_per_project_patterns` |
| lib.rs:2024 | Calls `update_per_project_patterns` |
| secrets_scanner.rs | Only has `update_patterns()` |

**Fix (Option A):** Rename call sites
```rust
// lib.rs
secrets_scanner::update_patterns(...)  // Remove _per_project suffix
```

**Fix (Option B):** Add missing function
```rust
// secrets_scanner.rs
pub fn update_per_project_patterns(...) {
    // delegate to update_patterns
    update_patterns(...)
}
```

**Recommendation:** Option A (rename call sites) is simpler.

---

### TIER 1: Clear Clippy Warnings (PARALLEL AFTER BUILD)

After `cargo build` succeeds, fix clippy warnings. These can be worked **in parallel** across multiple files.

| Priority | Category | Count | Auto-fixable | Files |
|----------|----------|-------|--------------|-------|
| **1-A** | Unused imports | 38 | YES (`clippy --fix`) | 24 files |
| **1-B** | Unused variables | 33 | PARTIAL | 18 files |
| **1-C** | Unnecessary mut | 3 | YES (manual) | 3 files |

**Estimated Time:** 2-3 hours (parallelizable)
**Blocks:** Phase 1 CI gate

---

#### TIER 1-A: Unused Imports (38 occurrences)

**Hotspot Files:**
- api_bead_files.rs (4)
- prompt_substitute.rs (3)
- cross_project_propagation.rs (2)
- api_fix_patterns.rs (2)

**Fix:**
```bash
# Auto-fix most
nix-shell --run 'cargo clippy --fix --allow-dirty'

# Manual cleanup for any remaining
cargo check 2>&1 | grep 'unused_import' | cut -d: -f1 | sort -u
```

---

#### TIER 1-B: Unused Variables (33 occurrences)

**Hotspot Files:**
- auth.rs (3)
- api_scripts.rs (3)
- api_skills.rs (3)
- cross_project_propagation.rs (4)
- lib.rs (4)

**Fix:**
```bash
# Auto-fix prefixes with underscore
nix-shell --run 'cargo clippy --fix --allow-dirty'

# Manual cleanup for remaining
# Prefix with underscore: _variable_name
```

---

#### TIER 1-C: Unnecessary Mut (3 occurrences)

| File | Line | Variable |
|------|------|----------|
| api_tour_project.rs | 240 | conn |
| api_fix_patterns.rs | 454 | conn |
| lib.rs | 3446 | shutdown_rx |

**Fix:** Remove `mut` keyword.

---

### TIER 2: Integration & Verification (AFTER TIER 0 + TIER 1)

| Priority | Task | Dependencies | Bead |
|----------|------|--------------|------|
| **2-A** | Verify `cargo build` succeeds | Tier 0 | bf-1cu |
| **2-B** | Run `cargo clippy -- -D warnings` | Tier 0 + Tier 1 | bf-4y3mc |
| **2-C** | Run `cargo test` | Tier 0 + Tier 1 | bf-5mpcl |
| **2-D** | Verify `hoop status --json` | Tier 0 + Tier 1 | bf-18rrg |

---

## Dependency Graph

```
Tier 0-A (ToSchema) ───┐
Tier 0-B (arity)      ├──→ cargo build succeeds ───┐
Tier 0-C (field)      │                               │
Tier 0-D (async)      │                               │
Tier 0-E (function)   ┘                               │
                                                      │
                                                      ├──→ Tier 1-A (imports) ──┐
                                                      │                        │
                                                      ├──→ Tier 1-B (vars) ────┤
                                                      │                        │
                                                      └──→ Tier 1-C (mut) ────┤
                                                                              ├──→ Tier 2-B (clippy clean) ──→ bf-5mpcl (CI gate)
                                                                              │
                                                                              └──→ Tier 2-C (tests pass) ────→ bf-5mpcl (CI gate)
```

---

## Execution Order (Sequential)

### Step 1: Fix Tier 0-A (ToSchema derives)
**Time:** 15 minutes
**Bead:** bf-67x

1. Edit `hoop-daemon/src/api_transcription.rs`:
   - Add `#[derive(ToSchema)]` to `ListJobsQuery` (line 19)

2. Edit `hoop-daemon/src/api_screen_capture.rs`:
   - Add `#[derive(ToSchema)]` to `CreateScreenCaptureRequest` (line 34)
   - Add `#[derive(ToSchema)]` to `StartStreamingUploadRequest` (line 352)
   - Add `#[derive(ToSchema)]` to `CompleteStreamingUploadRequest` (line 469)

3. Verify:
   ```bash
   nix-shell --run 'cargo check 2>&1 | grep -c "error\[E"'
   # Should be 8 (down from 22)
   ```

4. Commit:
   ```bash
   git add hoop-daemon/src/api_transcription.rs hoop-daemon/src/api_screen_capture.rs
   git commit -m 'fix(compile): add ToSchema derives for OpenAPI structs'
   ```

---

### Step 2: Fix Tier 0-B through 0-E
**Time:** 30-45 minutes
**Bead:** bf-67x

1. Run `cargo check` to get exact error locations:
   ```bash
   nix-shell --run 'cargo check 2>&1 | grep -A8 "error\["'
   ```

2. Fix each error group:
   - **0-B:** Adjust function call sites to match signatures
   - **0-C:** Add `redaction` field to `ResolvedConfig`
   - **0-D:** Add `.await` to async calls
   - **0-E:** Rename `update_per_project_patterns` to `update_patterns`

3. Verify:
   ```bash
   nix-shell --run 'cargo check'
   # Should exit 0
   ```

4. Commit:
   ```bash
   git add -u
   git commit -m 'fix(compile): resolve remaining 8 compile errors'
   ```

---

### Step 3: Fix Tier 1 Clippy Warnings (PARALLEL)
**Time:** 1-2 hours
**Beads:** bf-4y3mc, bf-2emzk, bf-5setb, bf-sux5h, bf-52yup (parallelizable)

1. Auto-fix imports and variables:
   ```bash
   nix-shell --run 'cargo clippy --fix --allow-dirty -- -D warnings'
   ```

2. Manual cleanup for remaining:
   ```bash
   nix-shell --run 'cargo clippy -- -D warnings 2>&1 | grep "warning:"'
   ```

3. Verify clean:
   ```bash
   nix-shell --run 'cargo clippy -- -D warnings 2>&1 | grep "^error" | wc -l'
   # Should be 0
   ```

4. Commit:
   ```bash
   git add -u
   git commit -m 'fix(clippy): resolve all clippy warnings'
   ```

---

### Step 4: Verify Tier 2 Exit Criteria
**Time:** 30 minutes
**Beads:** bf-5mpcl, bf-18rrg, bf-1cu

1. Build release binary:
   ```bash
   nix-shell --run 'cargo build --release'
   ```

2. Run tests:
   ```bash
   nix-shell --run 'cargo test --workspace'
   ```

3. Verify non-interactive mode:
   ```bash
   ./target/release/hoop status --json | jq .
   ```

4. Close umbrella bead bf-5mpcl when all pass.

---

## Rationale for Priority Order

### Why Tier 0 First?
- **Blocking:** All 22 compilation errors prevent `cargo build` from succeeding
- **Testability:** Cannot run tests or verify functionality without a working binary
- **Downstream:** Tier 1 and Tier 2 work require `cargo build` to succeed

### Why Tier 1 Parallel?
- **Independent:** Unused imports/variables in one file don't affect other files
- **Auto-fixable:** Most Tier 1 fixes can be done by `clippy --fix`
- **Time-saving:** Parallel execution across multiple agents reduces wall-clock time

### Why Tier 2 Last?
- **Gate-dependent:** Tier 2 is verification, not implementation
- **Requires:** All preceding fixes to be in place

---

## Success Metrics

| Metric | Current | Target | Command |
|--------|---------|--------|---------|
| `cargo check` errors | 22 | 0 | `nix-shell --run 'cargo check 2>&1 | grep "^error" | wc -l'` |
| `cargo clippy` warnings | 74 | 0 | `nix-shell --run 'cargo clippy -- -D warnings 2>&1 | grep "^error" | wc -l'` |
| `cargo test` | BLOCKED | 100% pass | `nix-shell --run 'cargo test 2>&1 | tail -5'` |
| `hoop status --json` | BLOCKED | Valid JSON | `./target/release/hoop status --json | jq .` |

---

## Notes

- **NixOS Requirement:** All `cargo` commands must be run via `nix-shell` due to `openssl-sys` dependency
- **Commit Requirements:** Each completed bead MUST produce at least one git commit (per HOOP workflow)
- **Bead Tracking:** The bead tracker (`.beads/`) is authoritative; `bf-5mpcl` is the umbrella bead for Phase 1 CI gate

---

**Document Owner:** bf-fwva2
**Last Updated:** 2026-07-02
**Next Review:** After Tier 0 fixes complete
