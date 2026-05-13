# hoop-ttb.11.1: testrepo fixture completion

## Date
2026-05-13

## Summary
Built complete testrepo/ fixture for HOOP integration testing with realistic file tree, synthetic bead data, CLI sessions, and attachments.

## What was built

### File tree structure
- **538 files** total (excluding load-test-data/)
- Synthetic Rust crate structure with:
  - `src/` - Main source code (services, storage, crypto, network, api, core, parsing, async, models, utils, cli, migrations)
  - `tests/` - Integration test files
  - `examples/` - Example configuration files
  - `docs/` - Documentation
  - `fixtures/` - Test fixtures and scenarios

### Pre-populated .beads/ data
- **12 synthetic beads** in known states:
  - 3 open: tr-open-001, tr-open-002, tr-open-003
  - 3 claimed/in_progress: tr-claimed-001, tr-claimed-002, tr-claimed-003
  - 3 closed: tr-closed-001, tr-closed-002, tr-closed-003
  - 3 failed: tr-failed-001, tr-failed-002, tr-failed-003
- **9 NEEDLE events** covering all event types (claim, dispatch, complete, fail, release, timeout, crash, close, update)
- **3 heartbeats** covering all states (idle, executing, knot)

### CLI session fixtures
- **67 total session entries** across 5 adapters:
  - Claude: 11 entries
  - Codex: 9 entries
  - Gemini: 8 entries
  - OpenCode: 7 entries
  - Aider: 8 entries
- All sessions include proper `[needle:<worker>:<bead>:<strand>]` prefixes

### Attachments
- **PNG screenshot**: `.beads/attachments/tr-open-001/screenshot.png`
- **WAV audio**: `.beads/attachments/tr-open-001/audio_message.wav`
- **MP4 video**: `.beads/attachments/tr-open-001/demo_video.mp4`
- **Text log**: `.beads/attachments/tr-closed-002/error_log.txt`
- **JSON metrics**: `.beads/attachments/tr-failed-001/metrics.json`

### br stub binary
- `bin/br` bash script that:
  - Emulates all br read verbs (list, show, ready, blocked, etc.)
  - Records write verbs to `.stub-log.jsonl`
  - Returns fixture JSON without requiring real br installation
  - Executable and properly documented

### Regeneration scripts
- `scripts/regenerate-fixtures.sh` - Main regeneration script (all fixtures)
- `scripts/regenerate-cli-sessions.py` - CLI session regeneration
- `scripts/regenerate-attachments.py` - Attachment file regeneration
- `scripts/verify-fixture.sh` - Verification script (27 checks, all passing)

### Size
- **2.9MB** total (well under 50MB limit)

## Acceptance criteria met

✓ testrepo/ committed to HOOP repo
✓ ~500 files with realistic Rust crate structure
✓ Pre-populated .beads/ with synthetic beads in known states
✓ Pre-recorded CLI session JSONL per adapter with proper needle prefixes
✓ Canned events.jsonl and heartbeats.jsonl
✓ Example attachments (image, audio, video, text, JSON)
✓ br stub binary that records calls
✓ Fixture regeneration script documented (FIXTURE.md + scripts)
✓ Size bounded (<50MB)

## Integration tests
The following tests use testrepo:
- `testrepo_integration` - Daemon boot and state projection tests
- `testrepo_harness_integration` - Integration harness tests
- `needle_events_roundtrip` - Event serialization/deserialization
- `golden_transcripts_regression` - Transcript parsing validation
- `performance_budget` - Performance budget verification
- `state_projections` - State projection consistency

## Usage
```bash
# Verify fixture integrity
cd testrepo && bash scripts/verify-fixture.sh

# Regenerate all fixtures
cd testrepo && bash scripts/regenerate-fixtures.sh

# Run integration tests against testrepo
cargo test --test testrepo_integration
cargo test --test needle_events_roundtrip
cargo test --test golden_transcripts_regression
```

## Documentation
See `testrepo/FIXTURE.md` for complete fixture documentation including:
- Directory structure
- Bead states and purposes
- Attachment types and locations
- CLI session format
- br stub binary behavior
- Regeneration procedures
- Integration test references

## Retrospective

### What worked
- The fixture structure was already well-designed from previous iterations
- Bash-based br stub proved simple and reliable
- Verification script provides excellent validation coverage
- Size is well within bounds (2.9MB vs 50MB limit)

### What didn't
- Initial OpenSSL compilation issues prevented running full integration test suite
- Had to rely on verification script instead of compiled test binaries

### Surprise
- The fixture was already largely complete from previous commits
- 67 CLI session entries across 5 adapters provides good coverage
- All 27 verification checks passed on first run

### Reusable pattern
For future fixture development:
1. Start with a verification script that validates all requirements
2. Use bash scripts for simple stubs (like br)
3. Include regeneration scripts from the beginning
4. Document fixtures in their own README/FIXTURE.md
5. Keep size in check from the start (exclude generated artifacts)
