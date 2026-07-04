# HOOP Test Process Cleanup Workflow Guide

## Overview

HOOP integration tests spawn long-lived subprocesses that do not self-terminate on failure. This guide explains when and how to use the cleanup tools to prevent process accumulation and OOM kills.

## When to Run Cleanup

### Always Run Before Tests

Run cleanup **before** starting any test session to ensure a clean environment:

```bash
./bin/kill-hoop-test-processes
```

### Always Run After Tests

Run cleanup **after** tests complete (even if they pass) to catch any leaked processes:

```bash
./bin/kill-hoop-test-processes
```

### Run When Tests Hang or Fail

If tests hang, fail unexpectedly, or timeout:

```bash
./bin/kill-hoop-test-processes --force
```

The `--force` flag uses SIGKILL instead of SIGTERM for more aggressive cleanup.

## Cleanup Tools

### 1. Comprehensive Cleanup Script

**Script:** `./bin/kill-hoop-test-processes`

**Coverage:** 27 process patterns (primary, subprocess, edge cases)

**Features:**
- Targeted cleanup (only kills HOOP-related processes)
- Hierarchical subprocess detection
- Orphaned process handling
- Colored output and progress reporting

**Usage:**

```bash
# Standard cleanup with SIGTERM
./bin/kill-hoop-test-processes

# Cleanup with verification
./bin/kill-hoop-test-processes --verify

# Force cleanup with SIGKILL
./bin/kill-hoop-test-processes --force
```

**Exit Codes:**
- `0` - All processes cleaned successfully (verification passed if --verify used)
- `1` - Verification failed (processes still present)

**Process Coverage:**

| Category | Patterns | Example |
|----------|----------|---------|
| **Primary (9)** | HOOP test binaries | `hoop-0964aabb985f3f32` |
| | HOOP daemon binaries | `hoop_daemon-37d6544c861da05a` |
| | Target/debug/deps | `HOOP/target/debug/deps/hoop` |
| | Testrepo stub binaries | `testrepo/bin/br` |
| | Testrepo scripts | `testrepo/scripts/myscript` |
| | Cargo build scripts | `build-script-build` |
| | Build script processes | `target/debug/build/*/build-script-build` |
| **Subprocess (15)** | br (beads CLI) | `br list` |
| | git (version control) | `git status` |
| | rg (ripgrep) | `grep pattern` |
| | tailscale (identity) | `tailscale status` |
| | age (encryption) | `age decrypt` |
| | ffmpeg (audio) | `ffmpeg -i file.wav` |
| | aider (agent adapter) | `aider --model gpt-4` |
| | claude (agent adapter) | `claude prompt` |
| | codex (agent adapter) | `codex generate` |
| | gemini (agent adapter) | `gemini chat` |
| | opencode (agent adapter) | `opencode edit` |
| | gcloud (capacity) | `gcloud instances list` |
| | systemctl (service checks) | `systemctl status hoop-daemon` |
| | tmux (version checks) | `tmux -V` |
| | df (disk checks) | `df -h` |
| | curl (HTTP requests) | `curl https://api.example.com` |
| **Edge Cases (3)** | Orphaned subprocesses | PPID=1, HOOP-related |
| | Agent adapter trees | Deep process trees |
| | Hung processes | Uninterruptible, SIGKILL required |

### 2. Simple Cleanup Script

**Script:** `./bin/cleanup-hoop-test-processes.sh`

**Coverage:** Core patterns (test binaries + immediate subprocesses)

**Features:**
- Faster execution (fewer checks)
- Good for routine cleanup
- Includes automatic verification

**Usage:**

```bash
./bin/cleanup-hoop-test-processes.sh
```

### 3. Verification Script

**Script:** `./bin/verify-hoop-test-processes.sh`

**Coverage:** All 23 patterns

**Features:**
- Comprehensive process detection
- Colored pass/fail output
- Verbose mode for debugging
- Exit codes for scripting

**Usage:**

```bash
# Basic verification
./bin/verify-hoop-test-processes.sh

# Verbose mode (shows each process found)
./bin/verify-hoop-test-processes.sh --verbose
```

**Exit Codes:**
- `0` - Clean (no HOOP test processes found)
- `1` - Unclean (HOOP test processes present)
- `2` - Warning (zombie or uninterruptible processes found)

## Recommended Workflows

### Standard Test Session

