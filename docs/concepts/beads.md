# Beads

> A **Bead** is NEEDLE's internal execution unit — the smallest tracked piece of work. HOOP abstracts beads into Stitches, so operators rarely need to think about them.

## What makes a Bead

A Bead represents one unit of executable work in the NEEDLE system:

| Attribute | Description |
|-----------|-------------|
| `id` | Unique identifier (UUID or timestamp-based) |
| `title` | Short description of the work |
| `state` | `open`, `claimed`, `closed`, or `failed` |
| `workspace` | Which `.beads/` directory owns it |
| `created_at` | When the bead was created |
| `claimed_at` | When a worker picked it up |
| `closed_at` | When the work completed |

## Bead lifecycle

```
open → claimed → closed
                ↘ failed
```

1. **Open** — Created by `br create`, waiting for a worker
2. **Claimed** — A worker has picked it up and is executing
3. **Closed** — Work completed successfully
4. **Failed** — Worker encountered an error

## Bead vs Stitch

| Concept | Visibility | Managed by | Scope |
|---------|------------|------------|-------|
| **Bead** | Expert/debug only | NEEDLE (`br`) | One workspace |
| **Stitch** | Primary UI | HOOP | One project (may span workspaces) |

HOOP reads beads from each workspace's `.beads/` directory and presents them as Stitches. The mapping is:

- **Worker Stitch** → Aggregates all beads executed by a worker session
- **Ad-hoc Stitch** → Tracks beads created during a manual CLI session
- **Operator/Dictated Stitches** → May have linked beads for reference

## Where beads live

Each workspace has its own bead queue:

```
/home/coding/myproject/
  .beads/
    beads.db       # SQLite database
    beads.jsonl    # Event log
    open/          # Open beads (optional)
```

HOOP never writes directly to `.beads/` — it shells out to `br` (beads_rust):

```bash
# HOOP creates beads via br
br create --title "Fix Calico IP selection" --workspace myproject

# HOOP reads bead state via br queries
br list --workspace myproject
```

## When you see bead IDs

Bead IDs appear in these contexts:

| Context | Why bead ID is shown |
|---------|---------------------|
| **Expert/debug views** | For tracing execution |
| **Audit log** | To track what `br create` was called |
| **Git commit trailers** | `Bead-Id` trailer links commit to bead |
| **Worker logs** | To identify which bead a worker executed |

In normal day-to-day use, you work with Stitch names and titles, not bead IDs.

## Creating beads

### Via HOOP (recommended)

When you draft work in HOOP and approve the preview:

1. HOOP calls `br create` with the Stitch title and description
2. NEEDLE workers pick up the bead from the queue
3. The Stitch tracks progress as beads move through states

### Via br directly (advanced)

```bash
# Create a bead manually
br create --title "Investigate rate limit" \
  --workspace kalshi-weather \
  --description "Look into why retries are failing"

# List open beads
br list --workspace kalshi-weather --state open

# Show bead details
br show <bead-id>
```

## Bead states and worker behavior

| Bead state | Worker behavior |
|------------|-----------------|
| `open` | Worker can claim this bead |
| `claimed` | Worker is actively executing |
| `closed` | Worker completed successfully |
| `failed` | Worker encountered error; may be retryable |

HOOP's Stitch status derives from the states of its linked beads:

- **Open** — All beads are open or some claimed
- **In progress** — At least one bead is claimed
- **Complete** — All beads are closed
- **Failed** — Any bead failed (may be partial)

## Bead-Stitch linking

A Stitch can link to beads for traceability:

1. Worker Stitch → Auto-linked to beads it executed
2. Ad-hoc Stitch → Links to beads created during session
3. Operator/Dictated Stitches → Optional manual links

The **Stitch Net-Diff viewer** shows which files were touched by linked beads, using git commit trailers to map beads → commits.

## Related concepts

- **Stitches** — HOOP's user-facing abstraction over beads
- **Workspaces** — Each has its own `.beads/` queue
- **NEEDLE** — The worker supervision system that executes beads
