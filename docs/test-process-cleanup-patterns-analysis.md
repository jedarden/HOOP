# HOOP Test Process Cleanup Patterns — Analysis Summary

## Task: Research and document test process patterns

**Date:** 2026-07-04  
**Bead:** bf-5lgbl  
**Status:** COMPLETE

## Executive Summary

The HOOP repository has comprehensive test process cleanup patterns already documented and implemented. Three cleanup scripts are available in the `bin/` directory, with detailed documentation covering 17 primary process patterns and 6 edge cases.

## Existing Cleanup Infrastructure

### Scripts Available

1. **`bin/cleanup-hoop-test-processes.sh`** (4803 bytes)
   - Comprehensive cleanup with subprocess tracking
   - Kills parent processes first, then orphaned children
   - Uses pstree to find child processes
   - Integrated with verification script
   - Counts and reports killed processes

2. **`bin/kill-hoop-test-processes`** (1773 bytes)
   - Quick one-liner style cleanup
   - Kills by pattern match (pkill)
   - Optional verification with `--verify` flag
   - Lightweight for manual cleanup

3. **`bin/verify-hoop-test-processes.sh`** (10967 bytes)
   - Comprehensive verification of all patterns
   - Exit codes: 0 (clean), 1 (unclean), 2 (zombie/uninterruptible)
   - Verbose mode available with `--verbose`
   - Checks for edge cases (zombies, D state, orphans)

### Documentation

**`docs/test-process-cleanup-patterns.md`**
- Comprehensive guide covering all 17 primary patterns
- Edge cases and detection strategies
- Cleanup commands for each pattern
- Automation examples
- Total: 23 distinct process patterns tracked

## Process Patterns Catalog

### Primary Patterns (17)

| # | Pattern | Example | Source | Cleanup Command |
|---|---------|---------|--------|-----------------|
| 1 | `hoop-*` test binaries | `hoop-0964aabb985f3f32` | Cargo test compilation | `pkill -f 'HOOP/target/debug/deps/hoop'` |
| 2 | `hoop_daemon-*` test binaries | `hoop_daemon-37d6544c861da05a` | Cargo test compilation | `pkill -f 'hoop_daemon-[a-f0-9]\{16,\}$'` |
| 3 | `br` subprocess | `br` | `hoop-daemon/src/br_verbs.rs` | `pkill -f '^br\b'` |
| 4 | `git` subprocess | `git` | Multiple modules | `pkill -f '^git\b'` |
| 5 | `rg` (ripgrep) | `rg` | `files.rs` search | `pkill -f '^rg\b'` |
| 6 | `tailscale` | `tailscale` | Identity checks | `pkill -f '^tailscale\b'` |
| 7 | `age` (encryption) | `age` | Backup pipeline | `pkill -f '^age\b'` |
| 8 | `ffmpeg` | `ffmpeg` | Audio redaction/transcription | `pkill -f '^ffmpeg\b'` |
| 9 | `aider` (agent) | `aider` | Agent adapter | `pkill -f '^aider\b'` |
| 10 | `claude` (agent) | `claude` | Agent adapter | `pkill -f '^claude\b'` |
| 11 | `codex` (agent) | `codex` | Agent adapter | `pkill -f '^codex\b'` |
| 12 | `gemini` (agent) | `gemini` | Agent adapter | `pkill -f '^gemini\b'` |
| 13 | `opencode` (agent) | `opencode` | Agent adapter | `pkill -f '^opencode\b'` |
| 14 | `gcloud` | `gcloud` | Capacity monitoring | `pkill -f '^gcloud\b'` |
| 15 | `systemctl` | `systemctl` | System service checks | `pkill -f '^systemctl\b'` |
| 16 | `tmux` | `tmux` | Version checks | `pkill -f '^tmux\b'` |
| 17 | `df` (disk free) | `df` | Disk space checks | `pkill -f '^df\b'` |

### Edge Cases (6)

| # | Edge Case | Pattern | Detection | Cleanup |
|---|-----------|---------|-----------|---------|
| 1 | Long-lived daemons | Test binary won't exit | `ps aux \| grep hoop-.*` | `pkill -f 'target/debug/deps/hoop'` |
| 2 | Agent adapter trees | Agent + children | `pstree -p` | `pkill -9 -g` (process group) |
| 3 | Interactive hangs | Process in `S+` state | `ps aux \| grep S+` | `kill -9` required |
| 4 | Orphaned groups | PPID = 1 | `ps ao pid,ppid,comm` | Direct PID kill |
| 5 | Network hangs | Process in `D` state | `ps aux \| grep D` | `kill -9` required |
| 6 | Build scripts | `build-script-build` | `ps aux \| grep build-script` | `pkill -f 'build-script'` |