```bash
# 1. Clean before starting
./bin/kill-hoop-test-processes

# 2. Run tests
nix-shell --run 'cargo test'

# 3. Clean after completion
./bin/kill-hoop-test-processes --verify

# 4. Verify clean state
./bin/verify-hoop-test-processes.sh
```

### Using Makefile (Recommended)

The Makefile includes automatic cleanup:

```bash
make test              # Unit tests with auto cleanup
make test-load         # Load tests with auto cleanup
make test-load-medium  # Medium-scale load test with auto cleanup
make test-load-full    # Full-scale load test with auto cleanup
```

### Failed Test Recovery

When tests fail or hang:

```bash
# 1. Force kill any stuck processes
./bin/kill-hoop-test-processes --force

# 2. Verify cleanup
./bin/verify-hoop-test-processes.sh --verbose

# 3. If verification fails, investigate
ps aux | grep -E 'HOOP/target|testrepo' | grep -v grep

# 4. Manually kill any survivors (if needed)
kill -9 <PID>

# 5. Re-verify
./bin/verify-hoop-test-processes.sh
```

### Integration with CI/CD

For automated test runs:

```bash
#!/bin/bash
set -e

# Pre-test cleanup
./bin/kill-hoop-test-processes --force

# Run tests (capture exit code)
TEST_EXIT=0
nix-shell --run 'cargo test' || TEST_EXIT=$?

# Post-test cleanup (always run)
./bin/kill-hoop-test-processes --force

# Verify clean state
./bin/verify-hoop-test-processes.sh

# Exit with original test result
exit $TEST_EXIT
```

### Quick Development Loop

For rapid edit-test cycles:

```bash
# Create an alias in ~/.bashrc
alias hoop-test='./bin/kill-hoop-test-processes; nix-shell --run "cargo test"; ./bin/kill-hoop-test-processes'

# Use it
hoop-test
```

## Edge Cases and Troubleshooting

### 1. Zombie Processes (Defunct)

**Detection:**
```bash
ps aux | grep -E 'hoop|testrepo' | grep -E 'Z$|<defunct>'
```

**Issue:** Dead processes not yet reaped by parent.

**Resolution:**
- Usually auto-clear when parent process exits
- If persistent, kill the parent process
- Verification script reports these with exit code 2

### 2. Uninterruptible Processes (D State)

**Detection:**
```bash
ps aux | grep -E 'hoop|testrepo' | grep -E ' D '
```

**Issue:** Process waiting for I/O (usually network or disk).

**Resolution:**
```bash
./bin/kill-hoop-test-processes --force
```
- Uses SIGKILL instead of SIGTERM
- May require system intervention if persistent
- Check for hung network connections or NFS mounts

### 3. Orphaned Processes (PPID=1)

**Detection:**
```bash
ps ao pid,ppid,comm,args | awk '$2 == 1 && ($4 ~ "HOOP" || $4 ~ "testrepo")'
```

**Issue:** Parent process died, child re-parented to init.

**Resolution:**
- Comprehensive cleanup script handles these automatically
- Manual kill by PID if needed: `kill -9 <PID>`

### 4. Interactive Process Hangs

**Detection:**
```bash
ps aux | grep -E "S\+.*git|age|claude|aider"
```

**Issue:** Process waiting for user input.

**Resolution:**
```bash
./bin/kill-hoop-test-processes --force
```
- SIGTERM may not work on interactive processes
- SIGKILL required (--force flag)

### 5. Deep Process Trees

**Detection:**
```bash
pstree -p <PID>
```

**Issue:** Agent adapters spawn children that spawn more children.

**Resolution:**
- Comprehensive cleanup script handles tree traversal
- Kill by process group if needed: `pkill -9 -g <PGID>`

### 6. Network-Connected Processes

**Detection:**
```bash
lsof -i | grep -E 'hoop|testrepo'
ss -tunap | grep -E 'hoop|testrepo'
```

**Issue:** Process hung on network connection.

**Resolution:**
```bash
./bin/kill-hoop-test-processes --force
```
- May need to kill parent process first
- Check Tailscale connectivity and firewall rules

### 7. Editor Processes (vim, nano, code)

**Detection:**
```bash
ps aux | grep -E 'vim|nano|code' | grep -E 'HOOP|testrepo'
```

**Issue:** Editor invoked by tests, waiting for user input.

**Warning:** DO NOT kill these if user is actively editing!

**Resolution:**
- Verify user is not editing before killing
- Check terminal for active editor sessions
- Use path-based filtering to limit to test directories

### 8. Script Permission Issues

**Symptom:** Permission denied when running cleanup scripts.

