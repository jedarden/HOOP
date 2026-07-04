# await_holding_lock Analysis: embedding_service.rs

## Summary

**Status**: ✅ All await_holding_lock issues have been fixed

**Date**: 2026-07-04

**Analysis Result**: The file `embedding_service.rs` previously contained one await_holding_lock issue that was fixed in commit `1731365` on 2026-07-04. No remaining await_holding_lock issues exist in the current codebase.

---

## Issue Found and Fixed

### Issue 1: await_holding_lock in `acquire_rate_limit` (FIXED)

**Location**: `hoop-daemon/src/embedding_service.rs:398-444` (current code)

**Status**: ✅ FIXED in commit `1731365`

**Severity**: HIGH - Potential deadlock

#### Description (BEFORE the fix)

The `acquire_rate_limit` function was holding a `write()` lock on `self.request_timestamps` across the `tokio::time::sleep(wait_time).await` call. This is a classic await_holding_lock bug.

**Problematic code pattern (BEFORE fix)**:
```rust
async fn acquire_rate_limit(&self) -> Result<()> {
    if let Some(ref semaphore) = self.rate_limiter {
        // Acquire semaphore permit (may yield here)
        let _permit = semaphore.acquire().await?;

        // Acquire write lock
        let mut timestamps = self.request_timestamps.write().unwrap();
        let now = Instant::now();
        let window = Duration::from_secs(60);

        // Remove timestamps older than 1 minute
        timestamps.retain(|ts| now.duration_since(*ts) < window);

        // Check if we're within rate limit
        if let Some(rpm) = self.config.rate_limit_rpm {
            let requests_per_minute = timestamps.len() as u32;
            if requests_per_minute >= rpm {
                let oldest = timestamps.first().copied().unwrap_or(now);
                let wait_time = window.saturating_sub(now.duration_since(oldest));
                
                // ❌ BUG: Holding write lock across .await!
                if wait_time > Duration::ZERO {
                    tokio::time::sleep(wait_time).await;
                }
            }
        }

        timestamps.push(now);
    }
    Ok(())
}
```

**Why this is dangerous**:
- `tokio::time::sleep().await` yields control to the runtime
- While sleeping, the task continues to hold the write lock on `self.request_timestamps`
- Other tasks attempting to acquire this lock will block indefinitely
- Can cause deadlocks or severe performance degradation

#### Fix Applied (commit 1731365)

**Commit message**: "fix(embedding-service): Fix await_holding_lock deadlock in acquire_rate_limit"

**Fixed code pattern (AFTER fix)**:
```rust
async fn acquire_rate_limit(&self) -> Result<()> {
    if let Some(ref semaphore) = self.rate_limiter {
        // First pass: compute wait time WITHOUT holding the semaphore permit
        let wait_time = {
            // Lock only to check rate limit status
            let mut timestamps = self.request_timestamps.write().unwrap();
            let now = Instant::now();
            let window = Duration::from_secs(60);

            // Remove timestamps older than 1 minute
            timestamps.retain(|ts| now.duration_since(*ts) < window);

            // Check if we're within rate limit
            if let Some(rpm) = self.config.rate_limit_rpm {
                let requests_per_minute = timestamps.len() as u32;
                if requests_per_minute >= rpm {
                    let oldest = timestamps.first().copied().unwrap_or(now);
                    Some(window.saturating_sub(now.duration_since(oldest)))
                } else {
                    None
                }
            } else {
                None
            }
            // ✅ Lock is released here (block scope ends)
        };

        // ✅ Sleep WITHOUT holding any locks
        if let Some(duration) = wait_time {
            if duration > Duration::ZERO {
                tokio::time::sleep(duration).await;
            }
        }

        // ✅ Acquire semaphore permit (await without holding the write lock)
        let _permit = semaphore.acquire().await.map_err(|e| {
            anyhow::anyhow!("Failed to acquire rate limit permit: {}", e)
        })?;

        // Now acquire the write lock and record the timestamp
        let mut timestamps = self.request_timestamps.write().unwrap();
        let now = Instant::now();
        timestamps.push(now);
        // Both permit and lock are dropped here
    }
    Ok(())
}
```

**Fix strategy**:
1. Use a block scope `{}` to ensure the write lock is released immediately after computing `wait_time`
2. Perform the `tokio::time::sleep()` after the lock is released
3. Acquire the semaphore permit separately
4. Re-acquire the write lock only to record the timestamp

---

## Other Lock Usage Analysis

### embed_cached (lines 311-357)

**Status**: ✅ No await_holding_lock issues

The `embed_cached` function correctly uses scoped locks:

```rust
async fn embed_cached(&self, text: &str) -> Result<EmbeddingVec> {
    let hash = self.compute_hash(text);

    // Check cache - lock scoped to this block only
    {
        let cache = self.cache.read().unwrap();
        if let Some(entry) = cache.get(&hash) {
            let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
            let age = Instant::now().duration_since(entry.created_at);

            if age < ttl {
                metrics().hoop_embedding_cache_hits_total.inc();
                self.update_cache_hit_rate();
                return Ok(entry.embedding);
            }
        }
    } // ✅ Lock released here

    metrics().hoop_embedding_cache_misses_total.inc();

    // Generate embedding (this may await, but without holding lock)
    let embedding = if adapter_kind == "remote" {
        self.embed_remote(text).await?
    } else {
        self.embed_local(text)?
    };

    // Store in cache - lock scoped to this block only
    {
        let mut cache = self.cache.write().unwrap();
        cache.insert(
            hash,
            CacheEntry {
                embedding,
                created_at: Instant::now(),
            },
        );
    } // ✅ Lock released here

    self.update_cache_hit_rate();
    Ok(embedding)
}
```

**No issues**: Both read and write locks are released in scoped blocks before any `.await` calls.

---

## Lock Types Used

- **`Arc<RwLock<HashMap<String, CacheEntry>>>`** (line 99): In-memory cache
- **`Arc<RwLock<Vec<Instant>>>`** (line 103): Request timestamps for rate limiting
- **`Arc<Semaphore>`** (line 101): Rate limiter for remote API calls

All `RwLock` usage in async contexts has been verified to be safe (no locks held across `.await` points).

---

## Verification

To verify no await_holding_lock issues remain:

```bash
# Run clippy with await_holding_lock lint
cargo clippy -- -W await_holding_lock 2>&1 | grep embedding_service
# (No output = no issues)
```

**Result**: No await_holding_lock warnings found for `embedding_service.rs` in the current codebase.

---

## Conclusion

**Summary**: One await_holding_lock issue was found and fixed in commit `1731365`. The current code is free of await_holding_lock bugs.

**Recommendation**: No further action required for this module. The fix is correct and complete.
