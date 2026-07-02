# hoop-mcp Dependency Availability Verification

**Date:** 2026-07-02
**Bead ID:** bf-36fwp
**Status:** ✓ All dependencies verified and available
**Verification Method:** cargo fetch + cargo check

## Summary

All 18 dependencies listed in `hoop-mcp/Cargo.toml` are successfully available from their specified sources. No 404 or not-found errors encountered.

## Dependency Categories

### 1. Regular crates.io dependencies (15 packages)

All verified via successful `cargo fetch` and `cargo check`:

| Package | Version Request | Status |
|---------|-----------------|--------|
| anyhow | "1.0" | ✓ Available |
| serde | "1.0" (derive) | ✓ Available |
| serde_yaml | "0.9" | ✓ Available |
| chrono | "0.4" (serde) | ✓ Available |
| uuid | "1" (v4) | ✓ Available |
| dirs | "5.0" | ✓ Available |
| rusqlite | "0.30" (bundled) | ✓ Available |
| sha2 | "0.10" | ✓ Available |
| hex | "0.4" | ✓ Available |
| regex | "1" | ✓ Available |
| ignore | "0.4" | ✓ Available |
| clap | "4.5" (derive) | ✓ Available |
| reqwest | "0.12" (json, blocking, rustls-tls) | ✓ Available |
| jsonschema | "0.26" | ✓ Available |
| fnv | "1.0" | ✓ Available |

### 2. Workspace dependencies (4 packages)

Inherited from root workspace Cargo.toml:

| Package | Status |
|---------|--------|
| tokio | ✓ Available (workspace: 1.35, full) |
| tracing | ✓ Available (workspace: 0.1) |
| tracing-subscriber | ✓ Available (workspace: 0.3, env-filter) |
| serde_json | ✓ Available (workspace: 1.0) |

### 3. Local path dependency (1 package)

| Package | Path | Status |
|---------|------|--------|
| hoop-schema | ../hoop-schema | ✓ Available |

### 4. Dev dependencies (2 packages)

| Package | Version Request | Status |
|---------|-----------------|--------|
| tempfile | "3" | ✓ Available |
| trybuild | "1.0" | ✓ Available |

## Verification Commands

```bash
# Fetch all dependencies (retrieves from crates.io and resolves workspace deps)
cargo fetch --manifest-path hoop-mcp/Cargo.toml
# Result: Success (no errors)

# Verify dependency resolution and compilation
cargo check --manifest-path hoop-mcp/Cargo.toml
# Result: Success (no errors)

# Verify local path dependency exists
ls hoop-schema/
# Result: Directory exists with valid Cargo.toml
```

## Acceptance Criteria Met

- ✓ All dependencies are available from their specified sources
- ✓ No 404 or not-found errors when resolving dependencies
- ✓ Path dependencies are accessible (hoop-schema exists)
- ✓ Git dependencies: None in hoop-mcp (all are crates.io or workspace/path)
- ✓ Dependency resolution completes successfully via cargo

## Conclusion

**All 22 total dependencies (15 crates.io + 4 workspace + 1 path + 2 dev) are verified available.** The hoop-mcp crate can successfully fetch and resolve all dependencies for building and testing.
