# HOOP Cleanup - Practical Examples and Test Scenarios

This document provides practical examples and test scenarios for the HOOP cleanup workflow.

## Basic Usage Examples

### Example 1: Daily Development Workflow

```bash
# Morning: Start fresh
cd ~/HOOP
./bin/kill-hoop-test-processes --verify

# Work on code...
vim src/some_file.rs

# Before lunch: Run tests
./bin/kill-hoop-test-processes
nix-shell --run 'cargo test'
./bin/kill-hoop-test-processes --verify

# End of day: Final cleanup
./bin/kill-hoop-test-processes --verify
./bin/verify-hoop-test-processes.sh
```

### Example 2: Test Session with Failure Recovery

```bash
# Clean start
./bin/kill-hoop-test-processes

# Run specific test
nix-shell --run 'cargo test test_foo_bar'

# Test hangs! Ctrl+C doesn't work

# Force cleanup
./bin/kill-hoop-test-processes --force

# Verify clean
./bin/verify-hoop-test-processes.sh --verbose

# Retry test
./bin/kill-hoop-test-processes
nix-shell --run 'cargo test test_foo_bar'
./bin/kill-hoop-test-processes --verify
```

### Example 3: Using Makefile (Recommended)

```bash
# All cleanup handled automatically
make test              # Unit tests
make test-load         # Load tests
make test-load-medium  # Medium-scale tests
make test-load-full    # Full-scale tests

# Makefile output:
# → Cleaning up HOOP test processes...
# → Running tests...
# → Cleaning up after tests...
# → Verifying clean state...
# ✓ All checks passed
```

## Test Scenarios

### Scenario 1: First-Time Setup

**Goal:** Verify cleanup tools are installed and working.

```bash
# 1. Verify scripts are executable
ls -l bin/kill-hoop-test-processes
ls -l bin/verify-hoop-test-processes.sh
ls -l bin/cleanup-hoop-test-processes.sh

# 2. Fix permissions if needed
chmod +x bin/kill-hoop-test-processes
chmod +x bin/verify-hoop-test-processes.sh
chmod +x bin/cleanup-hoop-test-processes.sh

# 3. Test cleanup on clean environment
./bin/kill-hoop-test-processes --verify

# Expected output:
# ✓ No HOOP test processes found - already clean
# ✓ Verification passed

# 4. Test verification script
./bin/verify-hoop-test-processes.sh

# Expected output:
# ✓ VERIFICATION PASSED: No HOOP test processes found
```

### Scenario 2: Simulating Process Leaks

**Goal:** Create controlled process leaks and test cleanup.

```bash
# 1. Start a long-running test in background
nix-shell --run 'cargo test test_long_running -- --ignored' &
TEST_PID=$!

# 2. Wait a bit for processes to spawn
sleep 5

# 3. Check what's running
ps aux | grep -E 'HOOP/target|testrepo' | grep -v grep

# 4. Kill the test parent
kill -9 $TEST_PID 2>/dev/null || true

# 5. Check for orphans
ps ao pid,ppid,comm,args | awk -v hoop_path="$PWD/HOOP" '
    $2 == 1 && ($4 ~ hoop_path || $4 ~ "testrepo") {print $0}
'

# 6. Clean up
./bin/kill-hoop-test-processes --force

# 7. Verify
./bin/verify-hoop-test-processes.sh
```

### Scenario 3: Load Test Cleanup

**Goal:** Test cleanup after high-load test runs.

```bash
# 1. Clean start
./bin/kill-hoop-test-processes --force

# 2. Run load test
make test-load-medium

# 3. If test fails, force cleanup
./bin/kill-hoop-test-processes --force

# 4. Verify with verbose output
./bin/verify-hoop-test-processes.sh --verbose

# 5. Check system resources
df -h                     # Disk space
free -h                   # Memory
uptime                    # Load average

# 6. If resources are high, investigate
ps aux --sort=-%mem | head -20
ps aux --sort=-%cpu | head -20
```

### Scenario 4: CI/CD Integration

**Goal:** Integrate cleanup into automated workflows.

```bash
#!/bin/bash
# File: scripts/ci-test.sh

set -e

echo "=== HOOP CI Test Runner ==="

# Pre-test cleanup
echo "Cleaning up before tests..."
./bin/kill-hoop-test-processes --force

# Verify clean state
echo "Verifying clean state..."
if ! ./bin/verify-hoop-test-processes.sh; then
    echo "ERROR: Environment not clean before tests!"
    exit 1
fi

# Run tests
echo "Running tests..."
if nix-shell --run 'cargo test --locked'; then
    TEST_RESULT=0
    echo "✓ Tests passed"
else
    TEST_RESULT=$?
    echo "✗ Tests failed with code $TEST_RESULT"
fi

# Post-test cleanup
echo "Cleaning up after tests..."
./bin/kill-hoop-test-processes --force

# Verify clean state
echo "Verifying clean state after tests..."
if ! ./bin/verify-hoop-test-processes.sh; then
    echo "ERROR: Environment not clean after tests!"
    ./bin/verify-hoop-test-processes.sh --verbose
    exit 1
fi

# Exit with test result
exit $TEST_RESULT
```

