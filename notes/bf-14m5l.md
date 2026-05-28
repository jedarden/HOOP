# bf-14m5l: hoop restore CLI verification

## Task Claim
Plan §15.4 defines 'hoop restore --from s3://...' as a Phase 6 operational polish deliverable. Phase 6 is claimed complete but this command does not exist.

## Verification Result: FALSE

The `hoop restore` command **exists and is fully implemented**.

## Evidence

### 1. Command exists and is functional
```bash
$ ./target/release/hoop restore --help
Restore from a prior snapshot (requires daemon stopped)

Usage: hoop restore [OPTIONS] --from <FROM>

Options:
      --from <FROM>  S3 URI: s3://<bucket>/<prefix>/<snapshot-id>
      --dry-run      Validate and show what would be restored without making changes
  -h, --help         Print help
```

### 2. Implementation location
`hoop-cli/src/restore.rs` (933 lines, 13 tests)

### 3. All required features implemented

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| S3 URI parsing (`s3://bucket/key`) | ✅ | `parse_s3_uri()` (lines 23-34) |
| S3 download with AWS SigV4 | ✅ | `s3_get()` (lines 137-168) |
| Dry-run flag | ✅ | `--dry-run` CLI arg + dry_run logic (lines 310-341) |
| Daemon running check | ✅ | `is_daemon_running()` (lines 184-202) |
| Rollback mechanism | ✅ | `move_aside_for_rollback()`, `rollback_to()` (lines 217-274) |
| SHA-256 integrity check | ✅ | `verify_sha256()` (lines 172-180) |
| Manifest validation | ✅ | Uses `manifest.validate()` (line 306) |
| Schema migrations | ✅ | `restore_and_migrate()` call (line 432) |
| Audit hash chain verification | ✅ | `verify_hash_chain()` call (lines 440-454) |
| Config file restoration | ✅ | `restore_config_files()` (lines 525-562) |
| Attachments restoration | ✅ | Lines 397-420 |

### 4. Documentation

The command is documented in:
- `docs/plan/plan.md` §15.4
- `docs/cli.md` - Full CLI reference
- `docs/operations.md` - Disaster recovery procedures
- `docs/troubleshooting.md` - Restore failure troubleshooting

### 5. Tests

13 unit tests in `restore.rs` covering:
- S3 URI parsing
- SHA-256 verification
- Manifest parsing
- Rollback behavior
- Newer-version rejection
- Cleanup on success
- Mid-flight failure rollback

Integration tests:
- `hoop-daemon/tests/backup_restore_cycle.rs`
- `hoop-daemon/tests/disaster_recovery_runbook.rs`

## Conclusion

**The bead's premise is incorrect.** The `hoop restore` command is fully implemented, tested, and documented. It was completed as part of Phase 6 (TTB-12: Backup & DR) as documented in `notes/hoop-ttb.12-final-verification-2026-05-09.md`.

This bead should be closed as "already complete" with no action required.
