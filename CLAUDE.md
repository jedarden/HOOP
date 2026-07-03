# HOOP — Claude Code Notes

See [AGENTS.md](AGENTS.md) for the full repository guide.

## Before running tests

HOOP integration tests spawn long-lived subprocesses that do **not** self-terminate on failure. Leaked processes accumulate across sessions and cause OOM kills on the lab server.

Always kill any lingering test processes before starting a new test run.

**Option 1: Use the cleanup script (recommended)**

```bash
bin/cleanup-hoop-test-processes.sh
```

**Option 2: Quick one-liner**

```bash
pkill -f 'HOOP/target/debug/deps/hoop' 2>/dev/null || true
pkill -f 'testrepo/bin/' 2>/dev/null || true
```

Then run tests via nix-shell (bare cargo fails on NixOS — see AGENTS.md):

```bash
nix-shell --run 'cargo test'
```

After tests complete (pass or fail), verify no processes remain:

```bash
ps aux | grep 'HOOP/target' | grep -v grep
```

Kill any survivors before finishing the session.

**See also:** `docs/test-process-cleanup-patterns.md` for comprehensive patterns and edge cases.
