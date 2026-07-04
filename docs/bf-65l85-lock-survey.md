# Lock Survey: embedding_service.rs

**Bead:** bf-65l85
**Date:** 2026-07-04
**Purpose:** Complete catalog of all lock types in embedding_service.rs for child bead analysis

---

## Summary

**Total lock types found: 3**
- 2 × `RwLock` (std::sync::RwLock)
- 1 × `Semaphore` (tokio::sync::Semaphore)

**All locks are wrapped in Arc** for thread-safe sharing across async tasks.

---

## Lock 1: Cache RwLock

### Declaration
- **Location:** Line 99 (struct field declaration)
- **Type:** `Arc<RwLock<HashMap<String, CacheEntry>>>`
- **Field name:** `cache`

### What it protects
- **Data structure:** `HashMap<String, CacheEntry>`
- **Purpose:** In-memory cache of embedding vectors, keyed by SHA-256 content hash
- **Entry structure:** `CacheEntry { embedding: EmbeddingVec, created_at: Instant }`

### Initialization
- **Location:** Line 124 (in `new()` constructor)
- **Code:** `cache: Arc::new(RwLock::new(HashMap::new()))`
- **Initial state:** Empty HashMap

### Usage sites (read operations)
1. **Line 212** - `clear_cache()` method:
   ```rust
   let mut cache = self.cache.write().unwrap();
   ```
   - Purpose: Acquire write lock to clear all entries

2. **Line 220** - `cache_stats()` method:
   ```rust
   let cache = self.cache.read().unwrap();
   ```
   - Purpose: Acquire read lock to calculate statistics (total, expired, valid entries)

3. **Line 316** - `embed_cached()` method:
   ```rust
   let cache = self.cache.read().unwrap();
   ```
   - Purpose: Acquire read lock to check cache hit before computing embedding

### Usage sites (write operations)
1. **Line 212** - `clear_cache()`:
   - Action: `cache.clear()`

2. **Line 345** - `embed_cached()`:
   ```rust
   let mut cache = self.cache.write().unwrap();
   cache.insert(hash, CacheEntry { ... });
   ```
   - Purpose: Insert newly computed embedding into cache

### Test usage sites
1. **Line 540** - `test_cache_stats()`:
   ```rust
   let mut cache = service.cache.write().unwrap();
   ```
   - Purpose: Insert test entry to verify statistics

2. **Line 567** - `test_clear_cache()`:
   ```rust
   let mut cache = service.cache.write().unwrap();
   ```
   - Purpose: Insert test entry before verifying clear operation

---

## Lock 2: Request Timestamps RwLock

### Declaration
- **Location:** Line 103 (struct field declaration)
- **Type:** `Arc<RwLock<Vec<Instant>>>`
- **Field name:** `request_timestamps`

### What it protects
- **Data structure:** `Vec<Instant>`
- **Purpose:** Rolling window of request timestamps for rate limiting
- **Window duration:** 60 seconds (1 minute)
- **Used to:** Track request rate and compute wait time for rate limiting

### Initialization
- **Location:** Line 126 (in `new()` constructor)
- **Code:** `request_timestamps: Arc::new(RwLock::new(Vec::new()))`
- **Initial state:** Empty Vec

### Usage sites (write operations)
1. **Line 403** - `acquire_rate_limit()` method:
   ```rust
   let mut timestamps = self.request_timestamps.write().unwrap();
   ```
   - Purpose: Clean old timestamps (>60s) and check if rate limit exceeded
   - Critical section: Lines 403-422
   - Operations:
     - `timestamps.retain(...)` - remove old timestamps
     - `timestamps.len()` - count recent requests
     - `timestamps.first()` - get oldest timestamp for wait time calculation
   - **Lock is released at line 423** (end of scope)

2. **Line 438** - `acquire_rate_limit()` method:
   ```rust
   let mut timestamps = self.request_timestamps.write().unwrap();
   ```
   - Purpose: Record new request timestamp after rate limit check
   - Operation: `timestamps.push(now)`
   - Critical section: Lines 438-441
   - **Lock is released at line 441** (end of scope)

### Notable pattern
- **Two-phase locking in `acquire_rate_limit()`:**
  1. First write lock (403-422): Check rate limit, compute wait time, clean old entries
  2. Sleep happens **without holding lock** (426-430)
  3. Semaphore permit acquired (433)
  4. Second write lock (438-441): Record timestamp
- This pattern prevents holding locks during async sleep

---

## Lock 3: Rate Limiter Semaphore

### Declaration
- **Location:** Line 101 (struct field declaration)
- **Type:** `Option<Arc<Semaphore>>`
- **Field name:** `rate_limiter`

