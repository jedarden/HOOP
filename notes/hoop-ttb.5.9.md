# hoop-ttb.5.9: Already-Started Detection - Semantic Pre-Dedup at Draft Time

## Summary

Verified that the semantic pre-deduplication feature is fully implemented in the HOOP daemon.

## Acceptance Criteria Verification

### 1. Local embedding model (gte-small or similar CPU-bound)
- **Status**: ✅ COMPLETE
- **Implementation**: `embedding.rs` uses `TransformerEmbedder` with BGE-small-en-v1.5
- **Model**: BGE-small-en-v1.5 (384 dimensions, ~130MB, cached locally)
- **Alternative**: NgramEmbedder fallback if model loading fails
- **No external API**: All embedding computation is CPU-bound locally

### 2. In-memory vector index rebuilt on bead/Stitch events
- **Status**: ✅ COMPLETE
- **Implementation**: `vector_index.rs` provides `VectorIndex` with:
  - In-memory storage of embeddings
  - `rebuild()` method for full rebuilds
  - `add()`/`remove()` for incremental updates
- **Event-driven rebuild**: `lib.rs` (lines 2900-2916) subscribes to bead/Stitch events
- **Persistence**: fleet.db `vector_index` table (migration 1.23.0 → 1.24.0)

### 3. Threshold configurable (default 0.82 cosine sim)
- **Status**: ✅ COMPLETE
- **Implementation**: `DedupConfig` in `vector_index.rs`
- **Default**: 0.82
- **Configuration**: `HOOP_DEDUP_THRESHOLD` environment variable

### 4. False positive rate <5% tracked over first 30d
- **Status**: ✅ COMPLETE
- **Implementation**: `DedupStats` tracks:
  - `total_checks`: Total dedup checks performed
  - `duplicates_found`: Duplicates detected
  - `false_positives_reported`: User-reported false positives
  - `false_positive_reports`: Timestamped reports for 30-day rolling
- **Metrics**:
  - `false_positive_rate()`: Cumulative all-time rate
  - `false_positive_rate_30d()`: Rolling 30-day rate
- **API**: `POST /api/dedup/false-positive` for reporting

### 5. Test: synthetic cross-project duplicate caught with >95% recall
- **Status**: ✅ COMPLETE
- **Test**: `test_synthetic_cross_project_recall` in `vector_index.rs`
- **Test data**: 20 pairs of semantically similar titles across projects
- **Result**: Asserts recall >= 95% at threshold 0.82

## Integration Points

### Draft Queue API
- **Endpoint**: `POST /api/drafts`
- **Dedup check**: Lines 302-312 in `api_draft_queue.rs`
- **Message format**: "this looks like `<project>/<id>`, which is in progress. Continue that, add this as a child, or proceed as new?"
- **Bypass**: `force_create` flag allows overriding

### Stitch Submit API
- **Endpoint**: `POST /api/stitches` (via `api_stitch_decompose.rs`)
- **Dedup check**: Lines 299-313 and 559-570
- **Same behavior**: Returns 409 CONFLICT with similar message

## Database Schema

### vector_index table
```sql
CREATE TABLE vector_index (
    id TEXT PRIMARY KEY NOT NULL,
    project TEXT NOT NULL,
    title TEXT NOT NULL,
    kind TEXT NOT NULL,
    description TEXT,
    embedding BLOB NOT NULL,
    tokens TEXT NOT NULL DEFAULT '[]',
    model_name TEXT NOT NULL,
    model_version TEXT NOT NULL,
    indexed_at TEXT NOT NULL
)
```

### vector_index_metadata table
```sql
CREATE TABLE vector_index_metadata (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
)
-- Tracks: current_model_name, current_model_version, last_rebuild_at
```

## Model Choice: BGE-small-en-v1.5 vs gte-small

The implementation uses BGE-small-en-v1.5 instead of gte-small. This is acceptable because:
- Both are compact sentence embedding models
- Both are CPU-bound and run locally
- BGE-small-en-v1.5 produces 384-dim embeddings (efficient for in-memory search)
- No external API dependency
- Model is cached locally after first download (~130MB)

## Performance

### Test: `test_performance_10k_items_p95_under_50ms`
- **Result**: P95 latency < 50ms for 10k item index
- **Approach**: Brute-force cosine similarity (sufficient for expected corpus size)

## Notes

- The vector index is loaded from fleet.db on startup if the model hasn't changed
- Full rebuilds occur when the embedding model version changes
- Incremental updates are available via `add_to_db()` and `remove_from_db()`
- False positive reporting enables threshold tuning based on real usage data
