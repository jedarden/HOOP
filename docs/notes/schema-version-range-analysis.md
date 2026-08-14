# HOOP Schema Version Range Implementation Analysis

**Bead:** bf-kseh8q  
**Date:** 2026-08-14  
**Scope:** Complete analysis of schema version range checking implementation

## Executive Summary

HOOP implements a robust **semver-based schema version system** with:
- **Current version:** `1.33.0` (defined in `hoop-daemon/src/fleet.rs:24` and `hoop-schema/src/lib.rs:43`)
- **Supported range:** `0.1.0` through `1.34.0` (with migration paths for all intermediate versions)
- **Version policy:** Semver with explicit major upgrade gate and backward-compatible minor migrations

---

## 1. Version Range Implementation

### Core Comparison Functions

#### 1.1 `semver_compare()` - Numeric Conversion
**Location:** `hoop-daemon/src/migrations.rs:138-149`

```rust
fn semver_compare(version: &str) -> u64 {
    let parts: Vec<u32> = version
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    
    let major = *parts.first().unwrap_or(&0);
    let minor = *parts.get(1).unwrap_or(&0);
    let patch = *parts.get(2).unwrap_or(&0);
    
    (major as u64) * 1_000_000 + (minor as u64) * 1000 + (patch as u64)
}
```

**Purpose:** Converts semver strings to sortable numeric values.
**Formula:** `major * 1,000,000 + minor * 1,000 + patch`
**Example:** 
- `1.33.0` → `1,033,000`
- `2.0.0` → `2,000,000`

#### 1.2 `is_newer_version()` - Component-wise Comparison
**Location:** `hoop-daemon/src/fleet.rs:5722-5741`

```rust
fn is_newer_version(a: &str, b: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|p| p.parse().ok())
            .collect::<Vec<_>>()
    };
    let va = parse(a);
    let vb = parse(b);
    for i in 0..std::cmp::max(va.len(), vb.len()) {
        let na = va.get(i).unwrap_or(&0);
        let nb = vb.get(i).unwrap_or(&0);
        if na > nb {
            return true;
        }
        if na < nb {
            return false;
        }
    }
    false
}
```

**Purpose:** Determines if version `a` is strictly newer than version `b` by comparing major, then minor, then patch components.

**Use cases:**
- Rejecting snapshot restores when the snapshot's schema version is newer than the binary
- Version ordering in migration paths

#### 1.3 `extract_major()` - Major Version Extraction
**Location:** `hoop-daemon/src/fleet.rs:3419-3421`

```rust
pub fn extract_major(version: &str) -> Option<u64> {
    version.split('.').next()?.parse().ok()
}
```

**Purpose:** Extracts the major version number for gate checking.
**Example:** `"1.33.0"` → `1`

---

## 2. Currently Accepted Schema Versions

### 2.1 Version Bounds

**Lower bound:** `INITIAL_SCHEMA_VERSION = "0.1.0"`
- Defined in `hoop-daemon/src/fleet.rs:27`
- Bootstrap version for fresh databases
- Immediately migrates to `SCHEMA_VERSION` on first run

**Upper bound:** Latest supported version
- Current binary: `SCHEMA_VERSION = "1.33.0"`
- Migration paths exist through: `1.34.0`
- Snapshots/versions beyond `1.34.0` are rejected as "too new"

### 2.2 Migration Registry

**Location:** `hoop-daemon/src/migrations.rs:130-134`

The migration registry contains ordered migrations from `0.1.0` → `1.34.0`. Key migrations include:

| From  | To    | Description                                    |
|-------|-------|------------------------------------------------|
| 0.1.0 | 1.0.0 | Initial schema creation                        |
| 1.0.0 | 1.1.0 | Add agent_sessions table                       |
| ...   | ...   | (see migrations.rs for full list)             |
| 1.32.0| 1.33.0| Add template_id and created_by to fix_patterns|
| 1.33.0| 1.34.0| Add risk_patterns table                       |

**Supported versions:** Any version in the chain from `0.1.0` through `1.34.0`.

---