### Scenario 5: Interactive Development

**Goal:** Cleanup workflow for interactive development sessions.

```bash
# Add to ~/.bashrc
hoop-test() {
    (
        cd ~/HOOP
        echo "Cleaning up..."
        ./bin/kill-hoop-test-processes

        echo "Running tests..."
        nix-shell --run "cargo test $@"

        echo "Cleaning up..."
        ./bin/kill-hoop-test-processes --verify

        echo "Verifying..."
        ./bin/verify-hoop-test-processes.sh
    )
}

# Usage:
# hoop-test                          # All tests
# hoop-test test_foo                # Specific test
# hoop-test test_foo -- --ignored   # Ignored tests
```

### Scenario 6: Debugging Cleanup Issues

**Goal:** Diagnose and fix cleanup problems.

```bash
# 1. Run cleanup
./bin/kill-hoop-test-processes

# 2. Verify (expecting failure)
./bin/verify-hoop-test-processes.sh

# 3. If verification fails, get details
./bin/verify-hoop-test-processes.sh --verbose

# 4. Check specific patterns
ps aux | grep -E 'HOOP/target' | grep -v grep
ps aux | grep -E 'testrepo' | grep -v grep
ps aux | grep -E 'br\s|git\s|rg\s' | grep -v grep

# 5. Check process trees
pstree -p $(pgrep -f 'HOOP/target' | head -1)

# 6. Check for orphans
ps ao pid,ppid,comm,args | awk '$2 == 1'

# 7. Manual cleanup if needed
pkill -9 -f 'HOOP/target'

# 8. Re-verify
./bin/verify-hoop-test-processes.sh
```

### Scenario 7: Monitoring Process Accumulation

**Goal:** Track process count over time.

```bash
#!/bin/bash
# File: scripts/monitor-hoop-processes.sh

LOG_FILE="$HOME/HOOP/process-count.log"

while true; do
    COUNT=$(ps aux | grep -E 'HOOP/target|testrepo' | grep -v grep | wc -l)
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    echo "$TIMESTAMP: $count HOOP processes" >> "$LOG_FILE"

    if [ "$COUNT" -gt 5 ]; then
        echo "WARNING: High process count ($COUNT)" | tee -a "$LOG_FILE"
        ./bin/kill-hoop-test-processes
    fi

    sleep 300  # Check every 5 minutes
done
```

## Advanced Examples

### Example 4: Parallel Test Execution

```bash
# Run multiple test suites in parallel
for suite in unit integration load; do
    (
        ./bin/kill-hoop-test-processes
        nix-shell --run "cargo test $suite"
        ./bin/kill-hoop-test-processes --verify
    ) &
done

# Wait for all to complete
wait

# Final cleanup
./bin/kill-hoop-test-processes --force
./bin/verify-hoop-test-processes.sh
```

### Example 5: Automated Cleanup on Schedule

```bash
# Add to crontab (crontab -e)

# Clean every hour
0 * * * * cd ~/HOOP && ./bin/kill-hoop-test-processes --force

# Check and report daily
0 2 * * * cd ~/HOOP && ./bin/verify-hoop-test-processes.sh || mail -s "HOOP cleanup failed" admin@example.com
```

### Example 6: Git Integration

```bash
# Pre-commit hook: .git/hooks/pre-commit
#!/bin/bash

cd ~/HOOP

if ! ./bin/verify-hoop-test-processes.sh; then
    cat <<\EOF
ERROR: HOOP test processes present. Commit blocked.

Run: ./bin/kill-hoop-test-processes

Or bypass with: git commit --no-verify
EOF
    exit 1
fi

exit 0
```

## Common Workflows

### Workflow 1: Test-Driven Development

```bash
# Write test
vim tests/test_new_feature.rs

# Run test (auto cleanup via Makefile)
make test test_new_feature

# Write implementation
vim src/new_feature.rs

# Run test again
make test test_new_feature

# Commit (pre-commit hook verifies clean state)
git add .
git commit -m "Add new feature"
```

### Workflow 2: Debugging Failing Tests

```bash
# Test fails
make test

# Force cleanup
./bin/kill-hoop-test-processes --force

# Run with verbose output
./bin/kill-hoop-test-processes
nix-shell --run 'cargo test test_failing -- --nocapture --show-output'

# Debug...
vim src/failing_code.rs

# Retry
./bin/kill-hoop-test-processes
nix-shell --run 'cargo test test_failing'
./bin/kill-hoop-test-processes --verify
```

