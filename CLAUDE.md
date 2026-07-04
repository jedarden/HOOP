# HOOP — Claude Code Notes

See [AGENTS.md](AGENTS.md) for the full repository guide.

## Before running tests

HOOP integration tests spawn long-lived subprocesses that do **not** self-terminate on failure. Leaked processes accumulate across sessions and cause OOM kills on the lab server.

**Using the Makefile (recommended):**

The Makefile test targets automatically handle cleanup before and after tests:

```bash
make test              # Unit tests with auto cleanup
make test-load         # Load tests with auto cleanup
make test-load-medium  # Medium-scale load test with auto cleanup
make test-load-full    # Full-scale load test with auto cleanup
```

**Manual cleanup before running tests directly via cargo:**

If running `cargo test` directly (not via Makefile), always kill lingering processes first:

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

**After tests complete:**

- **Via Makefile:** Verification runs automatically after `make test` / `make test-load*`
- **Via cargo:** Verify manually after tests complete (pass or fail):

```bash
./bin/verify-hoop-test-processes.sh
```

For a quick check without the script:

```bash
ps aux | grep 'HOOP/target' | grep -v grep
```

Kill any survivors before finishing the session.

**See also:** `docs/test-process-cleanup-patterns.md` for comprehensive patterns and edge cases.
