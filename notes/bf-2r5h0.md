# nix-shell Verification (bf-2r5h0)

## Finding

nix-shell works when called with full path, but is not in the standard PATH.

## Details

- **Issue:** `nix-shell` command not found in PATH
- **Location:** Nix binaries available at `/nix/var/nix/profiles/default/bin/nix-shell`
- **Version:** Determinate Nix 3.18.0 (nix 2.33.3)

## Test Results

When run with full path, nix-shell enters successfully:

```bash
/nix/var/nix/profiles/default/bin/nix-shell --run 'echo "nix-shell entered successfully"'
```

Output:
```
HOOP development environment loaded
Rust version: rustc 1.95.0 (59807616e 2026-04-14) (built from a source tarball)
Node version: v22.23.1
pnpm version: 11.9.0
nix-shell entered successfully
```

## Recommendation

The nix-shell entry works correctly. The PATH issue is a system configuration problem that affects all Nix commands, not specific to HOOP. Users should either:
1. Use the full path: `/nix/var/nix/profiles/default/bin/nix-shell`
2. Add Nix to PATH in their shell profile
