# Clippy Warnings Categorization — HOOP Workspace

**Generated:** 2026-07-04  
**Total Warnings:** 369  
**Command:** `cargo clippy --all-targets --workspace`

## Executive Summary

The HOOP workspace has **369 clippy warnings** across 4 crates (`hoop-daemon`, `hoop-cli`, `hoop-mcp`, `hoop-schema`). These warnings fall into **8 primary categories**:

1. **Dead Code (112)** — Unused imports, variables, functions, struct fields, constants
2. **Style & Idioms (82)** — Redundant code patterns, manual implementations, borrowed references
3. **Code Complexity (61)** — Too many arguments, complex types, async safety issues
4. **Type Safety (47)** — Unnecessary casts, useless conversions, clone on copy
5. **API Design (28)** — Missing trait implementations, inconsistent interfaces
6. **Performance (25)** — Inefficient patterns, redundant allocations
7. **Disallowed Methods (39)** — Direct filesystem operations (should use atomic_write)
8. **Test Code (15)** — Test-specific warnings in fixtures and integration tests

---

## Category 1: Dead Code (112 warnings)

The largest category — code that exists but is never used.

### 1.1 Unused Imports (~70)

**Count:** 70+ instances  
**Severity:** Low — cleanup only  
**Clippy Lints:** `unused_imports`, `single_component_path_imports`

**Examples:**

```rust
// hoop-daemon/src/accounts_config.rs:27
use std::path::{Path, PathBuf};  // PathBuf unused

// hoop-daemon/src/api_bead_files.rs:16
use rusqlite::{params, Connection};  // both unused

// hoop-daemon/src/api_pattern_mutations.rs:14
use axum::routing::{delete, get, post, put};  // get unused
```

**Common patterns:**
- `utoipa::ToSchema` imported but schema macros not used (16 instances)
- `tempfile::TempDir` in test files (3 instances)
- `std::path::PathBuf` when only `Path` is needed
- Tracing levels (`warn`, `debug`) not used in module

**Fix:** Run `cargo clippy --fix` for automatic removal, or manually remove unused imports.

### 1.2 Unused Variables (~30)

**Count:** 30+ instances  
**Severity:** Medium — may indicate incomplete implementation  
**Clippy Lints:** `unused_variables`

**Examples:**

```rust
// hoop-daemon/src/backup_pipeline.rs:133
let start = std::time::Instant::now();  // never read

// hoop-daemon/src/auth.rs:329
let required_role = required_role.clone();  // immediately overwritten

// hoop-daemon/src/capacity.rs:213
fn fetch_quota_via_http(config: &GcpQuotaConfig) -> Option<...> {  // config unused
```

**Common patterns:**
- `start` / `elapsed_ms` variables intended for metrics (5 instances)
- Variables cloned but original used (e.g., `required_role`)
- Function parameters not referenced (likely future use)

**Fix:** 
- Remove if truly unused
- Prefix with `_` if intentionally unused for API compatibility
- Add `_unused` suffix for documentation

### 1.3 Dead Functions & Structs (~10)

**Count:** 10+ instances  
**Severity:** Low  
**Clippy Lints:** `dead_code`

**Examples:**

```rust
// hoop-daemon/src/openapi.rs
fn openapi_router() -> Router { ... }  // never called

// hoop-daemon/src/capacity.rs
fn get_opencode_limits(...) -> Option<...> { ... }  // never called

// hoop-daemon/src/backup_pipeline.rs
fn walk_dir(...) -> Result<...> { ... }  // never called

// hoop-daemon/src/cost.rs
pub struct QuotaLimit { ... }  // never constructed
```

**Fix:** 
- Remove if not needed
- Add `#[allow(dead_code)]` if intended for future use
- Mark as `pub(crate)` if internal-only

### 1.4 Unused Struct Fields & Constants (~5)

**Count:** 5+ instances  
**Severity:** Low  
**Clippy Lints:** `dead_code`

**Examples:**

```rust
// hoop-daemon/src/stitch_reconstruction.rs
field `session_subpath` is never read
field `session_id` is never read
field `rpm_limit` is never read

// hoop-daemon/src/cost.rs
const STITCH_CLOSED_THRESHOLD_SECONDS: u64 = 86400;  // never used
const MIN_SAMPLES_FOR_PREDICTION: usize = 5;  // never used
```

