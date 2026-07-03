# HOOP Debug Build Command

## Summary

Identified and verified the debug build command for HOOP.

## Debug Build Commands

### Standard command (works on this Debian system):
```bash
cargo build
```

### On NixOS systems (via nix-shell wrapper):
```bash
nix-shell --run 'cargo build'
```

The `shell.nix` file at the repo root provides required dependencies (pkg-config, openssl, rustc, node, pnpm) for NixOS environments.

## Current Build Status

The debug build command is correct, but **currently fails** due to compilation errors in the codebase:

- **Error type**: OpenAPI schema trait bounds not satisfied
- **Affected types**: `ScriptRunRequest`, `EnableTourRequest`, `ListJobsQuery`
- **Missing traits**: `ToSchema`, `PartialSchema`
- **Error count**: 22 compilation errors, 74 warnings (as of 2026-07-02)

These compilation errors are code issues, not build command issues. The correct build command is documented above; once the OpenAPI schema errors are fixed, the build will succeed.

## Build Output Location

Debug binaries are placed in:
```
target/debug/
├── hoop           # hoop-cli binary
├── hoop-daemon    # hoop-daemon binary  
└── hoop-mcp       # hoop-mcp binary
```

## Verification

- ✅ Build command syntax verified
- ✅ Cargo available at `/home/coding/.local/bin/cargo` (v1.95.0)
- ⚠️ Build fails due to existing compilation errors (OpenAPI traits)

## Environment Notes

- Current system: Debian GNU/Linux 13 (trixie)
- Cargo wrapper: `/home/coding/.local/bin/cargo` (intercepts `cargo test` for remote CI offloading)
- Rust version: 1.95.0
- Workspace members: hoop-cli, hoop-daemon, hoop-schema, hoop-ui, hoop-mcp
