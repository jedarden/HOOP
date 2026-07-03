# Search Results: EmbedderExt Trait Usage

## Task
Search the entire codebase for all references to `EmbedderExt` to understand its usage scope.

## Search Methods Used
1. `rg "EmbedderExt" --type rust` - ripgrep search for exact pattern
2. `grep -r "EmbedderExt" --include="*.rs"` - grep recursive search
3. `grep -ri "embedder" --include="*.rs"` - case-insensitive search for related terms

## Results

**No references to `EmbedderExt` found in the actual Rust source code.**

### Historical References (documentation only)

The following files reference `EmbedderExt` but are documentation, not active code:

1. **`clippy-inventory.txt:5`** - Clippy warning: `hoop-daemon/src/embedding_service.rs:458 - trait 'EmbedderExt' is never used`
   - This is a clippy inventory noting the trait was flagged as unused before it was removed

2. **`notes/bf-37psy.md`** - Documents commit `de4f668` "Remove unused EmbedderExt trait"
   - References the removal at line 458 of embedding_service.rs

3. **`notes/bf-42f9p.md`** - Current bead documenting that EmbedderExt was already removed
   - Cites commit `a452c70`: "fix(bf-2eulg): Remove unused EmbedderExt trait"

4. **`notes/bf-1e9la.md`** - Verification that EmbedderExt trait was removed
   - Confirms grep shows no EmbedderExt in embedding_service.rs

### Current Codebase Status

### What Exists Instead

The codebase contains a related trait `Embedder` (not `EmbedderExt`):

**Definition location:** `hoop-daemon/src/embedding.rs`

```rust
/// Trait for embedding text into fixed-dimension vectors.
pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Embedding;
    fn canonical_tokens(&self, text: &str) -> Vec<String>;
    fn model_info(&self) -> (String, String);
    fn as_any(&self) -> &dyn std::any::Any;
}
```

**Implementations:**
- `NgramEmbedder` - implements `Embedder` trait (fallback implementation)

**Files using the Embedder trait:**
- `hoop-daemon/src/embedding.rs` - trait definition + NgramEmbedder implementation
- `hoop-daemon/examples/test_oauth.rs` - uses Embedder, NgramEmbedder
- `hoop-daemon/examples/test_tokens.rs` - uses Embedder, NgramEmbedder
- `hoop-daemon/examples/test_sim.rs` - uses Embedder, NgramEmbedder
- `hoop-daemon/tests/pure_functions.rs` - uses Embedder, NgramEmbedder in tests

## Conclusion

The `EmbedderExt` trait does not exist in the HOOP codebase. Based on recent git commits (2d38227, 2bba3c9) documenting "EmbedderExt already removed," this trait was previously removed from the codebase. The current implementation uses the `Embedder` trait instead.

## Date
2026-07-03
