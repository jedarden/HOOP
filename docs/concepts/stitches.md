# Stitches

> A **Stitch** is a single conversation or work item within a project. It's HOOP's primary user-facing unit — operators work in Stitches, not beads.

## What makes a Stitch

A Stitch represents one thread of work or conversation, regardless of source:

- **Operator Stitches** — Human ↔ agent chat conversations
- **Dictated Stitches** — Voice notes captured via ADB dictation or direct upload
- **Worker Stitches** — NEEDLE worker CLI sessions (headless LLM execution)
- **Ad-hoc Stitches** — Operator's direct CLI sessions (`claude`, `aider`, etc.)

## Stitch lifecycle

1. **Created** — When a conversation starts, voice note is recorded, or worker begins
2. **Active** — While messages are being exchanged or work is progressing
3. **Closed** — When the work completes or conversation ends

## What a Stitch contains

| Field | Description |
|-------|-------------|
| `id` | Unique UUID |
| `project` | Project this Stitch belongs to |
| `kind` | `operator`, `dictated`, `worker`, or `ad-hoc` |
| `title` | Human-readable summary |
| `created_by` | Who created it (`system`, operator identity, or worker name) |
| `created_at` | ISO 8601 timestamp |
| `messages` | Conversation history (role + content) |
| `linked_beads` | Beads associated with this Stitch |

## Stitch vs Bead

**Beads** are NEEDLE's internal execution unit. HOOP abstracts beads into Stitches for the operator. In normal flow, you never see bead IDs — only Stitch names and titles.

| Concept | Visible to operator | Managed by |
|---------|-------------------|------------|
| **Stitch** | ✅ Yes | HOOP |
| **Bead** | ❌ No (expert/debug only) | NEEDLE (`br`) |

## Creating Stitches

### Via the UI
- Click the "+" button in the Stitches panel
- Choose a Stitch type
- Add a title and start adding content

### Via the agent
- Ask the agent to draft work
- Review the preview (cost estimate, risk assessment)
- Approve to create the Stitch

### Via voice dictation
- Press the dictation hotkey (or use ADB)
- Speak your note
- Stitch is created automatically with transcript

### Via CLI
- Run a headless CLI session in a tracked workspace
- HOOP automatically creates an ad-hoc Stitch

## Finding Stitches

```bash
# List all Stitches in a project
hoop stitches list myproject

# Search for Stitches
hoop stitches search "fix calico" myproject

# Get details of a specific Stitch
hoop stitches show <stitch-id>
```

## Stitch patterns

Common Stitch workflows:

| Pattern | When to use | Example |
|---------|-------------|---------|
| **Quick question** | One-off query to the agent | "What's the status of the migration?" |
| **Voice note** | Capture thought while away from keyboard | Dictate an idea while walking |
| **Bug investigation** | Deep dive with linked beads | Link relevant bug IDs and track findings |
| **Feature work** | Multi-step implementation | Track progress through worker sessions |
| **Cost anomaly** | Investigation of unusual spending | Document findings and remediation |

## Related concepts

- **Patterns** — Group multiple related Stitches toward a goal
- **Projects** — Logical grouping of work (may span multiple workspaces)
- **Beads** — Internal execution unit (abstracted by Stitches)