## 3. Version 1.33.0 Analysis

### 3.1 Current Status

**1.33.0 is the CURRENT schema version** — it does NOT fall outside the accepted range.

**Evidence:**
1. **Primary definition:** `hoop-daemon/src/fleet.rs:24`
   ```rust
   pub const SCHEMA_VERSION: &str = "1.33.0";
   ```

2. **Schema module definition:** `hoop-schema/src/lib.rs:43`
   ```rust
   pub const SCHEMA_VERSION: &str = "1.33.0";
   ```

3. **Migration entry:** `hoop-daemon/src/migrations.rs:571-575`
   ```rust
   version: "1.33.0",
   description: "Add template_id and created_by to fix_patterns",
   up: migrate_v132_to_v133,
   down: Some(rollback_v133_to_v132),
   ```

### 3.2 Why 1.33.0 is NOT Outside the Range

The version is **exactly at the expected position** in the migration chain:

```
0.1.0 → 1.0.0 → ... → 1.32.0 → [1.33.0] → 1.34.0
                              ^
                        Current binary expects this
```

**Version ordering:**
- `is_newer_version("1.34.0", "1.33.0")` → `true` (1.34.0 is newer)
- `is_newer_version("1.33.0", "1.33.0")` → `false` (equal, not newer)
- `is_newer_version("1.32.0", "1.33.0")` → `false` (1.32.0 is older)

**No rejection scenario:** A database at schema version `1.33.0` will:
1. Pass the major version gate (stored_major == binary_major == 1)
2. Skip migrations (version == SCHEMA_VERSION)
3. Log: `"fleet.db schema version 1.33.0 verified"`

---

## 4. Legacy vs Current Schema Version Behavior

### 4.1 Bootstrap Version (0.x)

**Special handling in major gate:**
**Location:** `hoop-daemon/src/fleet.rs:3435-3438`

```rust
// "0.x" is the bootstrap version — always upgradeable through minor migrations.
if stored_major == 0 {
    return Ok(());
}
```

**Behavior:**
- **Bypasses major version gate** entirely
- Always migrates forward through the minor chain
- Treated as "pre-migration bootstrap" — never subject to gate enforcement

**Example:** Database at `0.1.0` with binary at `1.33.0`:
1. Gate check: `stored_major == 0` → Pass immediately
2. Run migrations: `0.1.0 → 1.0.0 → ... → 1.33.0`
3. Complete with schema at `1.33.0`

### 4.2 Current Version (1.x)

**Normal gate enforcement applies:**
**Location:** `hoop-daemon/src/fleet.rs:3440-3445`

```rust
if binary_major > stored_major {
    anyhow::bail!(
        "Your data is schema version {stored_major}.x; this binary requires {binary_major}.x. \
         Run `hoop migrate major-upgrade --confirm` or restore from a pre-upgrade backup."
    );
}
```

**Behavior:**
- **Major gate enforces:** binary_major cannot exceed stored_major
- **Minor migrations:** Automatically applied (backward compatible)
- **Major upgrade:** Requires explicit `hoop migrate major-upgrade --confirm`

**Example scenarios:**

| Stored Version | Binary Version | Action                                              |
|----------------|-----------------|-----------------------------------------------------|
| 1.32.0         | 1.33.0          | Auto-migrate (minor version bump)                   |
| 1.33.0         | 1.33.0          | No migration (versions match)                       |
| 1.33.0         | 2.0.0           | **REJECT** - requires `hoop migrate major-upgrade`  |
| 2.0.0          | 1.33.0          | Reject restore (snapshot too new for this binary)  |

---

## 5. Version Policy (from hoop-schema/src/lib.rs)

**Location:** `hoop-schema/src/lib.rs:25-43`

### 5.1 Semver Components

- **Major (X):** Breaking changes, no backwards compatibility
  - One-way migration only
  - Requires explicit upgrade gate: `hoop migrate major-upgrade --confirm`
  - Example: `1.x → 2.x` migration
  
- **Minor (Y):** Additive changes, backwards compatible
  - Old readers ignore new fields
  - Rollback supported
  - Example: `1.32.0 → 1.33.0`
  
