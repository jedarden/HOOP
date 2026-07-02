# hoop-mcp Dependency Verification

**Date:** 2026-07-02
**Bead:** bf-3nmrb
**Related beads:** bf-1iqx8, bf-64pqx, bf-2jjv3, bf-3ll10

## Summary

The `hoop-mcp` crate dependency resolution has been verified. All dependencies are correctly configured and available. The crate compiles successfully with `cargo check -p hoop-mcp`.

## Current State of hoop-mcp/Cargo.toml

The `hoop-mcp/Cargo.toml` file is correctly structured with:
- Valid TOML syntax
- All required package fields present
- Proper workspace dependency references
- Appropriate external dependency versions

### Package Configuration
```toml
[package]
name = "hoop-mcp"
version.workspace = true
edition.workspace = true

[[bin]]
name = "hoop-mcp"
path = "src/main.rs"
```

### Features
- `default = []` - No default features enabled
- `zero-write-v01 = []` - Feature flag for zero-write v0.1 mode
- `create-only-write = []` - Feature flag for create-only write mode

## Verified Dependencies

### Workspace Dependencies (from workspace Cargo.toml)

| Dependency | Version | Features Used | Status |
|------------|---------|---------------|--------|
| tokio | 1.35 | full | ✓ Verified |
| tracing | 0.1 | - | ✓ Verified |
| tracing-subscriber | 0.3 | env-filter | ✓ Verified |
| serde | 1.0 | derive | ✓ Verified |
| serde_json | 1.0 | - | ✓ Verified |
| chrono | 0.4 | serde | ✓ Verified |

### Local Workspace Dependencies

| Dependency | Path | Status |
|------------|------|--------|
| hoop-schema | ../hoop-schema | ✓ Verified (workspace member) |

### External Dependencies

| Dependency | Version | Features | Purpose | Status |
|------------|---------|----------|---------|--------|
| anyhow | 1.0 | - | Error handling | ✓ Verified |
| serde_yaml | 0.9 | - | YAML parsing | ✓ Verified |
| uuid | 1 | v4 | UUID generation | ✓ Verified |
| dirs | 5.0 | - | Path resolution | ✓ Verified |
| rusqlite | 0.30 | bundled | SQLite database | ✓ Verified |
| sha2 | 0.10 | - | SHA-2 hashing | ✓ Verified |
| hex | 0.4 | - | Hex encoding | ✓ Verified |
| regex | 1 | - | Regular expressions | ✓ Verified |
| ignore | 0.4 | - | Gitignore-style patterns | ✓ Verified |
| clap | 4.5 | derive | CLI argument parsing | ✓ Verified |
| reqwest | 0.12 | json, blocking, rustls-tls | HTTP client | ✓ Verified |
| jsonschema | 0.26 | - | JSON Schema validation | ✓ Verified |
| fnv | 1.0 | - | FNV hashing | ✓ Verified |

### Dev Dependencies

| Dependency | Version | Purpose | Status |
|------------|---------|---------|--------|
| tempfile | 3 | Temporary file creation | ✓ Verified |
| trybuild | 1.0 | Compile testing | ✓ Verified |

## Compilation Status

- **`cargo check -p hoop-mcp`**: ✓ **PASSES** (no errors)
- **`cargo tree -p hoop-mcp`**: ✓ **RESOLVES** (all dependencies found)

## Issues Found and Resolution

### No Issues Found

All verification beads completed successfully:
- ✓ **bf-1iqx8**: Cargo.toml structure and syntax verified
- ✓ **bf-64pqx**: All dependencies verified as available
- ✓ **bf-2jjv3**: cargo check completed successfully

No missing, conflicting, or unresolved dependencies were identified.

## Dependency Tree Highlights

The cargo tree analysis shows:
- All workspace dependencies resolve correctly
- Local `hoop-schema` dependency is properly linked
- External dependencies are pulled from crates.io with appropriate versions
- No duplicate dependency versions or conflicts
- Transitive dependencies are within expected ranges

### Key Transitive Dependencies
- **chrono**: pulls in `iana-time-zone`, `num-traits`, `serde`
- **clap**: pulls in extensive CLI parsing ecosystem
- **rusqlite**: uses bundled SQLite feature (no system sqlite required)
- **reqwest**: configured with rustls-tls (no OpenSSL dependency)

## Security and Configuration Notes

### TLS Configuration
`reqwest` is configured with `rustls-tls` feature, avoiding OpenSSL dependencies. This is consistent with the NixOS environment and matches workspace conventions.

### SQLite Bundling
`rusqlite` uses the `bundled` feature, ensuring the SQLite library is included in the crate rather than depending on system sqlite3 libraries. This improves portability and matches NixOS conventions.

### Error Handling Strategy
The crate uses `anyhow` for error handling, which is consistent with the broader Rust ecosystem and provides flexible error context for the MCP server operations.

## Documentation Status

No documentation updates were required. The existing `AGENTS.md` and `CLAUDE.md` files already reference the correct crate structure and dependencies.

## Conclusion

The `hoop-mcp` crate dependency configuration is correct and complete. All dependencies resolve successfully, the crate compiles without errors, and no issues were found during the verification process. The crate is ready for continued development and testing.
