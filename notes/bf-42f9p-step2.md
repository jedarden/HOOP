# Decision: EmbedderExt Trait Disposition

## Decision: REMOVE

## Rationale

Based on the search results from bf-4zwq8 (documented in bf-42f9p-step1.md), the decision is clear:

### 1. Already Removed
The `EmbedderExt` trait was **already removed** from the codebase in commit `a452c70` ("fix(bf-2eulg): Remove unused EmbedderExt trait"). This action resolved bead `bf-2eulg`.

### 2. No Active References
- Comprehensive search (`rg "EmbedderExt" --type rust`, `grep -r "EmbedderExt" --include="*.rs"`) returned **zero results** in Rust source code
- The trait exists only in historical documentation (notes files) and clippy inventory
- No implementations, no consumers, no trait bounds

### 3. Replacement Exists
The codebase uses the `Embedder` trait instead:
- **Location:** `hoop-daemon/src/embedding.rs`
- **Active usage:** Used by `NgramEmbedder` implementation and referenced in multiple example/test files
- **Functional:** Provides the necessary abstraction for text embedding

### 4. Not a Public API
HOOP is not a library crate; it's a daemon binary + CLI. There are no external consumers depending on trait stability.

### 5. No Future Use Indicated
No documentation, comments, or beads indicate plans to reintroduce `EmbedderExt`.

## Conclusion

**REMOVE** (Action already taken)

The `EmbedderExt` trait was correctly removed in commit `a452c70`. The removal resolved the unused trait warning and simplified the codebase. The `Embedder` trait provides the necessary functionality without the extension complexity.

No further action required — this is a confirmation of an existing correct state.

## Date
2026-07-03
