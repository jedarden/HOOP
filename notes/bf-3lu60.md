# bf-3lu60: nix-shell Rust Toolchain Verification

## Task
Verify rustc is accessible and working within the nix-shell environment for HOOP.

## Findings

### Issue: nix-shell is NOT available on this system

**Evidence:**
- `nix-shell --run 'rustc --version'` → `bash: nix-shell: command not found`
- `nix-shell` not in PATH
- `~/.nix-profile/bin/nix-shell` does not exist
- `/etc/os-release` confirms this is NOT a NixOS system

### What DOES work
- `rustc --version` (direct, no nix-shell): `rustc 1.95.0 (59807616e 2026-04-14)`

## Impact

The HOOP project build process per `AGENTS.md` explicitly requires nix-shell:

> "Bare cargo check / cargo build / cargo test will fail with an openssl-sys / pkg-config not found error. Always use nix-shell"

This is a blocking infrastructure issue. The acceptance criteria cannot be met:
- ~~rustc --version executes within nix-shell~~ - CANNOT TEST (nix-shell not available)

## Resolution Path

Either:
1. Install Nix package manager on this server, OR
2. Update HOOP build process to not require nix-shell (use system rustc + deps directly)

Created bug bead `bf-12m0i` as a dependency for this task.