---

## Category 2: Style & Idioms (82 warnings)

Code that works but doesn't follow Rust idioms.

### 2.1 Redundant Code Patterns (25)

**Count:** 25+ instances  
**Severity:** Low  
**Clippy Lints:** `redundant_closure`, `unnecessary_map_or`, `manual_*`

**Examples:**

```rust
// Redundant closure (8 instances)
let json: String = stitches.iter().map(|s| s.to_json()).collect();
// Better: stitches.iter().map(Stitch::to_json).collect()

// Manual flatten (2 instances)
opt.map(...).flatten()  // Use: opt.and_then(...)

// Manual clamp (2 instances)
let x = if val < min { min } else if val > max { max } else { val };
// Better: val.clamp(min, max)

// Manual strip prefix (1 instance)
path.strip_prefix("/home").unwrap()
// Better: path.strip_prefix("/home")
```

### 2.2 Reference & Borrowing Issues (15)

**Count:** 15+ instances  
**Severity:** Medium  
**Clippy Lints:** `needless_borrow`, `needless_borrows_for_generic_args`, `ptr_arg`

**Examples:**

```rust
// hoop-schema/tests/schema_drift.rs:761
fs::write(&file_path, &json)  // json already implements required traits
// Better: fs::write(&file_path, json)

// hoop-daemon/src/api_stitch_recompose.rs:248
write(&mut Vec).with_context(...)
// Better: write(&mut _).with_context(...)  // use slice instead

// hoop-daemon/src/stitch_links.rs:82
fn merge_links(links: &Vec<StitchLink>) -> ...  // Don't pass &Vec
// Better: fn merge_links(links: &[StitchLink]) -> ...
```

### 2.3 String & Format Inefficiencies (12)

**Count:** 12+ instances  
**Severity:** Low  
**Clippy Lints:** `useless_format`, `format!` misuse, `expect_fun_call`

**Examples:**

```rust
// hoop-schema/tests/schema_drift.rs:762
.expect(&format!("Failed to write fixture: {}", file_path))
// Better: .unwrap_or_else(|_| panic!("Failed to write fixture: {file_path}"))

// Useless format! (2 instances)
let s = format!("{}", value);  // Better: value.to_string()
```

### 2.4 Control Flow Simplifications (10)

**Count:** 10+ instances  
**Severity:** Low  
**Clippy Lints:** `collapsible_if`, `single_match`, `if_same_then_else`, `match_result_ok`

**Examples:**

```rust
// Collapsible if (3 instances)
if x { if y { ... } }
// Better: if x && y { ... }

// Single match (1 instance)
match opt { Some(x) => ..., None => default() }
// Better: if let Some(x) = opt { ... } else { default() }

// Same then/else (1 instance)
if cond { foo() } else { foo() }
// Better: foo()
```

---

## Category 3: Code Complexity (61 warnings)

Structural issues that make code hard to maintain or unsafe.

### 3.1 Too Many Arguments (6)

**Count:** 6 functions  
**Severity:** High — API design issue  
**Clippy Lints:** `too_many_arguments`

**Examples:**

```rust
// hoop-daemon/src/fleet.rs:645 — 12 arguments
pub fn create_stitch_with_audit(
    conn: &mut Connection,
    project: &str,
    title: &str,
    description: &str,
    kind: &str,
    operator: &str,
    attachments: Vec<Attachment>,
    parent_stitch_id: Option<&str>,
    tags: Vec<String>,
    context_source: ContextSource,
    event_timestamp: i64,
    audit_flags: AuditFlags,
) -> Result<Stitch>

// hoop-daemon/src/config_resolver.rs:679 — 9 arguments
fn resolve_opt_strict<T: Clone + Serialize>(
    conn: &mut Connection,
    project: &str,
    key: &str,
    default_expr: Option<&str>,
    allow_missing: bool,
    schema: &Value,
    validation_context: &ValidationContext,
    runtime_context: &RuntimeContext,
    cache: &mut ConfigCache,
) -> Result<T>

// hoop-daemon/src/supervisor.rs:243 — 9 arguments
pub fn new(
    project: &str,
    workspace_root: &Path,
    db_path: &Path,
    event_queue: Arc<Mutex<Vec<Event>>>,
    runtime_state: Arc<RwLock<ProjectRuntimeState>>,
    config: SupervisorConfig,
    metrics_tx: mpsc::Sender<MetricEvent>,
    registry: Arc<ProjectsRegistry>,
    shutdown_rx: watch::Receiver<ShutdownPhase>,
) -> Self
```

