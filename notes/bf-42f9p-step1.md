# Search Results: EmbedderExt Trait Usage

## Task
Search the entire codebase for all references to `EmbedderExt` to understand its usage scope.

## Search Methods Used
1. `rg "EmbedderExt" --type rust` - ripgrep search for exact pattern
2. `grep -r "EmbedderExt" --include="*.rs"` - grep recursive search
3. `grep -ri "embedder" --include="*.rs"` - case-insensitive search for related terms

## Results

**No references to `EmbedderExt` found in the codebase.**

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
