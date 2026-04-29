# HOOP Configuration Examples

This directory contains example configuration files for common HOOP usage patterns. Copy these to `~/.hoop/` and customize for your environment.

## Quick Start

```bash
# Create config directory
mkdir -p ~/.hoop

# Copy example configurations
cp docs/examples/config.yml ~/.hoop/
cp docs/examples/accounts.yaml ~/.hoop/
cp docs/examples/projects.yaml ~/.hoop/

# Edit to customize
nano ~/.hoop/config.yml
```

## File Reference

### [`config.yml`](config.yml)

Main HOOP daemon configuration. Controls:

- **Server settings** — bind address, port
- **UI preferences** — theme, default sort, page size
- **Backup** — S3 endpoint, bucket, encryption
- **Agent** — model selection, Morning Brief
- **Metrics** — Prometheus endpoint
- **Reflection ledger** — rule learning threshold
- **Pricing** — per-model cost overrides

**Common customizations:**

```yaml
# Expose on Tailscale interface
server:
  bind_addr: "0.0.0.0:3000"

# Enable daily backups to Backblaze B2
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-yourname
  encryption: true

# Use Opus for complex tasks
agent:
  model: claude-opus-4-7
```

### [`accounts.yaml`](accounts.yaml)

Rate limit configuration per CLI adapter account. HOOP uses these to track capacity and prevent quota exhaustion.

**Account IDs** are derived from adapter config directories:

| Adapter | Config path | Account ID |
|---------|-------------|------------|
| Claude Code | `~/.config/claude-code/` | `claude-code-default` |
| OpenCode | `~/.local/share/opencode/` | `opencode-default` |
| Aider | `~/.config/aider/` | `aider-default` |
| Codex | `~/.config/codex/` | `codex-default` |
| Gemini | `~/.config/gemini/` | `gemini-default` |

**Example: Configure Claude Max tier**

```yaml
accounts:
  claude-code-default:
    adapter: claude-code
    limits:
      prompts_per_5h: 1600
      prompts_per_7d: 8000
      tokens_per_minute: 40000
```

### [`projects.yaml`](projects.yaml)

Project registry definition. Normally managed via `hoop projects add`, but can be edited directly.

**Single-repo project:**

```yaml
projects:
  - name: myproject
    description: My main application
    workspaces:
      - path: /home/coding/myproject
        role: primary
```

**Multi-repo project:**

```yaml
projects:
  - name: myservice-deployment
    description: Multi-repo service deployment
    workspaces:
      - path: /home/coding/myservice
        role: source
      - path: /home/coding/declarative-config
        role: manifests
      - path: /home/coding/secrets
        role: secrets
```

### [`fleet.yaml`](fleet.yaml)

NEEDLE worker fleet configuration. HOOP observes these fleets but does not control them.

**Note:** This file lives in each NEEDLE workspace (e.g., `~/.needle/fleet.yaml` or `~/.myproject/.needle/fleet.yaml`), not in `~/.hoop/`.

**Example fleet with mixed workers:**

```yaml
name: example-fleet
workspace: /home/coding/myproject
workers:
  - name: worker-opus
    model: claude-opus-4-7
    harness: claude-code
    concurrency: 1
  - name: worker-sonnet
    model: claude-sonnet-4-6
    harness: claude-code
    concurrency: 2
```

See [NEEDLE documentation](https://github.com/jedarden/NEEDLE) for complete fleet configuration options.

## Common Patterns

### Pattern 1: Single Developer, Single Project

**config.yml:**

```yaml
server:
  bind_addr: "127.0.0.1:3000"

ui:
  theme: dark
  default_project_sort: activity

agent:
  model: claude-sonnet-4-6
  morning_brief_enabled: true

backup:
  enabled: false
```

**projects.yaml:**

```yaml
projects:
  - name: myproject
    description: My main project
    workspaces:
      - path: /home/coding/myproject
        role: primary
```

### Pattern 2: Multi-Repo Migration Project

**projects.yaml:**

```yaml
projects:
  - name: calico-migration
    description: Migrate cluster networking from Calico to Cilium
    workspaces:
      - path: /home/coding/ardenone-cluster
        role: infrastructure
      - path: /home/coding/declarative-config
        role: manifests
      - path: /home/coding/service-a
        role: workload
      - path: /home/coding/service-b
        role: workload
```

Use Patterns in the UI to track all related Stitches across repos.

### Pattern 3: Production with Backup

**config.yml:**

```yaml
server:
  bind_addr: "0.0.0.0:3000"

backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-prod
  prefix: hoop/
  schedule: "0 4 * * *"
  retention_days: 30
  encryption: true

agent:
  model: claude-opus-4-7
  max_tokens: 200000

metrics:
  enabled: true
  port: 9091

audit:
  retention_days: 90
  hash_chain: true
```

Set `HOOP_BACKUP_AGE_KEY` environment variable with your age public key.

### Pattern 4: High-Volume Multi-Account

**accounts.yaml:**

```yaml
accounts:
  claude-code-primary:
    adapter: claude-code
    limits:
      prompts_per_5h: 1600
      prompts_per_7d: 8000
      tokens_per_minute: 40000

  claude-code-secondary:
    adapter: claude-code
    limits:
      prompts_per_5h: 500
      prompts_per_7d: 2000
      tokens_per_minute: 20000

  opencode-zai:
    adapter: opencode
    limits:
      prompts_per_5h: 1600
      prompts_per_7d: 8000
```

HOOP tracks capacity per-account and warns before limits.

## Validation

After modifying configuration files, validate syntax:

```bash
# Check configuration syntax
hoop config validate

# Restart to apply changes
systemctl --user restart hoop

# Check logs for errors
journalctl --user -u hoop -n 50
```

## Schema Version

All configuration files include a `schema_version` field. HOOP checks this on startup and runs migrations if needed. Never manually change this field.

## Environment Variables

Sensitive values can be overridden via environment variables:

- `HOOP_BACKUP_AGE_KEY` — age public key for backup encryption
- `HOOP_ANTHROPIC_API_KEY` — Anthropic API key for agent
- `HOOP_LOG_LEVEL` — log level (debug, info, warn, error)

Environment variables take precedence over config file values.
