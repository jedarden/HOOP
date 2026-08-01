# HOOP Toolchain Versions Documentation

## Task
Document and verify toolchain versions for HOOP development environment.

## Current Toolchain Versions (as of 2026-08-01)

### Rust Toolchain
- **cargo:** 1.95.0 (f2d3ce0bd 2026-03-21)
- **rustc:** 1.95.0 (59807616e 2026-04-14)
- **Rust Edition:** 2021 (workspace default)

### Node Toolchain (for UI components)
- **Node.js:** v20.19.2
- **pnpm:** 10.33.1

## Minimum Requirements

According to `Cargo.toml` workspace configuration:
- **Minimum rust-version:** 1.75
- **Workspace edition:** 2021
- **Current version:** 1.95.0

### Compatibility Verification
✅ **Compatible**: Current rustc version 1.95.0 exceeds minimum requirement of 1.75
✅ **Compatible**: Current cargo version 1.95.0 is aligned with rustc version
✅ **Compatible**: Rust Edition 2021 is properly configured

## Environment Access

**Current Environment:** Debian GNU/Linux 13 (trixie)

Toolchain versions are accessed directly (no nix-shell required on Debian):
```bash
cargo --version && rustc --version
```

For NixOS development environments, the same toolchain is available via:
```bash
nix-shell --run 'cargo --version && rustc --version'
```

The development environment includes:
- Rust toolchain (rustc, cargo) via ~/.cargo/bin
- Node.js and pnpm for UI development
- All required dependencies for HOOP development

## Build System Compatibility

The HOOP project uses standard Rust build tooling:
- **Build system:** cargo via workspace resolver
- **Workspace members:** hoop-cli, hoop-daemon, hoop-schema, hoop-ui, hoop-mcp, test_backup_deser
- **Testing:** Standard cargo test framework with integration tests
- **Package management:** Cargo workspace with shared dependencies

## Verification Status

All toolchain versions have been verified on 2026-08-01:
- ✅ cargo builds and runs correctly
- ✅ rustc compiles Rust code successfully
- ✅ Minimum version requirements are met (1.95.0 > 1.75)
- ✅ All workspace dependencies are resolvable
- ✅ Debian 13 build environment fully compatible
- ✅ No nix-shell required on Debian systems

### Build Verification Results
Current toolchain successfully builds all workspace members:
- hoop-schema ✅
- hoop-ui ✅
- hoop-daemon ✅
- hoop-cli ✅
- hoop-mcp ✅
- test_backup_deser ✅

## Notes

- Rust toolchain installed via ~/.cargo/bin (rustup managed)
- Node.js and pnpm via system packages and standalone binaries
- For NixOS development: use `shell.nix` at repo root for all dependencies
- Version alignment between cargo and rustc ensures proper compatibility
- Minimum rust-version of 1.75 specified in workspace Cargo.toml is well exceeded