### What it protects
- **Resource:** Concurrent API requests to remote embedding service
- **Purpose:** Limit concurrent requests to prevent overwhelming the API
- **Permit calculation:** `((rpm / 60.0) * 2.0).ceil()` (bursts up to 2x sustained rate)

### Initialization
- **Location:** Lines 114-119 (in `new()` constructor)
- **Code:**
  ```rust
  let rate_limiter = config.rate_limit_rpm.map(|rpm| {
      let permits = ((rpm as f64) / 60.0 * 2.0).ceil() as usize;
      Arc::new(Semaphore::new(permits.max(1)))
  });
  ```
- **Initial state:** `None` if no rate limit configured, or `Semaphore(permits)` if configured

### Usage sites (acquire operations)
1. **Line 433** - `acquire_rate_limit()` method:
   ```rust
   let _permit = semaphore.acquire().await.map_err(...)?;
   ```
   - Purpose: Acquire permit before making API request
   - Blocking: `.await` waits if no permits available
   - Permit held until end of scope (line 441)

### Hot-reload update
- **Location:** Lines 247-252 (in `update_config()` method)
- **Code:**
  ```rust
  if new_config.rate_limit_rpm != self.config.rate_limit_rpm {
      self.rate_limiter = new_config.rate_limit_rpm.map(|rpm| {
          let permits = ((rpm as f64) / 60.0 * 2.0).ceil() as usize;
          Arc::new(Semaphore::new(permits.max(1)))
      });
  }
  ```
- **Action:** Replaces entire Semaphore with new one if RPM changes
- **Note:** Old permits remain valid until dropped; new config affects new requests only

---

## Import Statements

### Line 19
```rust
use std::sync::{Arc, RwLock};
```
- `Arc`: Atomic Reference Counting for thread-safe sharing
- `RwLock`: Read-write lock for interior mutability across threads

### Line 21
```rust
use tokio::sync::Semaphore;
```
- `Semaphore`: Async semaphore for limiting concurrent operations

---

## Lock Ordering and Potential Deadlocks

### Observed locking order
1. **Cache RwLock** - acquired independently
2. **Request Timestamps RwLock** - acquired independently
3. **Rate Limiter Semaphore** - acquired after Request Timestamps lock is released

### Lock acquisition patterns
- **Cache RwLock:** Single lock at a time (read OR write, never both)
- **Request Timestamps RwLock:** Single lock at a time, released before async sleep
- **Semaphore:** Acquired after Request Timestamps lock released (line 433 after 423)

### Potential deadlock scenarios
**No lock ordering issues detected** - locks are acquired independently without cross-dependencies.

### Await-holding-lock analysis
- **Lines 403-423:** Request Timestamps write lock held, but **NO await** within this section
- **Line 426-430:** Async sleep happens **without holding lock** (correct pattern)
- **Line 433:** Semaphore `.await` happens **without holding RwLock** (correct pattern)
- **Lines 438-441:** Second write lock acquisition, **NO await** within section

---

## Key Statistics

| Lock | Type | Shared? | Read ops | Write ops | Test usage |
|------|------|---------|----------|-----------|------------|
| `cache` | RwLock<T> | Arc | 3 sites | 3 sites | 2 sites |
| `request_timestamps` | RwLock<T> | Arc | 0 | 2 sites | 0 |
| `rate_limiter` | Semaphore | Arc (via Option) | 0 | 1 site | 0 |

**Total lock operations:** 11 sites across 3 locks

---

## Child Bead 2 Analysis Checklist

For child bead analyzing await-holding-lock issues:

1. ✓ **Cache RwLock:** All operations are synchronous (`.unwrap()`, no `.await`)
2. ✓ **Request Timestamps RwLock:** Two-phase locking correctly avoids holding during sleep
3. ✓ **Semaphore:** Acquired via `.await` but only after RwLock released

**No critical await-holding-lock bugs detected** in this module, but child bead should verify:
- That `cache.read().unwrap()` and `cache.write().unwrap()` are never held across `.await`
- That the two-phase pattern in `acquire_rate_limit()` is intentional and correct
- That test code (lines 540, 567) does not reflect real async usage patterns

---

## Related Files

- **Embedding trait:** `hoop-daemon/src/embedding.rs` (defines `Embedder` trait)
- **NgramEmbedder:** `hoop-daemon/src/embedding.rs` (local fallback implementation)
- **Config:** `hoop-daemon/src/config_resolver.rs` (provides embedding configuration)
- **Metrics:** `hoop-daemon/src/metrics.rs` (exposes cache hit rate gauges)

---

## Bead Context

This survey provides the foundation for child bead 2 (bf-2ioxn or similar) to perform detailed analysis of each lock for potential await-holding-lock bugs. Each lock type and usage site should be verified for correct async/await hygiene.
