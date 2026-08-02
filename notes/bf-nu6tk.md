# Clippy Correctness Warnings Verification (bf-nu6tk)

**Date:** 2026-08-01
**Status:** ❌ ACCEPTANCE CRITERIA NOT MET
**Command:** `cargo clippy --all-targets`

## Task
Run clippy and verify all correctness warnings are fixed.

## Current Status
**28 disallowed method warnings remain** - these are restriction/correctness warnings that prevent crash-unsafe file operations.

### Correctness Warnings Summary

| Warning Type | Count | Status |
|--------------|-------|--------|
| `await_holding_lock` | 0 | ✅ Eliminated |
| `unnecessary_cast` | 0 | ✅ Eliminated |
| `useless_conversion` | 0 | ✅ Eliminated |
| `clone_on_copy` | 0 | ✅ Eliminated |
| `disallowed_methods` | **28** | ❌ **Remains** |

### Disallowed Method Warnings Breakdown

All warnings are for non-atomic file operations that should use `atomic_write` instead:
- `std::fs::write` (22 instances)
- `std::fs::File::create` (6 instances)

### File Locations (28 total)

#### hoop-daemon/src/atomic_write.rs
- Line 97: `File::create` (internal use - acceptable)
- Line 188: `File::create` (internal use - acceptable)

#### hoop-daemon/src/attachment_sync.rs
- Line 86: `std::fs::write`

#### hoop-daemon/src/attachments.rs
- Line 613: `std::fs::write`

#### hoop-daemon/src/backup_pipeline.rs
- Line 561: `std::fs::write`

#### hoop-daemon/src/log_rotation.rs
- Line 110: `File::create`
- Line 117: `File::create`
- Line 145: `File::create`

#### hoop-daemon/src/metrics.rs
- Line 1336: `std::fs::write`

#### hoop-daemon/src/parse_jsonl_safe.rs
- Line 236: `fs::write`

#### hoop-daemon/src/projects.rs
- Line 113: `fs::write`

#### hoop-daemon/src/template_library.rs
- Line 424: `std::fs::write`

#### hoop-daemon/src/uploads.rs
- Line 132: `fs::write`
- Line 190: `File::create`
- Line 446: `fs::write`
- Line 470: `fs::write`
- Line 516: `fs::write`
- Line 540: `fs::write`

#### hoop-daemon/src/api_screen_capture.rs
- Line 149: `std::fs::write`
- Line 164: `std::fs::write`
- Line 214: `std::fs::write`

#### hoop-daemon/src/screen_capture.rs
- Line 353: `fs::write`
- Line 358: `File::create`
- Line 418: `fs::write`
- Line 452: `fs::write`
- Line 494: `fs::write`

## Policy Context

Per `hoop-daemon/src/atomic_write.rs`:

> All critical filesystem writes in hoop-daemon MUST use the atomic write pattern:
> write to a temporary `.tmp` file, fsync, then rename into place. This ensures that
> crashes never leave partially-written files visible to readers.

**Do NOT use directly:**
- `std::fs::write()` — no fsync, can leave partial data on crash
- `File::create()` + `write_all()` without explicit `sync_all()` — not crash-safe

**Required fix pattern:**
```rust
use hoop_daemon::atomic_write::atomic_write_file;
use hoop_daemon::atomic_write::atomic_write_file_str;

// Replace fs::write(&path, data)
atomic_write_file(&path, data.as_bytes())?;

// Replace fs::write(&path, string)
atomic_write_file_str(&path, string)?;
```

## Required Actions

Each of the 26 non-internal instances needs to be replaced with atomic write operations:
- Use `atomic_write::atomic_write_file` or `atomic_write::atomic_write_file_str` instead of `std::fs::write`
- Use `atomic_write::atomic_write_file` instead of `std::fs::File::create`

This ensures crash-safe writes using the tmp + fsync + rename pattern.

## Files Requiring Updates (Priority Order)

**High priority (data integrity risk):**
- `screen_capture.rs` (5 warnings) - video/metadata writes
- `uploads.rs` (5 warnings) - file upload handling
- `api_screen_capture.rs` (3 warnings) - API write operations
- `log_rotation.rs` (3 warnings) - log file rotation

**Medium priority:**
- `attachments.rs`, `attachment_sync.rs`, `backup_pipeline.rs` - data writes
- `projects.rs`, `template_library.rs` - configuration writes

**Low priority (internal use):**
- `atomic_write.rs` (2 warnings) - internal implementation details (acceptable as-is)

## Acceptance Criteria

**Required:** Clippy passes with zero correctness warnings
**Actual:** 28 `disallowed_methods` warnings remain (26 requiring fixes)
**Status:** ❌ NOT MET

## Progress

- Previous analysis found 39 warnings
- Current run found 28 warnings
- **11 warnings already fixed** ✅
- **26 warnings remaining** ❌
