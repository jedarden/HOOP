# HOOP Test Process Cleanup Workflow

## Overview

HOOP integration tests spawn long-lived subprocesses that do **not** self-terminate on failure. Leaked processes accumulate across sessions and can cause OOM kills on the lab server. This document describes the complete cleanup workflow.

## The Problem

When HOOP tests run, they spawn processes across multiple categories:

1. **Test binaries** - `hoop-{16+ hex chars}`, `hoop_daemon-{16+ hex chars}`
2. **Subprocesses** - `br`, `git`, `rg`, `tailscale`, `age`, `ffmpeg`, agent adapters, etc.
3. **Build scripts** - `build-script-build` processes from interrupted builds
4. **Edge cases** - Orphaned processes, hung processes, deep process trees

If tests fail or are interrupted, these processes remain running and accumulate over time.

## Solution: Automated Cleanup Scripts

HOOP provides three scripts for comprehensive process management:

### 1. `bin/kill-hoop-test-processes` (Recommended)

**Purpose:** Primary cleanup script with comprehensive pattern coverage and safety features.

**Usage:**
```bash
# Basic cleanup (SIGTERM)
./bin/kill-hoop-test-processes

# Cleanup with verification
./bin/kill-hoop-test-processes --verify

# Force kill (SIGKILL) for stubborn processes
./bin/kill-hoop-test-processes --force
```

**Features:**
- Covers all 27 documented process patterns (9 primary + 14 subprocesses + 4 edge cases)
- **Safe targeting** - Only kills HOOP-related processes, not all git/claude/etc on system
- Child process detection - Finds and kills subprocesses spawned by HOOP tests
- Orphan detection - Cleans up processes whose parent (test binary) already died
- Color-coded output for easy reading
- Process count reporting

**Process Patterns Covered:**

**Primary (9 patterns):**
- HOOP test binaries: `hoop-{16+ hex chars}`, `hoop_daemon-{16+ hex chars}`
- HOOP target directory processes: `HOOP/target/debug/deps/hoop*`
- Testrepo processes: `testrepo/bin/br`, `testrepo/scripts/`
- Build scripts: `build-script-build`, `target/debug/build.*build-script`

**Subprocesses (14 patterns):**
- CLI tools: `br`, `git`, `rg`, `tailscale`, `age`, `ffmpeg`, `curl`
- Agent adapters: `aider`, `claude`, `codex`, `gemini`, `opencode`
- System tools: `gcloud`, `systemctl`, `tmux`, `df`

**Edge Cases (4 patterns):**
- Orphaned subprocesses (PPID=1, HOOP-related)
- Agent adapter deep process trees
- Hung/uninterruptible processes
- Interactive hangs (S+ state)

### 2. `bin/cleanup-hoop-test-processes.sh` (Alternative)

**Purpose:** Simpler cleanup script with automatic verification.

**Usage:**
```bash
./bin/cleanup-hoop-test-processes.sh
```

**Features:**
- Simpler logic, easier to understand
- Automatically runs verification after cleanup
- Uses `pstree` for child process detection
- Good for quick manual cleanup

### 3. `bin/verify-hoop-test-processes.sh` (Verification Only)

**Purpose:** Verify that no HOOP test processes remain.

**Usage:**
```bash
# Basic verification
./bin/verify-hoop-test-processes.sh

# Verbose mode (shows detailed process info)
./bin/verify-hoop-test-processes.sh --verbose
```

**Exit Codes:**
- `0` - No HOOP test processes found (clean)
- `1` - HOOP test processes found (unclean)
- `2` - Zombie/uninterruptible processes found (warning)

**Features:**
- Checks all 23 primary patterns
- Detects zombie processes (defunct but not reaped)
- Detects uninterruptible processes (D state)
- Detects orphaned processes (PPID=1)
- Verbose mode for detailed debugging

## Recommended Workflows

### Before Running Tests

