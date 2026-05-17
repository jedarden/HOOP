# CI Gate Verification: hoop-schema tests

**Date:** 2026-05-17
**Bead:** bf-205tk
**Task:** CI gate: cargo test -p hoop-schema

## Result

All tests passed successfully:
- 148 unit tests passed (id validators, path security, effort levels, schema round-trips)
- 1 schema drift test passed (1 ignored)
- 0 failures

## Test Coverage Summary

The hoop-schema crate test suite validates:
- ID format validators (bead_id, draft_id, job_id, pattern_id, project_name, stitch_id, upload_id, worker_name)
- Path security checks (traversal attacks, symlink escapes, null bytes, URL encoding)
- Effort level validation for Claude/Codex adapters
- Schema round-trip serialization/deserialization for all durable records
- Schema version format validation

No code changes were required — this was a verification-only gate.
