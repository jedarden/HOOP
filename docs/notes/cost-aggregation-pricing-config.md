# Cost Aggregation & Pricing Configuration

## Overview

HOOP aggregates token usage from CLI session files into cost buckets:
- Per-project
- Per-adapter
- Per-model
- Per-strand
- Per-day

## Configuration File

**Location:** `~/.hoop/pricing.yml`

If not present, HOOP uses default pricing (see `hoop-daemon/src/cost.rs`).

## Format

```yaml
adapters:
  <adapter-name>:
    models:
      <model-name>:
        input_per_million: <float>      # Price per million input tokens
        output_per_million: <float>     # Price per million output tokens
        cache_read_per_million: <float> # Optional: cache read price
        cache_write_per_million: <float> # Optional: cache write price
    default_model: <model-name>         # Fallback for unknown models
```

## Supported Adapters

- `claude` — Claude Code sessions
- `codex` — OpenAI Codex sessions  
- `gemini` — Google Gemini CLI sessions
- `opencode` — OpenCode sessions
- `aider` — Aider sessions (uses Claude pricing)

## Session Usage Fields by Adapter

| Adapter   | Input Tokens         | Output Tokens        | Cache Read           | Cache Write          |
|-----------|---------------------|----------------------|----------------------|----------------------|
| Claude    | `input_tokens`       | `output_tokens`      | `cache_read_tokens`  | `cache_creation_tokens` |
| Codex     | `token_count` (user) | `token_count` (asst) | —                    | —                    |
| OpenCode  | `tokens.input`       | `tokens.output`      | `tokens.cache_read`  | `tokens.cache_write`  |
| Gemini    | `usage.promptTokenCount` | `usage.candidatesTokenCount` | `usage.cachedContentTokenCount` | — |

## API Endpoints

- `GET /api/cost/buckets` — All cost buckets
- `GET /api/cost/buckets/:project` — Buckets for a specific project
- `POST /api/cost/reload-pricing` — Reload pricing config

## Reloading Pricing

After updating `~/.hoop/pricing.yml`, reload via:

```bash
curl -X POST http://localhost:3000/api/cost/reload-pricing
```

Or restart the daemon.
