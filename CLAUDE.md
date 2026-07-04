# HOOP — Claude Code Notes

See [AGENTS.md](AGENTS.md) for the full repository guide.

## Before running tests

HOOP integration tests spawn long-lived subprocesses that do **not** self-terminate on failure. Leaked processes accumulate across sessions and cause OOM kills on the lab server.

Always kill any lingering test processes before starting a new test run.

**Option 1: Quick pkill one-liner (recommended)**

```bash
pkill -f 'hoop-[a-f0-9]{16,}$' && pkill -f 'hoop_daemon-[a-f0-9]{16,}$' && pkill -f 'testrepo/(bin|scripts)/' && pkill -9 -f 'build-script-build$' || true
```

**Option 2: Use the comprehensive cleanup script**

```bash
bin/cleanup-hoop-test-processes.sh
```

**Option 3: Use the simple cleanup script**

```bash
bin/kill-hoop-test-processes
```

Then run tests via nix-shell (bare cargo fails on NixOS — see AGENTS.md):

```bash
nix-shell --run 'cargo test'
```

After tests complete (pass or fail), verify no processes remain:

```bash
./bin/verify-hoop-test-processes.sh
```

For a quick check without the script:

```bash
ps aux | grep 'HOOP/target' | grep -v grep
```

Kill any survivors before finishing the session.

**See also:** `docs/test-process-cleanup-patterns.md` for comprehensive patterns and edge cases.
