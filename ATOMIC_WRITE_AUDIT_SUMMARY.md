# Atomic .tmp + Rename Invariant Audit Summary

**Date:** 2026-04-25
**Task:** hoop-ttb.11.6
**Scope:** Audit all filesystem writes in hoop-daemon for atomic write safety

## Audit Results

### 1. Centralized Helper Function ✓

The `atomic_write.rs` module exists with:
- `atomic_write_file(dest, data)` - Atomic write for raw bytes
- `atomic_write_file_str(dest, content)` - Convenience wrapper for strings
- `AtomicWriteBuilder` - For custom temp file naming

**Implementation:**
```rust
// Write pattern: tmp file → write_all → fsync → rename
pub fn atomic_write_file(dest: &Path, data: &[u8]) -> Result<()> {
    let tmp_name = format!("{}.{}.tmp", filename, uuid::Uuid::new_v4());
    let tmp_path = parent.join(tmp_name);
    let mut file = File::create(&tmp_path)?;
    file.write_all(data)?;
    file.sync_all()?;  // Critical: ensures data reaches disk
    std::fs::rename(&tmp_path, dest)?;  // Atomic
    Ok(())
}
```

### 2. Critical Write Paths Audit

All critical write paths use the atomic write helper:

| Write Path | File | Line | Status |
|------------|------|------|--------|
| Audio data storage | `dictated_notes.rs` | 187 | ✓ Uses `atomic_write_file` |
| Manifest save | `attachment_sync.rs` | 78 | ✓ Uses `atomic_write_file_str` |
| Backup compression | `backup_pipeline.rs` | 404 | ✓ Uses `atomic_write_file` |
| Projects registry | `projects.rs` | 117 | ✓ Uses `atomic_write_file_str` |
| Template seeding | `template_library.rs` | 424 | ✓ Uses `atomic_write_file_str` |
| Upload metadata | `uploads.rs` | 130 | ✓ Uses `atomic_write_file` |
| SVG sanitization | `uploads.rs` | 434, 446 | ✓ Uses `atomic_write_file` |

### 3. Clippy Lint Rule ✓

`clippy.toml` already configured with `disallowed-methods`:

```toml
disallowed-methods = [
    { path = "std::fs::write", reason = "Use atomic_write::atomic_write_file instead" },
    { path = "std::fs::File::create", reason = "Use atomic_write::atomic_write_file instead" },
]
```

### 4. Crash-Injection Tests ✓

Comprehensive tests in `atomic_write.rs` covering:

**5 crash points in the write pipeline:**
1. Before any write (tmp file not created)
2. During tmp file write (before close)
3. After write but before fsync
4. After fsync but before rename
5. After rename (atomic, so complete or not at all)

**5 critical write paths:**
1. Audio data storage (`dictated_notes::store_audio`)
2. Manifest save (`attachment_sync::BackupManifest::save`)
3. Backup compression (`backup_pipeline::zstd_compress`)
4. Projects registry write (`projects::write_back`)
5. Template library seed (`template_library::seed_examples`)

### 5. Improvements Made

**Fixed `log_rotation.rs`:**
- Changed `flush()` to use `sync_all()` for crash-safe logging
- Ensures log entries are fully written to disk before rotation

## Acceptance Criteria Status

- ✓ Grep/lint rule prevents direct writes (clippy.toml disallowed-methods)
- ✓ Crash-injection tests at 5 points; no partial files
- ✓ Helper function `atomic_write_file()` centralized in atomic_write.rs

## Exceptions Documented

The following uses of direct writes are intentional and documented:

1. **Log files** (`log_rotation.rs`) - Append-only writes, now with `sync_all()`
2. **Upload partial files** (`uploads.rs`) - Temporary incomplete state, uses `sync_all()` after writes
3. **Test code** - Intentional crash simulation

## Conclusion

All critical filesystem writes in hoop-daemon follow the atomic write pattern (write to .tmp → fsync → rename). The centralized `atomic_write` module is used consistently, and comprehensive crash-injection tests verify crash safety at all critical points.

Plan reference: §3 principle 6, notes/architecture-patterns §F
