# HOOP Test Process Cleanup Patterns

## Problem

HOOP integration tests spawn long-lived subprocesses that do **not** self-terminate on failure. Leaked processes accumulate across sessions and can cause OOM kills on the lab server.

## Current Cleanup Pattern

```bash
pkill -f 'HOOP/target/debug/deps/' 2>/dev/null || true
```

This pattern is **incomplete** - it only catches the test binaries themselves, not the subprocesses that tests spawn.

## All Process Patterns Identified

### 1. Test Binaries (Primary Pattern)

**Pattern:** `HOOP/target/debug/deps/hoop-*` and `HOOP/target/debug/deps/hoop_daemon-*`

**Examples:**
- `hoop-0964aabb985f3f32`
- `hoop_daemon-37d6544c861da05a`
- `bead_created_by_hoop_broadcast-7e467bf2276a8777`

**Source:** Cargo test compilation

**Cleanup:** `pkill -f 'HOOP/target/debug/deps/hoop'`

### 2. `br` Subprocesses

**Pattern:** Process named `br` (beads CLI)

**Source:** 
- `hoop-daemon/src/br_verbs.rs` - All br read/create operations
- Tests that call br via `Command::new("br")`
- Stub binary at `/home/coding/HOOP/testrepo/bin/br`

**Cleanup:** `pkill -f '^br\b'` (careful - word boundary to avoid matching `brew`, etc.)

### 3. Git Subprocesses

**Pattern:** Process named `git`

**Source:**
- `api_bead_files.rs` - Git file operations
- `api_blame.rs` - Git blame for code archaeology
- `api_diff.rs` - Diff generation
- `bead_commit_index.rs` - Commit indexing
- `files.rs` - Git status and log
- `net_diff.rs` - Multi-commit diff aggregation
- `stitch_reconstruction.rs` - Branch and commit queries

**Cleanup:** `pkill -f '^git\b'`

### 4. Ripgrep (rg) Subprocesses

**Pattern:** Process named `rg`

**Source:** `files.rs` - File content search

**Cleanup:** `pkill -f '^rg\b'`

### 5. Tailscale Subprocesses

**Pattern:** Process named `tailscale`

**Source:**
- `api_scripts.rs` - Identity verification for script execution
- `api_skills.rs` - Identity verification for skill execution
- `audit.rs` - Startup audit
- `auth.rs` - Tailscale identity verification
- `identity.rs` - Identity lookups

**Cleanup:** `pkill -f '^tailscale\b'`

### 6. Age (Encryption) Subprocesses

**Pattern:** Process named `age`

**Source:** `backup_pipeline.rs` - Backup encryption/decryption

**Cleanup:** `pkill -f '^age\b'`

### 7. FFmpeg Subprocesses

**Pattern:** Process named `ffmpeg`

**Source:**
- `audio_redaction.rs` - Audio muting for redaction
- `transcription.rs` - Audio format conversion for Whisper

**Cleanup:** `pkill -f '^ffmpeg\b'`

### 8. Agent Adapter Subprocesses

**Pattern:** Process names: `aider`, `claude`, `codex`, `gemini`, `opencode`

**Source:** `agent_adapter.rs` - CLI adapter spawning for human-interface agent (Phase 5+)

**Cleanup:** 
```bash
pkill -f '^aider\b'
pkill -f '^claude\b'
pkill -f '^codex\b'
pkill -f '^gemini\b'
pkill -f '^opencode\b'
```

### 9. Script and Skill Subprocesses

**Pattern:** Arbitrary executable paths

**Source:**
- `api_scripts.rs` - User-defined scripts
- `api_skills.rs` - User-defined skills
- `script_trigger.rs` - Event-triggered scripts

**Cleanup:** No reliable pattern - scripts can be named anything

### 10. GCloud Subprocesses

**Pattern:** Process named `gcloud`

**Source:** `capacity.rs` - GCloud capacity monitoring (Phase 2+)

**Cleanup:** `pkill -f '^gcloud\b'`

### 11. Systemctl Subprocesses

**Pattern:** Process named `systemctl`

**Source:** `audit.rs` - System service status checks

**Cleanup:** `pkill -f '^systemctl\b'`

### 12. Tmux Subprocesses

**Pattern:** Process named `tmux`

**Source:** `audit.rs` - Version check for NEEDLE compatibility

**Cleanup:** `pkill -f '^tmux\b'`

### 13. DF (Disk Free) Subprocesses

**Pattern:** Process named `df`

**Source:** `audit.rs` - Disk space verification

**Cleanup:** `pkill -f '^df\b'`

### 14. cURL Subprocesses

**Pattern:** Process named `curl`

**Source:** HTTP requests in various modules

**Cleanup:** `pkill -f '^curl\b'`

### 15. Interactive Editor Processes

**Pattern:** Process named after editor (vim, nano, code, etc.)

**Source:** `hoop-cli/src/new.rs` - Editor invoked via `$EDITOR` for bead editing

