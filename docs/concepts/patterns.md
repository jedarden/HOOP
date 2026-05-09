# Patterns

> A **Pattern** is an optional, operator-curated grouping of Stitches toward a shared goal. Patterns can span multiple projects and are HOOP's mechanism for organizing work at scale.

## What makes a Pattern

A Pattern represents a longer-running initiative or goal that involves multiple Stitches:

| Attribute | Description |
|-----------|-------------|
| `id` | Unique UUID |
| `name` | Human-readable name |
| `description` | What this Pattern accomplishes |
| `projects` | List of projects this Pattern spans |
| `stitches` | Stitches grouped under this Pattern |
| `created_at` | When the Pattern was created |

## When to use Patterns

**Use a Pattern when:**
- Multiple Stitches contribute to one goal (e.g., a migration across repos)
- Work spans multiple projects (e.g., infrastructure upgrade)
- You need to track progress over weeks or months
- You want to see all related conversations in one place

**Don't use a Pattern when:**
- Work is a single, self-contained conversation (use a Stitch)
- Work is project-specific and short-lived (project view is sufficient)

## Pattern examples

### Multi-repo migration
```
Pattern: "Calico → Cilium migration"
Projects: ardenone-cluster, declarative-config, service-a, service-b
Stitches:
  - Assess current Calico configuration
  - Design Cilium manifests
  - Test Cilium in staging
  - Roll out to production cluster
  - Remove Calico leftovers
```

### Feature development
```
Pattern: "Q2 performance improvements"
Project: myapp
Stitches:
  - Profile database queries
  - Add caching layer
  - Optimize hot paths
  - Benchmark improvements
```

### Investigation
```
Pattern: "Cost spike investigation"
Projects: kalshi-weather, infrastructure
Stitches:
  - Identify anomalous spending
  - Trace expensive API calls
  - Implement rate limiting
  - Verify fix effectiveness
```

## Creating Patterns

### Via the UI
1. Navigate to the Patterns view
2. Click "Create Pattern"
3. Add a name and description
4. Select which projects to include
5. Add existing Stitches or create new ones

### Via the CLI
```bash
# Create a new Pattern
hoop patterns create "Calico migration" \
  --description "Migrate from Calico to Cilium networking" \
  --projects ardenone-cluster,declarative-config

# Add a Stitch to a Pattern
hoop patterns add-stitch <pattern-id> <stitch-id>

# List Patterns
hoop patterns list

# Show Pattern details
hoop patterns show <pattern-id>
```

## Pattern vs Project

| Dimension | Project | Pattern |
|-----------|---------|---------|
| **Purpose** | Logical unit (repo or group of repos) | Goal or initiative |
| **Scope** | One or more workspaces | Can span multiple projects |
| **Duration** | Long-lived (exists as long as code exists) | Temporary (deleted when goal achieved) |
| **Stitch ownership** | Every Stitch belongs to exactly one project | Stitches can be in zero or more patterns |

## Pattern analytics

Patterns provide aggregate analytics across all member Stitches:

- **Total cost** — Sum of all Stitch costs
- **Duration** — Time from first to last Stitch
- **Completion** — Closed vs open Stitches
- **Per-project breakdown** — Which projects contributed most

## Progressive Pattern suggestions

HOOP's agent may suggest creating a Pattern when it detects:

- 10+ Stitches share a theme but no Pattern exists
- Stitches across multiple projects reference similar topics
- Repeated keywords suggest an ongoing initiative

Suggestions appear as onboarding prompts and can be dismissed or acted upon.

## Related concepts

- **Stitches** — Individual conversations grouped by Patterns
- **Projects** — Logical units that Patterns may span
- **Reflection Ledger** — Learned rules may apply across Pattern Stitches
