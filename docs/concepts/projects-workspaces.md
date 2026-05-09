# Projects & Workspaces

> **Projects** are logical units you care about. **Workspaces** are the physical repos on disk. A project can contain one or more workspaces.

## The distinction

| Concept | What it means | Example |
|---------|---------------|---------|
| **Project** | A logical unit as you think of it | "Kalshi weather bot" |
| **Workspace** | A single git repo with `.beads/` | `/home/coding/kalshi-weather` |

One project = one or more workspaces. This matters when a logical unit spans multiple repos.

## When to use multi-workspace projects

**Single workspace** (most common):
- One repo = one project
- Example: A standalone application

**Multi-workspace** (advanced):
- Multiple repos = one logical project
- Examples:
  - **Infrastructure repo** + **service repo** + **secrets repo** = "Production deployment"
  - **Source repo** + **manifests repo** = "Kubernetes migration"

## Project registry

HOOP tracks projects in `~/.hoop/projects.yaml`:

```yaml
projects:
  - name: kalshi-weather
    description: Weather prediction bot
    workspaces:
      - path: /home/coding/kalshi-weather
        role: primary

  - name: calico-migration
    description: Migrate cluster networking
    workspaces:
      - path: /home/coding/ardenone-cluster
        role: infrastructure
      - path: /home/coding/declarative-config
        role: manifests
      - path: /home/coding/service-a
        role: workload
```

## Managing projects

### CLI commands

```bash
# Add a single-repo project
hoop projects add /home/coding/myproject --name myproject

# Add a multi-repo project
hoop projects add-multi myservice-deployment \
  /home/coding/myservice:source \
  /home/coding/declarative-config:manifests \
  /home/coding/secrets:secrets

# Scan for projects automatically
hoop projects scan ~/

# List registered projects
hoop projects list

# Remove a project
hoop projects remove myproject

# Verify project health
hoop projects check
```

### Via `hoop init`

The first-time setup wizard offers to scan your home directory and auto-register discovered bead workspaces.

## Project cards

The HOOP UI shows one card per project on the dashboard:

| Field | Description |
|-------|-------------|
| **Name** | Project label (e.g., "Kalshi Weather") |
| **Active Stitches** | Number of open conversations |
| **Cost today** | Total LLM spend across all workspaces |
| **Alerts** | Degraded workspaces, stuck workers, etc. |
| **Last activity** | Most recent Stitch or bead creation |

Clicking a card shows all Stitches across that project's workspaces.

## Workspace roles

When registering workspaces, you can optionally specify a role:

| Role | When to use |
|------|-------------|
| `primary` | Main code workspace |
| `source` | Application source code |
| `manifests` | Kubernetes/deployment configs |
| `infrastructure` | Infrastructure-as-code |
| `secrets` | Secret management |
| `docs` | Documentation |
| `tests` | Test suites |

Roles are informational — they help you distinguish workspace purpose at a glance.

## What gets tracked per project

| Data type | Source |
|-----------|--------|
| **Stitches** | All conversations across workspaces |
| **Beads** | From each workspace's `.beads/beads.db` |
| **Cost** | Aggregated from all worker sessions |
| **Workers** | NEEDLE fleet status per workspace |
| **Files** | Git-tracked source code (file browser) |

## Project health

A project shows as **degraded** when:

- A workspace's `.beads/` directory is missing
- A workspace's git repo has errors
- No workers are reporting for an active project
- Disk space is critically low

Degraded projects show a warning card but don't block other projects.

## Related concepts

- **Stitches** — Live at the project level, aggregate across workspaces
- **Patterns** — Can span multiple projects
- **Beads** — Workspace-scoped (one `.beads/` per repo)
