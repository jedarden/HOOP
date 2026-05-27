# Hash Chain Implementation Status (bf-5mp41)

## Task Description
Plan §13 (Security model) requires an append-only hash chain on the `fleet.db` actions table: each row includes the hash of the previous row concatenated with its own content, providing tamper evidence.

## Implementation Status: COMPLETE ✓

All required components are already implemented and functional:

### 1. Schema (hoop-daemon/src/fleet.rs:825-840)
The `actions` table includes the required hash chain columns:
```sql
CREATE TABLE IF NOT EXISTS actions (
    id TEXT PRIMARY KEY NOT NULL,
    ts TEXT NOT NULL,
    actor TEXT NOT NULL,
    kind TEXT NOT NULL,
    target TEXT NOT NULL,
    project TEXT,
    args_json TEXT,
    result TEXT,
    error TEXT,
    source TEXT,
    stitch_id TEXT,
    args_hash TEXT,
    hash_prev TEXT NOT NULL,  -- Hash of previous row
    hash_self TEXT NOT NULL   -- Hash of this row
)
```

### 2. Hash Chain Computation (hoop-daemon/src/fleet.rs:165-230)
`write_audit_row()` maintains the hash chain:
- Fetches the most recent row's `hash_self` as `hash_prev`
- Computes `hash_self` from row content (id, ts, actor, kind, target, project, args_json)
- Uses SHA-256 with hex encoding
- First row uses `GENESIS_HASH` (64 zeros)

### 3. Verification Function (hoop-daemon/src/fleet.rs:524-589)
`verify_hash_chain()` walks the chain and reports broken links:
- Verifies `hash_prev` matches expected chain
- Recomputes each row's `hash_self` to detect tampering
- Returns detailed error on first broken link

### 4. CLI Command (hoop-cli/src/main.rs:482-513)
`hoop audit verify [--json]` command:
- Human-readable output: "Audit log hash chain is intact" + final hash
- JSON output for automation: `{status, message, final_hash}`
- Exit code 1 on verification failure

### 5. API Endpoint (hoop-daemon/src/api_audit.rs:176-322)
`GET /api/audit/verify` endpoint:
- Returns `{valid, message, row_count}`
- Registered in main router (hoop-daemon/src/lib.rs:1248)

### 6. Tests (hoop-daemon/src/fleet.rs:6283-6348)
- `test_sha256_hex_encode()` - hash function correctness
- `test_genesis_hash_constant()` - GENESIS_HASH validation
- `test_insert_genesis_row()` - first row hash chain integrity

## Verification

```bash
# CLI verification
$ hoop audit verify
Audit log hash chain is intact
Final hash: 1b4cfe3cbf8ae8c11f9b78361be39b4c0fa7705d172e62c85f4484e4d51dee7d

# JSON output
$ hoop audit verify --json
{"final_hash":"1b4cfe3cbf8ae8c11f9b78361be39b4c0fa7705d172e62c85f4484e4d51dee7d","message":"Audit log hash chain is intact","status":"ok"}
```

## Conclusion

The hash chain infrastructure required by plan §13 is **fully implemented and operational**. No additional work is required for this task.
