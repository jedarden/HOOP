# TestRepo Fixture Documentation

This directory contains a synthetic Rust workspace used as a fixture for HOOP integration testing.

## Purpose

The testrepo provides a realistic file tree and pre-populated `.beads/` data for testing HOOP's integration test suite without requiring live NEEDLE workers, CLI sessions, or LLM calls.

## Structure

```
testrepo/
├── .beads/                    # Pre-populated beads workspace
│   ├── attachments/           # Example attachments (image, audio, video)
│   ├── beads.db              # SQLite database (optional, regenerated)
│   ├── issues.jsonl          # Synthetic beads in various states
│   ├── events.jsonl          # NEEDLE event stream
│   ├── heartbeats.jsonl      # Worker heartbeat stream
│   └── config.yaml           # br configuration
├── bin/                       # Stub binaries
│   └── br                     # br CLI stub that records calls
├── cli-sessions/              # Pre-recorded CLI sessions per adapter
│   ├── claude/               # Claude adapter sessions
│   ├── codex/                # Codex adapter sessions
│   ├── gemini/               # Gemini adapter sessions
│   ├── opencode/             # OpenCode adapter sessions
│   └── aider/                # Aider adapter sessions
├── scripts/                   # Fixture regeneration utilities
│   ├── regenerate-fixtures.sh       # Main regeneration script
│   ├── regenerate-attachments.py    # Regenerate attachment files
│   └── regenerate-cli-sessions.py   # Regenerate CLI sessions
├── src/                       # Synthetic Rust source code
├── tests/                     # Synthetic test files
├── docs/                      # Documentation
└── fixtures/                  # Additional test fixtures
```

## Bead States

The `issues.jsonl` contains synthetic beads in various states:

| ID | State | Purpose |
|----|-------|---------|
| tr-open-001, tr-open-002, tr-open-003 | open | Unclaimed work items |
| tr-claimed-001, tr-claimed-002, tr-claimed-003 | in_progress | Currently claimed by agents |
| tr-closed-001, tr-closed-002, tr-closed-003 | closed | Completed work |
| tr-failed-001, tr-failed-002, tr-failed-003 | open (failed) | Failed tasks |

## Attachment Types

Example attachments are provided for multimodal testing:

| Type | Location | Purpose |
|------|----------|---------|
| Image (PNG) | `.beads/attachments/tr-open-001/screenshot.png` | Screenshot for bug reports |
| Audio (WAV) | `.beads/attachments/tr-open-001/audio_message.wav` | Voice message transcription |
| Video (MP4) | `.beads/attachments/tr-open-001/demo_video.mp4` | Screen recording |
| Text log | `.beads/attachments/tr-closed-002/error_log.txt` | Error logs |
| JSON data | `.beads/attachments/tr-failed-001/metrics.json` | Performance metrics |

## CLI Session Format

CLI sessions in `cli-sessions/*/` follow the JSONL format:

```json
{"ts":"2026-04-21T18:42:10Z","cmd":"br list","output":"[needle:alpha:bd-abc123:pluck] tr-open-001|Fix memory leak|open|bug"}
```

The `[needle:<worker>:<bead>:<strand>]` prefix convention tags each output with the NEEDLE worker context.

## br Stub Binary

The `bin/br` stub is a minimal bash script that:
- Emulates `br` read verbs (`list`, `show`, `ready`, etc.) against fixture JSON
- Records write verbs (`create`, `close`, `update`, etc.) to `.stub-log.jsonl`
- Returns fixture data without requiring a real `br` installation

## Regenerating Fixtures

To regenerate all fixtures:

```bash
cd testrepo
./scripts/regenerate-fixtures.sh
```

To regenerate only attachments:

```bash
cd testrepo
python3 scripts/regenerate-attachments.py
```

To regenerate CLI sessions for a specific adapter:

```bash
cd testrepo
python3 scripts/regenerate-cli-sessions.py claude
```

## Size Constraints

The testrepo must remain under 50MB to keep the HOOP repo manageable. Current size is approximately 2.8MB.

## Integration Tests

Tests that use testrepo:

- `golden_transcripts_regression` - Validates transcript parsing
- `needle_events_roundtrip` - Tests event serialization/deserialization
- `protocol_contract` - Verifies br stub behavior

Run all integration tests:

```bash
cd /home/coding/HOOP
cargo test --test golden_transcripts_regression
cargo test --test needle_events_roundtrip
cargo test --test protocol_contract
```

## Adding New Fixtures

When adding new test scenarios:

1. Create new synthetic beads in `issues.jsonl`
2. Add corresponding events to `events.jsonl`
3. Record CLI sessions in `cli-sessions/`
4. Add attachments if needed
5. Update this documentation
6. Run `./scripts/regenerate-fixtures.sh` to rebuild

## Notes

- All timestamps are in UTC (ISO 8601 format)
- Bead IDs use the `tr-` prefix (testrepo)
- Worker names follow the alpha/bravo/charlie/delta pattern
- Session IDs in `closed_by_session` use `<worker>-<number>` format
