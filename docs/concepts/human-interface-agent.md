# Human-Interface Agent

> The **human-interface agent** is a persistent LLM session hosted by HOOP. It's your primary conversation partner — answering questions, drafting work, and providing summaries.

## What the agent does

The agent has **read-only access** to everything in your fleet:

- All Stitches across all projects
- All beads and their states
- All CLI session transcripts
- All costs and capacity metrics
- All file contents via the file browser

The agent has **one write**: creating Stitches via the preview queue. It never:

- Launches, stops, or kills NEEDLE workers
- Mutates bead state beyond `br create`
- Enforces capacity or routes work
- Modifies files directly

## Agent capabilities

### 1. Question answering

```
You: What's going on in kalshi-weather?

Agent: The kalshi-weather project has:
  - 3 open Stitches (rate limit retries, model upgrade, config migration)
  - 1 stuck worker (worker-opus on bead-123)
  - $12.40 in costs today (above usual baseline of ~$8)
  - Last successful closure: "Fix Calico IP selection" 2 hours ago
```

### 2. Work drafting

```
You: Draft a Stitch to investigate the rate limit issue.

Agent: [Creates a draft with]
  Title: "Investigate evening rate-limit failures"
  Description: Analysis of why retries are failing more often...
  Estimated cost: $2.50
  Risk level: Medium
```

### 3. Morning Brief

Each day (or on login), the agent produces a briefing:

- What closed successfully overnight
- What failed (with cost impact)
- What's stuck or anomalous
- Pre-drafted Stitches for follow-ups
- **One headline** — the priority for today

### 4. Cross-project propagation

When you close a Stitch that has structural siblings:

```
Agent: You just closed "fix Calico IP selection" in iad-acb.
      The same pattern exists in iad-ci, rs-manager...

      [Propagates the fix to sibling projects]
```

## Agent configuration

Configure in `~/.hoop/config.yml`:

```yaml
agent:
  # Which LLM adapter to use
  adapter: claude  # claude | anthropic | zai

  # Model selection
  model: claude-opus-4-7

  # Optional rate limiting
  rate_limit_rpm: 50

  # Optional cost cap
  cost_cap_usd: 100.00

  # Morning brief schedule
  morning_brief_enabled: true
  morning_brief_hour: 7
```

## Agent adapters

| Adapter | Description | When to use |
|---------|-------------|-------------|
| `claude` | Claude Code CLI | Default, requires `claude` CLI in PATH |
| `anthropic` | Anthropic API | Direct API access, requires `HOUP_AGENT_ANTHROPIC_API_KEY` |
| `zai` | ZAI proxy with GLM models | Fallback for Claude outages |

Switching adapters is a config change — no migration needed.

## Agent-off switch

HOOP remains fully functional without the agent. Disable by:

1. **UI**: Click "Disable" in the agent chat pane
2. **Config**: Remove the `agent` section from `config.yml`
3. **API**: `POST /api/agent/disable`

Read-only surfaces (dashboard, Stitches, file browser) continue working.

## Agent tool belt

The agent has access to these tools via the MCP server:

**Read tools:**
- `find_stitches(project, filter)` — List Stitches
- `read_stitch(id)` — Get Stitch details with messages
- `find_beads(project, filter)` — List beads
- `read_bead(id)` — Get bead details
- `read_file(project, path, revision)` — Read file contents
- `grep(project, pattern)` — Search files
- `search_conversations(query)` — Search transcripts
- `summarize_project(project)` — Get activity summary
- `summarize_day()` — Get daily summary

**Write tools (one write):**
- `create_stitch(project, title, description, kind, attachments)` — Creates a draft in the preview queue

**Utility:**
- `escalate_to_operator(message)` — Send a UI banner

## Session persistence

The agent's session persists across restarts:

1. **First spawn** — Creates new session with conversation history
2. **Daemon restart** — Reattaches to existing session
3. **Adapter change** — Creates new session (old session archived)

Sessions are stored in `~/.hoop/fleet.db` and include:

- Full message history
- Tool calls and results
- Cost tracking per session
- Session metadata (adapter, model, start time)

## Privacy and redaction

The agent sees redacted content:

- CLI transcripts are redacted before being sent to the agent
- Secrets detected by the privacy scanner are replaced with `[REDACTED]`
- File contents are scanned before being included in context

See `docs/concepts/privacy.md` for details.

## Related concepts

- **Reflection Ledger** — Learned rules injected into every agent session
- **Morning Brief** — Autonomous daily briefing from the agent
- **MCP Server** — How the agent communicates with HOOP
