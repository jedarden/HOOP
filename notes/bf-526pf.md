# Clippy Verification Status — bf-526pf

**Date:** 2026-07-03
**Status:** FAILED — 176 clippy errors remain
**Acceptance:** 0 required
**Dependency:** bf-iwgtf (should have fixed all warnings)

## Command Run
```bash
cargo clippy --workspace -- -D warnings 2>&1 | grep '^error:' | wc -l
# Output: 176
```

## Error Categories (Detailed)

### 1. Dead Code (unused functions, structs, constants, fields)
**Functions:**
- `openapi_router` - unused function (hoop-daemon/src/lib.rs:1277)
- `load_hoop_config` - unused async function (hoop-daemon/src/lib.rs:3797)
- `check_and_emit_capacity_alert` - unused function (hoop-daemon/src/lib.rs:4074)
- `get_opencode_limits` - unused function (hoop-daemon/src/capacity.rs:472)

**Fields:**
- `session_id` - unread field (hoop-daemon/src/capacity.rs:358)
- `session_subpath` - unread field (hoop-daemon/src/capacity.rs:526)
- `rpm_limit` - unread field (hoop-daemon/src/capacity.rs:55)
- `subpath` - unread field (hoop-daemon/src/sessions.rs:557)

**Constants:**
- `MAX_UNASSIGNED_SESSIONS` - unused constant (hoop-daemon/src/sessions.rs:763)
- `MIN_SAMPLES_FOR_PREDICTION` - unused constant (hoop-daemon/src/stitch_percentile_index.rs:68)
- `STITCH_CLOSED_THRESHOLD_SECONDS` - unused constant (hoop-daemon/src/stitch_percentile_index.rs:72)

**Structs:**
- `QuotaLimit` - never constructed (hoop-daemon/src/capacity.rs:60)

### 2. Disallowed Methods (project lint rules)
**Multiple violations of custom lint rules:**
- `std::fs::write` - multiple instances blocked by project lint
- `std::fs::File::create` - multiple instances blocked by project lint

### 3. Simplification Opportunities
- **Manual `RangeInclusive::contains`** → use built-in method
- **`Iterator::last` on `DoubleEndedIterator`** → inefficient iteration, use `rev().next()`
- **Redundant closures** → remove unnecessary closures
- **`sort_by` → `sort_by_key`** → where applicable
- **`Option::and_then(|x| Some(y))` → `map(|x| y)`** → simplify
- **Clamp-like patterns** → use `clamp()` function
- **Useless `format!`** → use `to_string()` or literal
- **`map_or` simplifications** → can be simplified

### 4. Derivable Implementations
- Manual `impl` that can use `#[derive]` instead
- Missing `Default` implementation suggestion for `BackupManifest`

### 5. Reference Lifetime Issues
- Expressions creating references immediately dereferenced by compiler

### 6. Unnecessary Pattern Matching
- Unnecessary `if let` for `Ok` variants only

## Next Steps for Follow-up Bead

1. **Verify dependency status:** Check if `bf-iwgtf` completed — it should have fixed all these warnings
2. **Fix dead code:** Remove unused items or add `#[allow(dead_code)]` with documentation
3. **Replace disallowed methods:** Use project's atomic write pattern (see `atomic_write.rs`) instead of `std::fs::write` / `File::create`
4. **Apply clippy suggestions:** Simplify code patterns as recommended
5. **Add derives:** Use `#[derive]` where applicable instead of manual impls
6. **Re-verify:** Run same command until count reaches 0

## Context

This bead depends on `bf-iwgtf` which was supposed to fix all clippy warnings. The presence of 176 remaining errors suggests either:
- `bf-iwgtf` has not been completed yet
- The fixes applied were incomplete
- New code was added after the fixes

**Do not close this bead until acceptance criteria is met (0 errors).**