**Resolution:**
```bash
chmod +x bin/kill-hoop-test-processes
chmod +x bin/verify-hoop-test-processes.sh
chmod +x bin/cleanup-hoop-test-processes.sh
```

### 9. Processes Reappearing After Cleanup

**Detection:**
```bash
./bin/kill-hoop-test-processes
./bin/verify-hoop-test-processes.sh
# Still shows processes
```

**Issue:** Something is continuously spawning new processes.

**Resolution:**
- Check for cron jobs or systemd timers
- Look for background services in `hoop-daemon`
- Review test code for infinite loops or respawn logic
- Use `--force` flag and investigate source

## Manual Cleanup Patterns

If automated scripts fail, use these manual patterns:

### Quick One-Liner

```bash
pkill -f 'hoop-[a-f0-9]{16,}$' && pkill -f 'hoop_daemon-[a-f0-9]{16,}$' && pkill -f 'testrepo/(bin|scripts)/' && pkill -9 -f 'build-script-build$' || true
```

### Comprehensive Manual Cleanup

```bash
# Kill test binaries
pkill -f 'HOOP/target/debug/deps/hoop' 2>/dev/null || true

# Kill testrepo processes
pkill -f 'testrepo/(bin|scripts)/' 2>/dev/null || true

# Kill build scripts
pkill -9 -f 'build-script-build$' 2>/dev/null || true
pkill -9 -f 'target/debug/build.*build-script' 2>/dev/null || true

# Kill HOOP subprocesses (only if parent is HOOP test)
for pid in $(pgrep -f 'HOOP/target/debug/deps/hoop'); do
    pkill -P "$pid" 2>/dev/null || true
done

# Kill orphaned HOOP processes
ps ao pid,ppid,comm,args | awk -v hoop_path="$PWD/HOOP" '
    $2 == 1 && ($4 ~ hoop_path || $4 ~ "testrepo") {print $1}
' | xargs -r kill -9 2>/dev/null || true

# Verify
ps aux | grep -E 'HOOP/target|testrepo|br\s|git\s|rg\s' | grep -v grep || echo "Clean"
```

## Monitoring and Prevention

### Check for Process Accumulation

```bash
# Count HOOP-related processes
count=$(ps aux | grep -E 'HOOP/target|testrepo' | grep -v grep | wc -l)
echo "HOOP process count: $count"

# Alert if threshold exceeded
if [ "$count" -gt 10 ]; then
    echo "WARNING: High HOOP process count!"
    ./bin/kill-hoop-test-processes
fi
```

### Add to Crontab for Automated Cleanup

```bash
# Clean up every hour
0 * * * * /home/coding/HOOP/bin/kill-hoop-test-processes --force

# Check daily at 2 AM and report
0 2 * * * /home/coding/HOOP/bin/verify-hoop-test-processes.sh || mail -s "HOOP cleanup failed" admin@example.com
```

### Pre-Commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Ensure clean state before committing

if /home/coding/HOOP/bin/verify-hoop-test-processes.sh; then
    exit 0
else
    echo "ERROR: HOOP test processes present. Run: ./bin/kill-hoop-test-processes"
    exit 1
fi
```

## Safety Guarantees

The cleanup scripts are designed to be **safe** for daily use:

### Only Kills HOOP-Related Processes

- Uses hierarchical subprocess detection (checks parent PIDs)
- Path-based filtering (only processes in HOOP/testrepo paths)
- Won't kill your working git/claude/etc processes in other repos

### Idempotent

- Safe to run multiple times
- No errors if environment is already clean
- No side effects if run accidentally

### Non-Destructive to Work

- SIGTERM by default (graceful shutdown)
- SIGKILL only with --force flag
- Won't delete files, modify source, or affect git state

## Summary

**Best Practice:**
```bash
./bin/kill-hoop-test-processes --verify
```

**Before Every Test:**
```bash
./bin/kill-hoop-test-processes
```

**After Every Test:**
```bash
./bin/kill-hoop-test-processes --verify
```

**When Tests Fail:**
```bash
./bin/kill-hoop-test-processes --force
```

**Verify Clean State:**
```bash
./bin/verify-hoop-test-processes.sh
```

## Related Documentation

- `docs/test-process-cleanup-patterns.md` - Detailed pattern analysis
- `CLAUDE.md` - Repository instructions with cleanup requirements
- `bin/kill-hoop-test-processes` - Comprehensive cleanup script (annotated)
- `bin/verify-hoop-test-processes.sh` - Verification script (annotated)