**Cleanup Challenge:** Cannot reliably kill - user may be actively editing. Only kill if clearly associated with test processes (use path-based filtering).

**Detection:** Path-based filtering to limit to HOOP test directories only.

### 16. Whisper CLI Processes

**Pattern:** Variable path to whisper CLI binary

**Source:** `hoop-daemon/src/transcription.rs` - Audio transcription

**Cleanup Challenge:** Dynamic path - depends on configuration. Use path-based filtering to identify HOOP-related instances.

### 17. Build Script Processes

**Pattern:** `target/debug/build/*/build-script-build` and `*_build_script_*`

**Source:** Cargo build scripts (can be left running if interrupted)

**Cleanup:** `pkill -f 'target/debug/build.*build-script'`

## Edge Cases and Variations

### 1. Long-Lived Test Daemons

**Issue:** Integration tests spawn daemons via `tokio::spawn` in `integration_harness.rs`. These are meant to shut down when the `TestDaemon` handle is dropped, but:

- If test panics or is SIGKILLed, the `Drop` impl may not run
- Daemon runs as a tokio task inside the test binary process, not a separate process
- The daemon task may outlive the test if not properly aborted

**Pattern:** The test binary itself stays running

**Cleanup:** Same as primary test binary pattern

### 2. Agent Adapter Session Persistence

**Issue:** Agent adapters (claude, aider, etc.) spawned by tests may have their own subprocess management:

- They spawn child processes for tool execution
- They may persist sessions across multiple invocations
- Process trees can be deep (agent → tool → subprocess)

**Pattern:** Agent processes can have children that aren't directly named in the test code

**Cleanup:** `pkill -9` to kill entire process group, or use `pkill -g` to kill by process group

### 3. Interactive Prompts

**Issue:** Subprocesses that prompt for input will block indefinitely:

- `git` asking for credentials
- `age` asking for passphrase
- Agent adapters waiting for interactive input

**Pattern:** Process hangs in `S+` (interruptible sleep) state

**Detection:** `ps aux | grep -E "S\+.*git|age|claude|aider"`

**Cleanup:** Must use `SIGKILL` (`kill -9` or `pkill -9`)

### 4. Testrepo Stub Binaries

**Issue:** The testrepo has its own stub binaries that may be invoked:

- `/home/coding/HOOP/testrepo/bin/br` - Mock br for testing
- Scripts in `/home/coding/HOOP/testrepo/scripts/`

**Pattern:** Processes running from testrepo path

**Cleanup:** `pkill -f 'testrepo/(bin|scripts)/'`

### 5. Network-Connected Processes

**Issue:** Some subprocesses make network connections and may hang on:

- Tailscale API calls
- HTTP requests via reqwest (in separate tokio tasks)
- Websocket connections (WS fan-out to slow/stuck clients)

**Pattern:** Process in `D` (uninterruptible sleep) or `S` state with established network connections

**Detection:** `lsof -i` or `ss -tunap`

**Cleanup:** `pkill -9` for hung processes, may need to kill the parent test binary

### 6. Orphaned Process Groups

**Issue:** If the parent test process dies but children were spawned with `process::Command` without proper cleanup:

- Children get re-parented to init (PID 1)
- They're no longer associated with the test in `pstree`
- They continue running until system reboot or manual intervention

**Pattern:** Process with PPID = 1 running from a HOOP-related path

**Detection:** `ps ao pid,ppid,comm | awk '$2 == 1 && $3 ~ /br|git|rg|ffmpeg|age/'`

**Cleanup:** Must kill by PID directly, pattern matching won't catch PPID

## Recommended Comprehensive Cleanup

```bash
# Before running tests
pkill -f 'HOOP/target/debug/deps/hoop' 2>/dev/null || true
pkill -f 'testrepo/(bin|scripts)/' 2>/dev/null || true
pkill -9 -f '^(br|git|rg|tailscale|age|ffmpeg|aider|claude|codex|gemini|opencode|gcloud|systemctl|tmux|df|curl)\s' 2>/dev/null || true
pkill -9 -f 'target/debug/build.*build-script' 2>/dev/null || true

# After tests complete (verification)
ps aux | grep -E 'HOOP/target|testrepo|br\s|git\s|rg\s|tailscale\s|age\s|ffmpeg\s|curl\s' | grep -v grep || echo "No HOOP test processes found"
```

## Automation

Add to `~/.bashrc` or HOOP's test script:

```bash
cleanup_hoop_test_processes() {
    echo "Cleaning up HOOP test processes..."
    pkill -f 'HOOP/target/debug/deps/hoop' 2>/dev/null && echo "Killed test binaries"
    pkill -f 'testrepo/(bin|scripts)/' 2>/dev/null && echo "Killed testrepo processes"
    pkill -9 -f '^(br|git|rg|tailscale|age|ffmpeg|curl)\s' 2>/dev/null && echo "Killed subprocesses"
    pkill -9 -f 'target/debug/build.*build-script' 2>/dev/null && echo "Killed build scripts"
}

# Run before tests
alias hoop-test='cleanup_hoop_test_processes; nix-shell --run "cargo test"'
```

