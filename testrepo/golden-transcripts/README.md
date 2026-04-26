# Golden Transcripts Corpus

This directory contains expected-parsed-output fixtures for adapter parser regression testing.

## Structure

```
golden-transcripts/
├── README.md
├── claude/
│   └── v1.0/
│       ├── simple/
│       │   └── simple_turn.jsonl
│       ├── tool_heavy/
│       │   └── tool_heavy_turn.jsonl
│       └── failure/
│           └── failure_turn.jsonl
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

## File Format

Each `.jsonl` file contains newline-delimited JSON objects representing events from an adapter's stream output.

### Scenarios

Each adapter must have three scenarios:

1. **simple**: A basic text-only turn with minimal events
2. **tool_heavy**: A turn with multiple tool invocations and results
3. **failure**: A turn that results in an error condition

### Event Types

Events vary by adapter format but generally include:

- **Text/content deltas**: Streaming text responses from the model
- **Tool use/calls**: Model invoking tools
- **Tool results**: Output from tool execution
- **Turn completion**: End-of-turn markers with usage stats
- **Errors**: Error conditions (rate limits, auth failures, etc.)

## Updating Golden Transcripts

When an adapter's parser changes or a new event type is added:

1. Run the adapter to generate fresh output
2. Validate the output matches the expected event schema
3. Update the appropriate `.jsonl` file in the version directory
4. If the format changes fundamentally, create a new version directory (e.g., `v1.1/`)

## Size Constraints

The total corpus size must remain under 10MB to keep CI fast and git operations manageable.

## Tests

The regression test at `hoop-daemon/tests/golden_transcripts_regression.rs` validates:

- Directory structure matches the spec
- All adapters have all required scenarios
- All `.jsonl` files contain valid JSON
- Each scenario file has non-empty content
- Scenario types contain appropriate event types (text, tools, errors)
- Total corpus size is bounded

## Plan Reference

§14.3 golden transcripts — Real LLM integration is tested via recorded fixtures.
