# Global br Subprocess Semaphore Implementation (§4.8)

## Status: ✓ ALREADY COMPLETE

This task was already fully implemented in the codebase. All requirements from plan §4.8 are met:

## Implementation Details

### 1. Semaphore Infrastructure (lib.rs)

**DaemonState fields (lines 382-384):**
```rust
pub br_semaphore: Arc<tokio::sync::Semaphore>,
pub br_semaphore_target_permits: Arc<tokio::sync::RwLock<usize>>,
```

**Initialization at startup (lines 2877-2883):**
```rust
let br_semaphore_permits = project_count.min(10);
let br_semaphore = Arc::new(tokio::sync::Semaphore::new(br_semaphore_permits));
let br_semaphore_target_permits = Arc::new(tokio::sync::RwLock::new(br_semaphore_permits));
info!("br subprocess semaphore initialized with {} permits ({} projects)", ...);
```

**Hot-reload capacity update (lines 2938-2974):**
```rust
// Subscribe to projects config changes
tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        match event {
            ProjectsEvent::ConfigReloaded { config, .. } => {
                update_br_semaphore_capacity(&br_semaphore_for_update, ...).await;
            }
        }
    }
});
```

**Capacity update function (lines 405-467):**
- Calculates new target as `min(project_count, 10)`
- Adds permits if new target > old
- Tries to acquire/drop excess permits if new target < old
- Updates target permits RwLock

### 2. Semaphore-Aware Execution (br_verbs.rs)

**spawn_br_command (lines 344-396):**
```rust
pub async fn spawn_br_command(
    semaphore: &tokio::sync::Semaphore,
    mut cmd: std::process::Command,
    verb: &str,
) -> anyhow::Result<std::process::Output>
```
- Acquires permit from semaphore (blocks if limit reached)
- Increments `hoop_br_subprocess_concurrent` metric
- Spawns br command in blocking task
- Decrements metric and releases permit on completion
- Records total and duration metrics

**spawn_br_command_blocking (lines 413-454):**
- Synchronous version with same semantics
- Uses `try_acquire()` instead of `acquire().await`

### 3. Metrics (metrics.rs)

**Line 735:**
```rust
pub hoop_br_subprocess_concurrent: Gauge,
```

**Used in spawn functions:**
- Line 358: `inc()` before spawn
- Line 371: `dec()` after completion
- Line 423: `inc()` in blocking version
- Line 429: `dec()` in blocking version

### 4. All Call Sites Use Semaphore

Verified that all br subprocess invocations use the semaphore:
- api_beads.rs:593, 792 → `spawn_br_command`
- orphan_beads.rs:56, 138 → `spawn_br_command` / `spawn_br_command_blocking`
- api_orphans.rs:127 → `spawn_br_command_blocking`
- pattern_query_evaluator.rs:334 → `spawn_br_command_blocking`
- api_stitch_decompose.rs:502, 611 → `spawn_br_command`
- api_bead_blockers.rs:199 → `spawn_br_command_blocking`

## Verification

✓ Semaphore added to DaemonState
✓ Initialized to min(projects, 10) permits
✓ Hot-reload updates capacity on project changes
✓ spawn_br_command acquires permit before spawning
✓ spawn_br_command_blocking acquires permit before spawning
✓ hoop_br_subprocess_concurrent gauge metric defined and used
✓ All br subprocess call sites use semaphore-controlled functions

## Conclusion

No code changes were required - the implementation was already complete and satisfies all requirements from plan §4.8.