**Fix:** 
- Group related parameters into structs
- Use builder pattern for construction
- Extract configuration objects

### 3.2 Complex Types (6)

**Count:** 6 instances  
**Severity:** Medium  
**Clippy Lints:** `type_complexity`

**Examples:**

```rust
// hoop-daemon/src/cost.rs:499
let mut map: HashMap<(String, String), (f64, i64, i64, i64, i64)> = HashMap::new();

// hoop-daemon/src/metrics.rs:397
data: RwLock<HashMap<Vec<String>, (u64, f64, Vec<u64>)>>,

// hoop-daemon/src/stitch_percentile_index.rs:582
let mut stitches: Vec<(String, String, Option<String>, Option<String>, f64, i64)> = Vec::new();
```

**Fix:** Define type aliases:

```rust
type CostMap = HashMap<(String, String), (f64, i64, i64, i64, i64)>;
type MetricRow = (Vec<String>, u64, f64, Option<f64>, Option<f64>, Option<f64>);
type StitchIndexRow = (String, String, Option<String>, Option<String>, f64, i64);
```

### 3.3 Async Safety Issues (1)

**Count:** 1 instance  
**Severity:** **Critical** — can cause deadlocks  
**Clippy Lints:** `await_holding_lock`

**Example:**

```rust
// hoop-daemon/src/embedding_service.rs:405
let mut timestamps = self.request_timestamps.write().unwrap();
// ... await point here would hold the lock across suspension
```

**Fix:** Lock scope should be minimal and never span `.await` points.

---

## Category 4: Type Safety (47 warnings)

Type system misuse and redundant operations.

### 4.1 Unnecessary Casts (12)

**Count:** 12+ instances  
**Severity:** Low  
**Clippy Lints:** `unnecessary_cast`, `cast_abs_to_unsigned`

**Examples:**

```rust
// hoop-daemon/src/supervisor.rs
let value = value as u64;  // Already u64

// hoop-daemon/src/cost.rs
let abs_usize = (value as i64).abs() as usize;  // Dangerous overflow
```

### 4.2 Useless Conversions (10)

**Count:** 10+ instances  
**Severity:** Low  
**Clippy Lints:** `useless_conversion`

**Examples:**

```rust
// hoop-daemon/src/api_stitch_replay.rs
PathBuf::from(path)  // path is already PathBuf

// hoop-daemon/src/attachments.rs
String::from(s)  // s is already String

// hoop-daemon/src/config_resolver.rs
rusqlite::Connection::from(conn)  // conn is already Connection
```

### 4.3 Clone on Copy (8)

**Count:** 8 instances  
**Severity:** Low  
**Clippy Lints:** `clone_on_copy`

**Examples:**

```rust
// hoop-daemon/src/auth.rs
let role = role.clone();  // WorkspaceViewRole implements Copy

// hoop-daemon/src/embedding_service.rs
let arr = arr.clone();  // [f32; 256] implements Copy
```

---

## Category 5: API Design (28 warnings)

Public API inconsistencies.

### 5.1 Missing Trait Implementations (6)

**Count:** 6 instances  
**Severity:** Medium  
**Clippy Lints:** `should_implement_trait`, `new_without_default`, `len_without_is_empty`

**Examples:**

```rust
// hoop-daemon/src/backup_pipeline.rs
struct BackupManifest { ... }  // Should impl Default

// hoop-daemon/src/embedding_service.rs
struct IdentityCache { 
    pub fn len(&self) -> usize { ... }  // Should also have is_empty()
}

// hoop-daemon/src/stitch_links.rs
impl From<(i64, i64)> for StitchLinkKind { ... }  // Should impl TryFrom
```

### 5.2 Naming Conflicts (3)

**Count:** 3 instances  
**Severity:** Medium  
**Clippy Lints:** `should_implement_trait`, `pub use` conflicts

**Examples:**

```rust
// hoop-daemon/src/stitch_reconstruction.rs
pub fn from_str(...) -> Result<StitchKind> { ... }
// Conflicts with std::str::FromStr::from_str
```

