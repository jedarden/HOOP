# bf-4k8oj: Concurrent Backup Run Guard

## Finding

The concurrent run guard has already been fully implemented. The TODO comment mentioned in the bead description has been resolved.

## Existing Implementation (hoop-daemon/src/backup_pipeline.rs)

1. **Guard type**: `Arc<tokio::sync::Mutex<()>>` field named `running`
2. **AlreadyRunning error**: Custom error type with Display/Error impl
3. **is_running() method**: Checks lock state via `try_lock().is_err()`
4. **trigger() method**: Uses `try_lock()` to fail-fast with `AlreadyRunning` error
5. **Scheduler integration**: Checks with `try_lock()` before running scheduled backup

## API Implementation (hoop-daemon/src/api_backup.rs)

1. **POST /api/backup/trigger**: Returns 409 Conflict when `AlreadyRunning` is detected
2. **GET /api/backup/status**: Returns current state ("idle" or "running")

## Requirements Satisfied

- ✓ Returns 409 Conflict if backup is in progress
- ✓ Releases guard when backup completes (RAII drop)
- ✓ Exposes status endpoint

## Files

- `hoop-daemon/src/api_backup.rs` - API endpoints
- `hoop-daemon/src/backup_pipeline.rs` - Core implementation
