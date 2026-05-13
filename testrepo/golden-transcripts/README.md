# Golden Transcripts Fixture

This directory contains golden transcripts for testing agent adapter parsers.

## Purpose

Golden transcripts are canonical CLI session recordings that validate parser behavior across different LLM adapters (Claude, Codex, OpenCode, Gemini, Aider).

## Directory Structure

```
golden-transcripts/
├── README.md
├── claude/
│   └── v1.0/
│       ├── simple/
│       ├── tool_heavy/
│       └── failure/
├── codex/
│   └── v1.0/
│       ├── simple/
│       ├── tool_heavy/
│       └── failure/
├── opencode/
│   └── v1.0/
│       ├── simple/
│       ├── tool_heavy/
│       └── failure/
├── gemini/
│   └── v1.0/
│       ├── simple/
│       ├── tool_heavy/
│       └── failure/
└── aider/
    └── v1.0/
        ├── simple/
        ├── tool_heavy/
        └── failure/
```

## Scenarios

### simple
Basic single-turn interactions: list, show, claim. Tests fundamental parsing without complexity.

### tool_heavy
Multi-turn sessions with many tool calls: searches, file reads, updates. Tests parser performance and state management.

### failure
Error conditions and edge cases: timeouts, crashes, malformed output. Tests error handling and recovery.

## Format

Each `.jsonl` file contains one JSON object per line:

```json
{"ts":"2026-04-21T18:42:10Z","cmd":"br list","output":"[needle:alpha:bd-abc123:pluck] tr-open-001|Fix memory leak|open|bug"}
```

Fields:
- `ts`: ISO 8601 timestamp in UTC
- `cmd`: The CLI command that was executed
- `output`: The command output, including the `[needle:<worker>:<bead>:<strand>]` prefix

## Needle Prefix Convention

All outputs include the `[needle:<worker>:<bead>:<strand>]` prefix to tag the NEEDLE worker context:
- `worker`: alpha, bravo, charlie, delta, echo (NEEDLE worker ID)
- `bead`: bd-<alphanumeric> (bead ID)
- `strand`: pluck, mend, explore, weave, graft (operation type)

## Size Constraints

The entire golden-transcripts directory must remain under 10MB to keep the HOOP repo manageable.

## Integration Tests

Tests that use golden transcripts:

- `golden_transcripts_regression` — Validates transcript parsing
- `adapter_parser_contract` — Verifies adapter-specific behavior

Run all integration tests:

```bash
cd /home/coding/HOOP
cargo test --test golden_transcripts_regression
cargo test --test adapter_parser_contract
```

## Adding New Transcripts

When adding new test scenarios:

1. Create new `.jsonl` files in the appropriate adapter/version/scenario directory
2. Follow the JSONL format exactly
3. Include the `[needle:...]` prefix in all outputs
4. Use realistic timestamps and commands
5. Update this README if new scenarios or adapters are added
6. Run `./scripts/regenerate-cli-sessions.py` to rebuild fixtures

## Versioning

The `v1.0` directory represents the first version of the golden transcripts. When the transcript format changes significantly:

1. Create a new `v2.0` directory
2. Copy and update the scenario directories
3. Update integration tests to reference the new version
4. Keep old versions for regression testing

## Notes

- All timestamps are in UTC (ISO 8601 format)
- Bead IDs use the `tr-` prefix (testrepo)
- Worker names follow the alpha/bravo/charlie/delta/echo pattern
- Session IDs use `<worker>-<number>` format