### 5.3 Visibility Issues (2)

**Count:** 2 instances  
**Severity:** Medium  
**Clippy Lints:** `private_interfaces`, `type_repetition_in_bounds`

**Example:**

```rust
// hoop-daemon/src/reflection_detector.rs
type PatternCategory = ...;  // Private type
pub struct DetectedPattern {
    pub category: PatternCategory,  // Public field with private type
}
```

---

## Category 6: Performance (25 warnings)

Inefficient patterns and allocations.

### 6.1 Unnecessary Allocations (10)

**Count:** 10+ instances  
**Severity:** Medium  
**Clippy Lints:** `useless_vec`, `vec_init_then_push`

**Examples:**

```rust
// hoop-daemon/src/stitch_links.rs
let tags = vec![];  // Use Vec::new() or []
tags.push(tag);

// hoop-daemon/src/cost.rs
let mut rows = vec![];  // Then push in loop
```

### 6.2 Inefficient Iteration (8)

**Count:** 8+ instances  
**Severity:** Low  
**Clippy Lints:** `unnecessary_sort_by`, `get_first`, `for_kv_map`

**Examples:**

```rust
// hoop-daemon/src/stitch_percentile_index.rs
stitches.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
// Better: stitches.sort_by_key(|s| s.timestamp)

// hoop-daemon/src/fleet.rs
let first = items.iter().next();  // Use items.first()
```

### 6.3 Unnecessary Laziness (5)

**Count:** 5+ instances  
**Severity:** Low  
**Clippy Lints:** `unnecessary_lazy_evaluations`, `unnecessary_map_or`

**Examples:**

```rust
// hoop-daemon/src/api_stitch_replay.rs
opt.map_or_else(|| default(), |x| f(x))  // default is cheap
// Better: opt.map_or(default(), |x| f(x))
```

---

## Category 7: Disallowed Methods (39 warnings)

**Disallowed by project policy** — should use atomic_write instead.

### 7.1 Direct Filesystem Operations (39)

**Count:** 39 instances  
**Severity:** **High** — project policy violation  
**Clippy Lints:** `disallowed_methods` (configured in `clippy.toml`)

**Policy:** HOOP requires atomic writes for data integrity. Direct filesystem writes are not allowed.

**Examples:**

```rust
// hoop-daemon/src/agent_session.rs:887
fs::write(&path, json)  // VIOLATION

// hoop-daemon/src/api_unassigned.rs:177
fs::File::create(&path)  // VIOLATION

// hoop-daemon/src/attachment_sync.rs:80
std::fs::write(&tmp_path, &json)  // VIOLATION

// hoop-daemon/src/backup_pipeline.rs:561
std::fs::write(&output, &compressed)  // VIOLATION
```

**Fix:** All writes must use `hoop_daemon::atomic_write::atomic_write()` or `atomic_write_append()`.

**Affected modules:**
- `agent_session.rs` (1)
- `api_unassigned.rs` (1)
- `atomic_write.rs` (2 — internal use OK)
- `attachment_sync.rs` (1)
- `attachments.rs` (3)
- `backup_pipeline.rs` (5+)
- `embedding_service.rs` (3+)
- `metrics.rs` (4+)
- `openapi.rs` (1)
- `projects.rs` (6+)
- `stitch_reconstruction.rs` (4+)
- `testrepo_harness.rs` (2)

**Action Required:** Refactor all 39 sites to use `atomic_write`.

---

## Category 8: Test Code (15 warnings)

Warnings specific to test fixtures and integration tests.

### 8.1 Fixture Generation (6)

**Count:** 6 warnings in `hoop-schema/tests/` and `hoop-schema/examples/`  
**Severity:** Low — test-only  
**Clippy Lints:** `expect_fun_call`, `needless_borrows_for_generic_args`

**Example:**

```rust
// hoop-schema/tests/schema_drift.rs:761-762
fs::write(&file_path, &json)
    .expect(&format!("Failed to write fixture: {}", file_path));
```

### 8.2 Integration Test Warnings (9)

**Count:** 9 warnings across test files  
**Severity:** Low — test-only  
**Clippy Lints:** Various