### Additional Dynamic Patterns (Edge Cases)

| Pattern | Source | Challenge |
|---------|--------|-----------|
| Editor processes (vim, nano, code) | `$EDITOR` environment variable | User may be actively editing |
| Whisper CLI | Configuration-dependent path | Dynamic binary path |
| Script processes | User-defined scripts | Arbitrary executable paths |
| Skill processes | User-defined skills | Arbitrary executable paths |

## Verification Against Codebase

### Cross-Reference: Codebase vs Documentation

**Code analysis of Command::new() calls:**

**Main codebase (hoop-daemon/src, hoop-cli/src, hoop-mcp/src):**
- ✅ All documented binaries found in code
- No missing patterns

**Test code (hoop-daemon/tests):**
- Additional binaries: `age-keygen`, `cargo`, `jq`, `which`
- These are test-specific and short-lived
- Not considered problematic for lingering processes

**Dynamic binaries (configuration-dependent):**
- `editor` (via `$EDITOR`) — documented as edge case
- `whisper_cli` (config path) — documented as edge case
- `script_path` (user scripts) — documented as edge case
- `skill.run_path` (user skills) — documented as edge case

### Testrepo Stub Patterns

The `testrepo/` directory contains mock binaries for testing:
- `/home/coding/HOOP/testrepo/bin/br` — Mock br CLI
- `/home/coding/HOOP/testrepo/scripts/` — Test scripts

**Cleanup pattern:** `pkill -f 'testrepo/(bin|scripts)/'`

## Makefile Integration

The Makefile already integrates cleanup scripts:

```bash
make test              # Unit tests with auto cleanup
make test-load         # Load tests with auto cleanup
make test-load-medium  # Medium-scale load test with auto cleanup
make test-load-full    # Full-scale load test with auto cleanup
```

Each target:
1. Runs `cleanup-hoop-test-processes.sh` before tests
2. Runs the test suite
3. Runs `verify-hoop-test-processes.sh` after tests

## Recommended Usage

### Before Running Tests (Manual)

```bash
# Option 1: Quick one-liner (from CLAUDE.md)
pkill -f 'hoop-[a-f0-9]{16,}$' && \
pkill -f 'hoop_daemon-[a-f0-9]{16,}$' && \
pkill -f 'testrepo/(bin|scripts)/' && \
pkill -9 -f 'build-script-build$' || true

# Option 2: Comprehensive script
./bin/cleanup-hoop-test-processes.sh

# Option 3: Simple script
./bin/kill-hoop-test-processes
```

### After Running Tests (Verification)

```bash
# Verify no processes remain
./bin/verify-hoop-test-processes.sh

# Verbose mode (shows PIDs and commands)
./bin/verify-hoop-test-processes.sh --verbose
```

### Via Makefile (Recommended)

```bash
make test              # Handles cleanup automatically
make test-load         # Handles cleanup automatically
```

## Pattern Count Summary

| Category | Count |
|----------|-------|
| Primary patterns (static binaries) | 17 |
| Edge cases (special scenarios) | 6 |
| Dynamic patterns (config-dependent) | 4 |
| Testrepo stub patterns | 2 |
| **Total distinct patterns tracked** | **29** |

## Key Findings

1. **Documentation is comprehensive** — All static binary patterns found in the codebase are documented
2. **Edge cases are well-covered** — Orphans, zombies, D state, interactive hangs all documented
3. **Dynamic patterns require path-based filtering** — Editor, Whisper, scripts, skills use path-based detection
4. **Test-specific binaries are short-lived** — `cargo`, `jq`, `which`, `age-keygen` don't linger
5. **Cleanup is automated** — Makefile integrates cleanup before/after tests
6. **Verification is thorough** — Exit codes allow automation (0 = clean, 1 = unclean, 2 = warning)

## No Action Required

The existing cleanup infrastructure is complete and comprehensive. All process patterns that can linger are documented and have cleanup strategies. The three scripts (cleanup, kill, verify) provide flexible options for manual and automated use.

## Recommendations

1. **Continue using Makefile targets** — `make test` and `make test-load` already handle cleanup
2. **Use verification script after manual tests** — Catch lingering processes before they accumulate
3. **No new patterns identified** — Current documentation covers all found patterns
4. **Documentation is accurate** — Scripts match documentation perfectly

---

**Research completed 2026-07-04**  
**All 29 process patterns identified and documented**  
**Ready for implementation (already implemented)**
