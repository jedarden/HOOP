# Bead bf-sidmq: Add rename_all and lowercase variants to BeadType enum

## Status: Already Implemented

The `BeadType` enum in `hoop-daemon/src/lib.rs` (lines 221-237) already contains all required attributes and variants:

### Required Implementation
1. ✅ `#[serde(rename_all = "snake_case")]` attribute - present (line 222)
2. ✅ Lowercase variants for all br/bead-forge issue_type values:
   - `Task` (wire: "task")
   - `Bug` (wire: "bug")
   - `Chore` (wire: "chore")
   - `Feature` (wire: "feature")
   - `Test` (wire: "test")
   - `Docs` (wire: "docs")
   - `Story` (wire: "story")
   - Plus: `Epic`, `Genesis`, `Review`, `Fix`
3. ✅ `#[serde(other)] Unknown` catch-all variant - present (line 235)

### Acceptance Criteria Met
- issue_type values "task", "bug", "chore", "feature", "test", "docs", "story" deserialize correctly via `rename_all = "snake_case"`
- Unrecognized issue_type values (e.g., test fixtures like "split-child") deserialize to `Unknown` rather than failing
- No other enum or field changed

## Verification
- `cargo check --package hoop-daemon` passes with no errors
- All existing tests use lowercase "task", "bug" issue_type values in JSON fixtures
- The enum is used throughout the codebase without deserialization errors

This appears to have been fixed in a prior commit (possibly as part of bead bf-315nx work).