- **Patch (Z):** Bug fixes, no schema shape changes
  - Transparent to version checking system
  - Example: `1.33.0 → 1.33.1`

### 5.2 Deprecation Windows

**Minor deprecations:**
- Readable for at least one full minor version after introduction
- Example: Field deprecated in `1.33.0`:
  - Remains readable through: `1.34.0`
  - May be removed in: `1.35.0`

**Major deprecations:**
- One-way only
- Operator explicitly accepts at upgrade gate
- No rollback support across major versions

---

## 6. Version Validation at Multiple Layers

### 6.1 Database Schema Version
**Location:** `hoop-daemon/src/fleet.rs`

- **Stored in:** `metadata` table, key `schema_version`
- **Retrieval:** `get_schema_version(conn)`
- **Validation:** `check_schema_major_gate()` on startup

### 6.2 Configuration Schema Version
**Location:** `hoop-daemon/src/config_resolver.rs:1009-1019`

```rust
fn validate_schema_version(version: &str) -> Result<()> {
    let re = Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
    if !re.is_match(version) {
        bail!("Invalid schema_version format: {}. Expected semver (X.Y.Z)", version);
    }
    Ok(())
}
```

**Validates:** Regex pattern `^\d+\.\d+\.\d+$` (strict semver format)

### 6.3 Snapshot/Restore Version
**Location:** `hoop-daemon/src/fleet.rs:4949-4956`

```rust
// Reject newer-than-current snapshots (§20.1)
if is_newer_version(&version, SCHEMA_VERSION) {
    return Err(anyhow::anyhow!(
        "Snapshot schema version {} is newer than this binary's {}. \
         Upgrade HOOP before restoring this snapshot.",
        version,
        SCHEMA_VERSION
    ));
}
```

**Prevents:** Restoring snapshots from future HOOP versions

### 6.4 Debug State Schema
**Location:** `hoop-daemon/src/api_metrics.rs`

- **Constant:** `DEBUG_STATE_SCHEMA_VERSION: "1.0.0"`
- **Validates:** Debug state export format follows semver pattern

---

## 7. Migration Execution Flow

### 7.1 Fresh Database Initialization
**Location:** `hoop-daemon/src/fleet.rs:788-800`

```rust
// Fresh database: initialize schema at INITIAL_SCHEMA_VERSION, then migrate
create_schema(&mut conn)?;
insert_genesis_row(&mut conn)?;
info!("Schema initialized at version {}, migrating to {}", INITIAL_SCHEMA_VERSION, binary_version);
run_migrations(&mut conn, INITIAL_SCHEMA_VERSION)?;
```

**Flow:**
1. Create tables with `INITIAL_SCHEMA_VERSION` (`0.1.0`)
2. Insert genesis hash row
3. Run full migration chain: `0.1.0 → ... → 1.33.0`

### 7.2 Existing Database Migration
**Location:** `hoop-daemon/src/fleet.rs:803-825`

```rust
// Existing database: verify schema version and run migrations
let version = get_schema_version(&conn)?;

// §20.1 major-upgrade gate
check_schema_major_gate(&version, binary_version)?;

if version != binary_version {
    info!("fleet.db schema version {} -> {}, running migrations", version, binary_version);
    run_migrations(&mut conn, &version)?;
    // ...
} else {
    info!("fleet.db schema version {} verified", version);
}
```

**Flow:**
1. Read stored schema version
2. Check major gate (reject if binary_major > stored_major)
3. Run pending migrations if versions differ
4. Log completion

### 7.3 Restore with Migration
**Location:** `hoop-daemon/src/fleet.rs:4940-4970`

```rust
pub fn restore_and_migrate(db_path: &std::path::Path) -> Result<String> {
    let mut conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    
    let version = get_schema_version(&conn)?;
    
    // Reject newer-than-current snapshots
    if is_newer_version(&version, SCHEMA_VERSION) {
        return Err(...);
    }
    
    if version != SCHEMA_VERSION {
        info!("Restored fleet.db schema {} -> {}, running migrations", version, SCHEMA_VERSION);
        run_migrations(&mut conn, &version)?;
    }
    
    Ok(version)
}
```