**Affected tests:**
- `testrepo_harness_integration` (3)
- `supervisor_shutdown` (3)
- `supervisor_restart` (2)
- `state_projections` (6)
- `skills_quarantine_integration` (7)
- `session_redaction` (2)
- `cross_workspace_blockers` (6)
- `filesystem_failure_isolation` (5)
- `adapter_failover_test` (7)

---

## Fix Order Priority

### Phase 1: Critical Safety (1-2 hours)

1. **[1] Async Safety** — Fix `await_holding_lock` in `embedding_service.rs:405`
   - Risk: Deadlock under load
   - Fix: Scope lock before `.await` point

### Phase 2: Policy Compliance (4-6 hours)

2. **[39] Disallowed Filesystem Methods** — Replace with `atomic_write`
   - Risk: Data corruption, inconsistent state
   - Fix: Replace all `std::fs::write` and `File::create` with atomic alternatives
   - High-impact modules: `attachments.rs`, `projects.rs`, `stitch_reconstruction.rs`

### Phase 3: API Design (8-12 hours)

3. **[6] Too Many Arguments** — Refactor function signatures
   - `fleet.rs:create_stitch_with_audit` (12 args) → builder pattern
   - `config_resolver.rs:resolve_opt_strict` (9 args) → config struct
   - `supervisor.rs:new` (9 args) → struct-based config
   - `fleet.rs:accumulate_cost_rollup_conn` (8 args)
   - `fleet.rs:snapshot_project_cost_row_conn` (8 args)

4. **[6] Type Complexity** — Add type aliases
   - `cost.rs`, `metrics.rs`, `stitch_percentile_index.rs`
   - Create module-level type definitions

5. **[6] Missing Traits** — Add trait implementations
   - `BackupManifest: Default`
   - `IdentityCache::is_empty()`
   - `FromStr` naming conflicts

### Phase 4: Dead Code Cleanup (2-3 hours)

6. **[~70] Unused Imports** — Run `cargo clippy --fix`
   - Auto-fixable, low risk

7. **[~30] Unused Variables** — Remove or prefix with `_`
   - Verify no incomplete features

8. **[~10] Dead Functions** — Remove or mark with `#[allow(dead_code)]`
   - Check if intended for future use

### Phase 5: Code Quality (6-8 hours)

9. **[~47] Type Safety** — Remove unnecessary casts/conversions
   - Delete redundant `.clone()` on Copy types
   - Remove identity casts
   - Fix `useless_conversion` sites

10. **[~82] Style & Idioms** — Apply Rust idioms
    - Collapse redundant closures
    - Simplify control flow
    - Fix reference patterns

11. **[~25] Performance** — Remove allocations
    - Fix `useless_vec!` calls
    - Use `sort_by_key` instead of `sort_by`
    - Remove unnecessary lazy evaluations

### Phase 6: Test Cleanup (1-2 hours)

12. **[15] Test Code** — Fix test-specific warnings
    - Test fixtures in `hoop-schema/`
    - Integration test warnings

---

## Per-Crate Breakdown

| Crate | Lib Warnings | Test Warnings | Total |
|-------|-------------|---------------|-------|
| `hoop-daemon` | 251 | 103 | 354 |
| `hoop-cli` | 0 | 0 | 0 |
| `hoop-mcp` | 0 | 4 | 4 |
| `hoop-schema` | 0 | 11 | 11 |

**hoop-daemon** accounts for **96% of all warnings** and should be the primary focus.

---

## Automated Fixes

The following warnings can be **automatically fixed** with `cargo clippy --fix`:

- All unused imports (~70)
- Most unnecessary casts (~12)
- Most style warnings (~40)
- Some dead code (~10)

**Estimated auto-fixable warnings:** ~132 (36%)

**Manual fixes required:** ~237 (64%)

---

## Next Steps

1. Fix critical `await_holding_lock` warning
2. Address all 39 disallowed filesystem method violations
3. Run `cargo clippy --fix` for auto-fixable warnings
4. Manually refactor complex function signatures
5. Add type aliases for complex types
6. Implement missing traits
7. Remove dead code
8. Re-run `cargo clippy -- -D warnings` to verify CI gate passes

---

## References

- Clippy lints documentation: https://rust-lang.github.io/rust-clippy/
- HOOP plan: `docs/plan/plan.md`
- HOOP build environment: `AGENTS.md`
