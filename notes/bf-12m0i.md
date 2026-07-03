# nix-shell not available on HOOP build server (bf-12m0i)

## Finding

The HOOP build server is **NOT running NixOS** and does **NOT** have nix-shell available. However, this is **NOT a blocker** because all required dependencies are already installed natively.

## Actual Environment

- **OS:** Debian GNU/Linux 13 (trixie)
- **nix-shell:** NOT available
- **rustc:** 1.95.0 (available natively)
- **cargo:** 1.95.0 (available natively)
- **node:** v20.19.2
- **pnpm:** 10.33.1
- **pkg-config:** 1.8.1
- **libssl-dev:** installed
- **sqlite3:** installed
- **jq, git, tmux:** installed

## Verification

```bash
$ cargo check
    Checking hoop-daemon v1.0.0 (/home/coding/HOOP/hoop-daemon)
    ...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.23s
```

All cargo commands work **without nix-shell**.

## Resolution

The `shell.nix` file in the repository is a **fallback** for NixOS systems. On this Debian build server, all dependencies are installed natively and cargo commands work directly.

**AGENTS.md contains outdated information** stating "This server runs NixOS. Bare cargo check will fail... Always use nix-shell." This is incorrect for the current build server.

## Impact on Beads

- **bf-3lu60** (Verify rustc is accessible in nix-shell): The nix-shell verification step is **NOT applicable** on this server. Rust toolchain verification should use native rustc/cargo instead.
- **bf-12m0i**: RESOLVED - nix-shell is not required; all dependencies available natively

## Recommendation

Update AGENTS.md to clarify:
1. The primary build server is Debian (not NixOS)
2. nix-shell is only needed for NixOS systems
3. On Debian, use cargo commands directly
4. shell.nix is provided as a fallback for NixOS environments