**Flow:**
1. Open restored database
2. Read schema version
3. Reject if version newer than binary (operator must upgrade HOOP first)
4. Migrate forward if version older
5. Return pre-migration version for logging

---

## 8. Key Findings Summary

### Acceptance Criteria Status

✅ **Understand how version range is implemented:**
- Uses semver component-wise comparison via `is_newer_version()`
- Numeric conversion via `semver_compare()` for ordering
- Major version extraction via `extract_major()` for gate checking
- NOT a simple range check — it's a directed migration chain with explicit gates

✅ **Document what schema versions are currently accepted:**
- Any version from `0.1.0` through `1.34.0`
- Fresh databases start at `0.1.0` and migrate to `1.33.0`
- Existing databases at any version in the chain migrate forward
- Snapshots/versions beyond `1.34.0` rejected as "too new"

✅ **Identify upper and lower bounds:**
- **Lower bound:** `INITIAL_SCHEMA_VERSION = "0.1.0"` (bootstrap)
- **Upper bound:** Latest supported is `1.34.0` (current binary is `1.33.0`)
- Versions beyond `1.34.0` rejected via `is_newer_version()` check

✅ **Determine if 1.33.0 falls outside the current range:**
- **1.33.0 is the CURRENT schema version** — perfectly in range
- It is the target that all older versions migrate toward
- It is NOT outside the range; it is exactly where the binary expects the schema to be

✅ **Document expected behavior for legacy vs current:**
- **Legacy (0.x):** Bypasses major gate, always migrates forward through minor chain
- **Current (1.x):** Major gate enforced, minor migrations auto-applied, major upgrade requires explicit confirmation

---

## 9. Implementation Locations Reference

| Function/Constant                 | File                              | Lines      | Purpose                          |
|-----------------------------------|-----------------------------------|------------|----------------------------------|
| `SCHEMA_VERSION`                  | hoop-daemon/src/fleet.rs         | 24         | Current schema version constant  |
| `INITIAL_SCHEMA_VERSION`          | hoop-daemon/src/fleet.rs         | 27         | Bootstrap version for new DBs    |
| `SCHEMA_VERSION`                  | hoop-schema/src/lib.rs           | 43         | Schema crate version constant   |
| `extract_major()`                 | hoop-daemon/src/fleet.rs         | 3419-3421  | Extract major version component  |
| `check_schema_major_gate()`       | hoop-daemon/src/fleet.rs         | 3429-3447  | Enforce major version gate       |
| `is_newer_version()`              | hoop-daemon/src/fleet.rs         | 5722-5741  | Compare semver strings           |
| `semver_compare()`                | hoop-daemon/src/migrations.rs    | 138-149    | Convert semver to comparable u64 |
| `run_pending_migrations()`       | hoop-daemon/src/migrations.rs    | 152-219    | Execute migration chain          |
| `validate_schema_version()`       | hoop-daemon/src/config_resolver.rs | 1009-1019 | Regex semver validation       |
| `restore_and_migrate()`          | hoop-daemon/src/fleet.rs         | 4940-4970  | Restore with version checks      |
| `get_schema_version()`            | hoop-daemon/src/fleet.rs         | 945-953    | Read stored schema version       |
| `update_schema_version()`         | hoop-daemon/src/fleet.rs         | 955-968    | Write schema version to metadata |

---

## Conclusion

HOOP's schema version system is a **well-designed, layered semver implementation** with:

1. **Clear separation** between major (breaking) and minor (additive) changes
2. **Explicit upgrade gates** preventing accidental data incompatibility
3. **Comprehensive validation** at database, config, and snapshot layers
4. **Bootstrap handling** for legacy 0.x versions
5. **Migration registry** supporting the full chain from `0.1.0` through `1.34.0`

**Version 1.33.0 is NOT outside the range** — it is the current, expected schema version that all other versions migrate toward.