**Option 1: Using Makefile (Recommended)**
```bash
make test              # Unit tests with auto cleanup
make test-load         # Load tests with auto cleanup
make test-load-medium  # Medium-scale load test
make test-load-full    # Full-scale load test
```

The Makefile automatically:
1. Runs cleanup before tests
2. Runs verification after tests
3. Warns if processes remain

**Option 2: Manual Cleanup**
```bash
# Clean up first
./bin/kill-hoop-test-processes

# Run tests
nix-shell --run 'cargo test'

# Verify after
./bin/verify-hoop-test-processes.sh
```

### After Tests Complete

**Via Makefile:** Verification runs automatically.

**Via cargo:** Verify manually after tests complete (pass or fail):

```bash
./bin/verify-hoop-test-processes.sh
```

For a quick check without the script:

```bash
ps aux | grep 'HOOP/target' | grep -v grep
```

Kill any survivors before finishing the session.

## When to Use Each Flag

### `--verify` Flag

**Use when:** You want confirmation that cleanup succeeded immediately.

```bash
./bin/kill-hoop-test-processes --verify
```

**What it does:**
1. Runs cleanup with SIGTERM
2. Automatically runs verification
3. Exits with verification result (0 = clean, 1 = unclean)

**Best for:** CI/CD pipelines, automated test runs

### `--force` Flag

**Use when:** Processes survived SIGTERM and need SIGKILL.

```bash
./bin/kill-hoop-test-processes --force
```

**What it does:**
1. Uses SIGKILL (-9) instead of SIGTERM
2. Force-kills all HOOP test processes
3. No graceful shutdown (processes die immediately)

**Best for:** Stubborn processes, hung processes, emergency cleanup

**Warning:** SIGKILL prevents graceful shutdown. Use only when SIGTERM fails.

### `--verbose` Flag (verify script only)

**Use when:** Debugging why processes remain or investigating cleanup issues.

```bash
./bin/verify-hoop-test-processes.sh --verbose
```

**What it does:**
- Shows full command line for each found process
- Displays PID and process details
- Helps identify which patterns are matching

**Best for:** Debugging, investigating false positives, understanding what's running

## Common Scenarios

### Scenario 1: Clean State (No Processes)

**Expected output:**
```
✓ No HOOP test processes found - already clean
```

**Action:** None needed. Safe to proceed with tests.

### Scenario 2: After Normal Test Completion

**Expected:** Cleanup script finds and kills leftover test binaries.

**Action:** Run `./bin/kill-hoop-test-processes --verify` to confirm cleanup.

### Scenario 3: Tests Hung or Interrupted

**Symptoms:**
- Test processes still running after Ctrl+C
- Verification finds processes
- Processes in S+ or D state

**Action:**
```bash
# First try SIGTERM
./bin/kill-hoop-test-processes

# If processes remain, use force
./bin/kill-hoop-test-processes --force

# Verify clean
./bin/verify-hoop-test-processes.sh
```

### Scenario 4: Suspected Process Leak

**Symptoms:**
- System slower than usual
- High memory usage
- Many old HOOP processes

**Action:**
```bash
# Check what's running
./bin/verify-hoop-test-processes.sh --verbose

# Clean up everything
./bin/kill-hoop-test-processes --force --verify

# If still issues, check for orphans manually
ps ao pid,ppid,comm,args | awk '$2 == 1 && ($4 ~ /HOOP/ || $4 ~ /testrepo/)'
```

### Scenario 5: False Positive Suspected

**Symptoms:** Cleanup script reporting processes you don't think are HOOP-related

**Action:**
```bash
# Verbose verification shows full process details
./bin/verify-hoop-test-processes.sh --verbose

# Check if processes are actually HOOP-related
ps aux | grep -E 'HOOP|testrepo' | grep -v grep
```

The scripts only kill processes that are:
- Children of HOOP test binaries, OR
- Contain HOOP/testrepo paths in command line, OR
- Are orphaned (PPID=1) but HOOP-related

