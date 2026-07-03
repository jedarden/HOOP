# bf-31fcc: EmbedderExt trait fix implementation

## Task
Implement the fix decided in bf-27w6p for the EmbedderExt trait.

## Decision from bf-27w6p
**Decision: REMOVE**

The trait was already removed in commit a452c70 (2026-06-27).

## Verification performed

1. **Searched for remaining references:**
   - No occurrences of "EmbedderExt" found in:
     - hoop-daemon/
     - hoop-cli/
     - hoop-mcp/
     - hoop-schema/

2. **Verified file state:**
   - `hoop-daemon/src/embedding_service.rs` now only uses the `Embedder` trait:
     - Line 13: `use crate::embedding::{Embedder, NgramEmbedder};`
   - No orphaned impl blocks remain
   - No trait definition exists

3. **Compilation check:**
   - `cargo check --all-targets` completed successfully (exit code 0)
   - No errors related to EmbedderExt

## Outcome
✅ EmbedderExt trait removal is complete and verified
✅ No orphaned code remains
✅ Code compiles without errors

## Reference commits
- a452c70 - "fix(bf-2eulg): Remove unused EmbedderExt trait"
- f788a41 - "docs(bf-27w6p): Document EmbedderExt trait disposition decision"
