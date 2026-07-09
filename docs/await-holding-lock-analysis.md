# await_holding_lock Violation Analysis — embedding_service.rs

## Overview

This document analyzes the `await_holding_lock` violation that existed in `hoop-daemon/src/embedding_service.rs` around line 405. **The issue was already fixed** in commit `be2b077` on 2026-07-04 (fixing bead bf-391vz).

## The Original Violation

### Location
- **File:** `hoop-daemon/src/embedding_service.rs`
- **Function:** `acquire_rate_limit()`
- **Line:** ~417 (original line 405 in some contexts)
- **Lock Type:** `RwLock` write guard on `self.request_timestamps`
- **Variable:** `timestamps: RwLockWriteGuard<'_, Vec<Instant>>`

### Original Code (BEFORE fix)
```rust
// Lines 402-424 (original)
async fn acquire_rate_limit(&self) -> Result<()> {
    if let Some(ref semaphore) = self.rate_limiter {
        // Acquire semaphore permit FIRST
        let _permit = semaphore.acquire().await.map_err(|e| {
            anyhow::anyhow!("Failed to acquire rate limit permit: {}", e)
        })?;

        // Clean up old timestamps and check rate limit
        let mut timestamps = self.request_timestamps.write().unwrap();  // ❌ Lock acquired
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
                if wait_time > Duration::ZERO {
                    tokio::time::sleep(wait_time).await;  // ❌ AWAIT WHILE HOLDING LOCK
                }
            }
        }

        timestamps.push(now);
        // Lock released here when timestamps goes out of scope
    }
    Ok(())
}
```

### Why This Was Unsafe

**Problem:** The code held a `RwLock` write guard across an `.await` point (`tokio::time::sleep().await`).

**Deadlock Scenario:**
1. Task A acquires the write lock on `request_timestamps`
2. Task A checks rate limit and decides to sleep (e.g., 30 seconds)
3. Task A calls `tokio::time::sleep(30sec).await` **while still holding the write lock**
4. The `.await` yields the executor, allowing other tasks to run
5. Task B attempts to acquire the same lock (for rate limiting or cache reading)
6. Task B blocks indefinitely waiting for the lock
7. **30 seconds later**, Task A resumes and releases the lock
8. Task B finally acquires the lock

**Impact:**
- **Lock convoy:** Other tasks pile up waiting for the lock
- **Priority inversion:** Low-priority rate-limited tasks block high-priority tasks
- **Wasted executor capacity:** Tasks that could run are blocked on a lock that's held by a sleeping task
- **Cache starvation:** The `embed_cached()` function also needs to read from `request_timestamps` indirectly

In async Rust, holding a mutex across an await point breaks the executor's ability to fairly schedule tasks, as the lock is held for wall-clock time (seconds) rather than just CPU time (microseconds).

### Clippy Warning
The clippy lint `await_holding_lock` specifically catches this pattern:

```
warning: this `RwLockWriteGuard` is held across an `.await` point
  --> hoop-daemon/src/embedding_service.rs:417:33
   |
403 |             let mut timestamps = self.request_timestamps.write().unwrap();
   |                 ------------------------- lock is held here
...
417 |                 tokio::time::sleep(wait_time).await;
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ await point here
   |
   = help: ensure the lock is released before this await point
```

## The Fix (Commit be2b077)

### Fixed Code (AFTER fix)
```rust
// Lines 397-444 (current)
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
            // Lock is released here
        };

        // Sleep WITHOUT holding any locks
        if let Some(duration) = wait_time {
            if duration > Duration::ZERO {
                tokio::time::sleep(duration).await;  // ✅ No lock held
            }
        }

        // Acquire semaphore permit (await without holding the write lock)
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

### What Changed

1. **Lock scope reduced:** The `RwLock` write guard is now held only for the minimal critical section (computing `wait_time`)
2. **Await moved outside lock:** `tokio::time::sleep().await` now happens AFTER the lock is released
3. **Semaphore moved after sleep:** The semaphore permit acquisition also happens outside the lock (original had it before the lock)
4. **Lock re-acquired for timestamp:** A second, brief lock acquisition adds the actual timestamp

### Current Behavior

- **Lock held for:** ~microseconds (only for hashmap retain and length check)
- **Lock released before:** Both `tokio::time::sleep().await` AND `semaphore.acquire().await`
- **No deadlock risk:** Other tasks can access `request_timestamps` during the sleep period
- **Fair scheduling:** The executor can schedule other tasks during sleep without lock contention

## Verification

### Check for Remaining Violations
```bash
nix-shell --run 'cargo clippy --message-format=short 2>&1 | grep await_holding_lock'
```
Result: No output (no violations found)

### Manual Code Review
- All `.await` points in `embedding_service.rs` occur when no `RwLock` guards are active
- Locks are always dropped before any `.await` by using block scopes `{}` to control guard lifetime

## Related Beads

- **bf-391vz:** Original bead that identified and tracked the await_holding_lock issue
- **be2b077:** Git commit that implemented the fix
- **bf-56w8j:** This bead (documentation of the historical issue)

## References

- [Rust Async: Hold No Locks Across Await](https://rust-lang.github.io/async-book/07_futures/01_future.html#await-and-the-async-keyword)
- [Clippy Lint: await_holding_lock](https://rust-lang.github.io/rust-clippy/master/index.html#await_holding_lock)
- [Tokio: Why Can't I Hold A MutexGuard Across an .await?](https://tokio.rs/tokio/tutorial/shared-state#why-cant-i-hold-a-mutexguard-across-an-await)