Your git/claude/etc processes in other repos are safe.

## Safety Features

The cleanup scripts are designed to be **safe to run anytime**:

1. **Targeted killing** - Only kills HOOP-related processes
2. **No collateral damage** - Your git/claude/etc in other repos are safe
3. **Graceful first** - Uses SIGTERM by default, SIGKILL only with --force
4. **Verification** - Always verify after cleanup
5. **No-op if clean** - Safe to run even when no processes exist

## Integration with CI/CD

The cleanup scripts are designed for CI/CD integration:

### Example: Argo Workflow Template

```yaml
# Before running tests
- name: cleanup-before
  command: ./bin/kill-hoop-test-processes

# Run tests
- name: run-tests
  command: nix-shell --run 'cargo test'

# After tests (always verify)
- name: cleanup-after
  command: ./bin/verify-hoop-test-processes.sh
```

### Exit Code Handling

```bash
#!/bin/bash
./bin/kill-hoop-test-processes --verify
case $? in
    0) echo "✓ Cleanup successful" ;;
    1) echo "✗ Processes remain after cleanup" ; exit 1 ;;
    2) echo "⚠ Zombie/uninterruptible processes found" ; exit 2 ;;
esac
```

## Troubleshooting

### Problem: Cleanup script reports "No processes found" but `ps aux` shows HOOP processes

**Solution:** The processes might not match the expected patterns. Check manually:

```bash
# Show all HOOP-related processes
ps aux | grep -i hoop | grep -v grep

# Check if they match expected patterns
ps aux | grep -E 'hoop-[a-f0-9]{16,}|hoop_daemon-[a-f0-9]{16,}'
```

If patterns don't match, update the script with new patterns.

### Problem: Verification exits with code 2 (zombie/uninterruptible)

**Solution:** Zombie/uninterruptible processes need special handling:

```bash
# Check for zombies
ps aux | grep -E 'hoop|testrepo' | grep -E 'Z$|<defunct>'

# Check for uninterruptible (D state)
ps aux | grep -E 'hoop|testrepo' | grep -E ' D '

# Force kill may help
./bin/kill-hoop-test-processes --force

# If still issues, may need system reboot
```

### Problem: Processes keep coming back

**Solution:** Something is restarting them. Check for:

1. **Systemd services:** `systemctl list-units | grep hoop`
2. **Cron jobs:** `crontab -l`
3. **Background agents:** `ps aux | grep -E 'agent|daemon'`

### Problem: Cleanup kills processes I'm actively using

**Solution:** This shouldn't happen with the targeted scripts, but if it does:

1. Use verbose verification to identify the issue
2. Check if your working directory contains "HOOP" in the path
3. Report the false positive pattern

## Quick Reference Card

```bash
# Before tests (always)
./bin/kill-hoop-test-processes

# Run tests
make test  # or nix-shell --run 'cargo test'

# After tests (always)
./bin/verify-hoop-test-processes.sh

# If cleanup fails
./bin/kill-hoop-test-processes --force

# For debugging
./bin/verify-hoop-test-processes.sh --verbose

# Quick check
ps aux | grep 'HOOP/target' | grep -v grep
```

## Related Documentation

- `docs/test-process-cleanup-patterns.md` - Complete pattern reference with 27 documented patterns
- `CLAUDE.md` - Project instructions with cleanup requirements
- `Makefile` - Automated test targets with cleanup integration

## Summary

| Script | Purpose | When to Use |
|--------|---------|-------------|
| `kill-hoop-test-processes` | Primary cleanup | Always before tests |
| `cleanup-hoop-test-processes.sh` | Simple cleanup + verify | Quick manual cleanup |
| `verify-hoop-test-processes.sh` | Verification only | After tests, debugging |

**Golden Rule:** Always clean up before running tests, always verify after tests complete.

**Remember:** The scripts are safe to run anytime - they only kill HOOP-related processes, not your other work.