## Edge Case Summary

| Edge Case | Pattern | Detection | Cleanup |
|-----------|---------|-----------|---------|
| Long-lived daemons | Test binary won't exit | `ps aux \| grep hoop-.*` | `pkill -f 'target/debug/deps/hoop'` |
| Agent adapter trees | Agent + children | `pstree -p \`pidof claude\`` | `pkill -9 -g` (process group) |
| Interactive hangs | Process in `S+` state | `ps aux \| grep S+` | `kill -9` required |
| Orphaned groups | PPID = 1 | `ps ao pid,ppid,comm` | Direct PID kill |
| Network hangs | Process in `D` state | `ps aux \| grep D` | `kill -9` required |
| Build scripts | `build-script-build` | `ps aux \| grep build-script` | `pkill -f 'build-script'` |
| Editor processes | vim/nano/code from tests | Path-based filtering | Use path-based filtering only |
| Whisper CLI | Dynamic binary path | Path-based filtering | Use path-based filtering only |
| cURL processes | HTTP request hangs | `ps aux \| grep curl` | `pkill -f '^curl\b'` |

## Verification

After cleanup, verify no processes remain using the dedicated verification script:

```bash
./bin/verify-hoop-test-processes.sh
```

The verification script checks all process patterns comprehensively:

- **HOOP test binaries**: `hoop-*`, `hoop_daemon-*`, `HOOP/target/debug/deps`
- **Testrepo processes**: `testrepo/bin/br`, `testrepo/scripts/`
- **Build scripts**: `build-script-build`
- **Subprocesses**: `br`, `git`, `rg`, `tailscale`, `age`, `ffmpeg`, `aider`, `claude`, `codex`, `gemini`, `opencode`, `gcloud`, `systemctl`, `df`, `curl`
- **Dynamic paths**: Editor processes (via $EDITOR), Whisper CLI (configuration-dependent)
- **Edge cases**: Zombie processes, uninterruptible (D state), orphaned processes (PPID=1)

**Total Pattern Count:** 17 primary patterns + 6 edge cases = 23 distinct process patterns tracked

### Exit Codes

- `0` - No HOOP test processes found (clean)
- `1` - HOOP test processes found (unclean)
- `2` - Zombie/uninterruptible processes found (warning)

### Verbose Mode

For detailed output showing each process found:

```bash
./bin/verify-hoop-test-processes.sh --verbose
```

### Manual verification (one-liner)

For quick checks without the script:

```bash
ps aux | grep -E 'HOOP/target|testrepo|br\s|git\s|rg\s|tailscale\s|age\s|ffmpeg\s|aider\s|claude\s|codex\s|gemini\s|opencode\s|curl\s' | grep -v grep
```

Expected output: (empty - no processes found)

If processes are found, run the comprehensive cleanup again and investigate why they weren't caught.

## Stdout/Stderr Capture in Log Files

### Current Implementation

The `bin/run-with-log.sh` wrapper script captures both stdout and stderr to a single log file:

```bash
"$@" > >(tee -a "$LOG_FILE" > "$CAPTURED_STDOUT") 2> >(tee -a "$LOG_FILE" > "$CAPTURED_STDERR")
```

### Verification Results

As of 2026-08-02, stdout/stderr capture has been verified (see `docs/stdout-stderr-capture-verification.md`):

✓ **Both streams are captured** - No output is lost
✓ **Log files are created** - With ISO 8601 timestamps for uniqueness
✓ **Separate capture variables** - `HOOP_CAPTURED_STDOUT` and `HOOP_CAPTURED_STDERR` exported for shell inspection
⚠ **Streams NOT distinguishable in log** - Both streams are interleaved without markers

### Known Limitation

Because both stdout and stderr are appended to the same file without stream markers:
- Lines from stdout and stderr are interleaved
- No way to tell from the log file which stream a line came from
- Potential for mid-line collisions when both streams write simultaneously

Example from actual log output:
```
253:STDERR_COUNT_STDOUT_SEQ_0    <-- Collision: stderr+stdout in same line
256:STDERR_SEQ_0
```

### Acceptable For Current Use Cases

For most HOOP testing scenarios (load tests, integration tests, general output capture), the current implementation is sufficient because:
- All output is captured without data loss
- The log preserves the sequence of events
- Human review can usually infer the stream from content

### If Stream Distinction Is Needed

To make streams distinguishable in log files, modify `run-with-log.sh` to prefix stderr:

```bash
"$@" > >(tee -a "$LOG_FILE") 2> >(sed 's/^/STDERR: /' | tee -a "$LOG_FILE")
```

Or use separate log files:
```bash
"$@" > "logs/${LOG_BASE}_stdout.log" 2> "logs/${LOG_BASE}_stderr.log"
```

See `docs/stdout-stderr-capture-verification.md` for detailed test results and recommendations.