### Workflow 3: Load Testing

```bash
# Baseline
./bin/kill-hoop-test-processes --verify
make test-load-medium

# Scale up
./bin/kill-hoop-test-processes --force
make test-load-full

# Monitor resources
htop                          # Watch CPU/memory
df -h                         # Watch disk space

# Cleanup and verify
./bin/kill-hoop-test-processes --force
./bin/verify-hoop-test-processes.sh --verbose
```

## Troubleshooting Examples

### Example 7: Processes Won't Die

```bash
# Run force cleanup
./bin/kill-hoop-test-processes --force

# Still showing processes?
./bin/verify-hoop-test-processes.sh --verbose

# Manual investigation
ps aux | grep -E 'HOOP|testrepo' | grep -v grep

# Kill by PID if needed
ps aux | grep 'HOOP/target' | grep -v grep | awk '{print $2}' | xargs -r kill -9

# Check for system-level issues
lsof | grep HOOP        # Open files
systemd-cgls           # Control groups

# Final verification
./bin/verify-hoop-test-processes.sh
```

### Example 8: Cleanup Script Fails

```bash
# Script fails with permission error
./bin/kill-hoop-test-processes
# bash: ./bin/kill-hoop-test-processes: Permission denied

# Fix permissions
chmod +x bin/kill-hoop-test-processes
chmod +x bin/verify-hoop-test-processes.sh
chmod +x bin/cleanup-hoop-test-processes.sh

# Try again
./bin/kill-hoop-test-processes
```

### Example 9: High Memory Usage

```bash
# Check memory
free -h

# Find HOOP processes using memory
ps aux --sort=-%mem | grep -E 'HOOP|testrepo' | head -10

# Force cleanup
./bin/kill-hoop-test-processes --force

# Verify
./bin/verify-hoop-test-processes.sh
free -h
```

## Testing Cleanup Tools

### Test 1: Verify Clean Environment

```bash
#!/bin/bash
# Test: Clean environment verification

echo "Test: Clean environment"

if ./bin/verify-hoop-test-processes.sh; then
    echo "✓ PASS: Environment is clean"
    exit 0
else
    echo "✗ FAIL: Environment has processes"
    ./bin/verify-hoop-test-processes.sh --verbose
    exit 1
fi
```

### Test 2: Cleanup and Verify

```bash
#!/bin/bash
# Test: Cleanup and verify

echo "Test: Cleanup and verify"

# Cleanup
./bin/kill-hoop-test-processes

# Verify
if ./bin/verify-hoop-test-processes.sh; then
    echo "✓ PASS: Cleanup successful"
    exit 0
else
    echo "✗ FAIL: Cleanup left processes"
    exit 1
fi
```

### Test 3: Force Cleanup

```bash
#!/bin/bash
# Test: Force cleanup

echo "Test: Force cleanup with verification"

# Force cleanup
./bin/kill-hoop-test-processes --force

# Verify
if ./bin/verify-hoop-test-processes.sh; then
    echo "✓ PASS: Force cleanup successful"
    exit 0
else
    echo "✗ FAIL: Force cleanup left processes"
    exit 1
fi
```

## Quick Reference

```bash
# Standard cleanup
./bin/kill-hoop-test-processes

# Cleanup with verification
./bin/kill-hoop-test-processes --verify

# Force cleanup
./bin/kill-hoop-test-processes --force

# Verify only
./bin/verify-hoop-test-processes.sh

# Verbose verification
./bin/verify-hoop-test-processes.sh --verbose

# Using Makefile
make test
make test-load
make test-load-medium
make test-load-full

# Quick one-liner
pkill -f 'HOOP/target' && pkill -f 'testrepo' && pkill -9 -f 'build-script' || true
```

## Best Practices

1. **Always clean before tests** - Ensures reproducible results
2. **Always clean after tests** - Prevents process accumulation
3. **Use Makefile targets** - Automatic cleanup built in
4. **Verify clean state** - Catch issues early
5. **Use force mode sparingly** - Only when tests fail or hang
6. **Monitor system resources** - Disk space, memory, CPU
7. **Check for orphans** - Processes with PPID=1
8. **Update pre-commit hooks** - Enforce clean commits
9. **Log cleanup runs** - Track process accumulation patterns
10. **Test cleanup tools** - Verify scripts work before relying on them

## Related Documentation

- `docs/cleanup-workflow-guide.md` - Comprehensive workflow guide
- `docs/test-process-cleanup-patterns.md` - Detailed pattern analysis
- `CLAUDE.md` - Repository instructions
