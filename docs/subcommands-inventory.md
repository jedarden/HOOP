# HOOP CLI subcommands inventory

This inventory describes the command tree wired into the `hoop` binary. It is
based on `hoop-cli/src/main.rs`, the nested command types used by its
`Commands` enum, and the handlers for those types.

The inventory distinguishes command-tree nodes from executable leaf commands:

- **22 top-level variants** are declared by `Commands`.
- **10 top-level variants** are command groups with nested subcommands.
- **12 top-level variants** are standalone executable commands.
- **39 nested subcommands** are declared by the nested command enums.
- **51 executable leaf invocations** are therefore classified below (12
  standalone plus 39 nested).
- **61 wired command variants** are present when group nodes and leaf commands
  are counted together.

The source has a private `ConfigCommands` declaration in `main.rs`, but the
wired variant is `Config(config::ConfigCommands)`. The inventory follows the
wired type, so it includes both `config diff` and `config validate` from
`hoop-cli/src/config.rs`.

## Tag definitions

- **[interactive]** — the handler prompts for a yes/no response. Launching an
  editor (as `new` does) is called out separately and is not counted as a
  yes/no prompt.
- **[requires-confirm]** — the command requires an explicit `--confirm`,
  either always or when `--no-interactive` is set.
- **[read-only]** — the command does not intentionally mutate persistent HOOP
  state. A command that only reads data but is not implemented is still marked
  read-only and also marked `[unimplemented]`.
- **[rejects-no-interactive]** — the global flag parses, but the handler
  explicitly refuses to execute with it.
- **[mutating]** — supplementary tag for commands that write state or trigger
  an external state change without falling into one of the requested prompt or
  confirmation categories.
- **[group]** — a top-level command that only dispatches to nested commands;
  it is not included in leaf-command category counts.
- **[unimplemented]** — the command currently exits with a not-implemented
  error.

## Top-level commands

Each top-level variant is listed, including the nested commands it exposes.

- `serve`: [read-only]
- `projects`: [group]
  - `projects add <path>`: [mutating]
  - `projects scan <root> [--yes]`: [interactive, mutating]
  - `projects list [--json]`: [read-only]
  - `projects remove <name> [--confirm]`: [interactive, requires-confirm, mutating]
  - `projects show <name>`: [read-only]
- `add <path>`: [mutating]
- `scan <root> [--yes]`: [interactive, mutating]
- `list`: [read-only]
- `remove <name> [--confirm]`: [interactive, requires-confirm, mutating]
- `status [project] [--json]`: [read-only]
- `audit`: [group]
  - `audit check [--json] [--strict]`: [read-only]
  - `audit verify [--json]`: [read-only]
- `agent`: [unimplemented]
- `new <project> [--dry-run]`: [mutating]
  - Opens `$EDITOR` (or `$VISUAL`, then `vi`) to edit the Stitch draft. The
    global `--no-interactive` flag is not consumed by this handler.
- `stitch [project]`: [read-only, unimplemented]
- `install-systemd`: [mutating]
- `backup`: [group]
  - `backup trigger [--addr <socket-addr>]`: [mutating]
  - `backup status [--addr <socket-addr>]`: [read-only]
- `restore --from <s3-uri> [--dry-run] [--confirm]`: [interactive, requires-confirm, mutating]
- `migrate`: [group]
  - `migrate run --confirm`: [requires-confirm, mutating]
  - `migrate status [--json]`: [read-only]
  - `migrate major-upgrade [--from <major>] --confirm`: [requires-confirm, mutating]
  - `migrate rollback <version> --confirm`: [requires-confirm, mutating]
  - `migrate rebuild-percentile-index`: [mutating]
- `script`: [group]
  - `script run <name> [args...]`: [mutating]
  - `script list [--project <name>]`: [read-only]
  - `script show <name>`: [read-only]
- `config`: [group]
  - `config diff`: [read-only]
  - `config validate`: [read-only]
- `risk-patterns`: [group]
  - `risk-patterns add --id ... --name ... --description ... --keywords ... --label-keywords ... --fix-recommendation ... --severity ... --category ...`: [mutating]
  - `risk-patterns list [--json]`: [read-only]
  - `risk-patterns seed [--force]`: [mutating]
- `skills`: [group]
  - `skills import <path>`: [mutating]
  - `skills enable <name>`: [mutating]
  - `skills disable <name>`: [mutating]
  - `skills list [--json]`: [read-only]
  - `skills show <name>`: [read-only]
  - `skills remove <name>`: [mutating]
- `pattern`: [group]
  - `pattern new <title> [options]`: [mutating]
  - `pattern list [--json]`: [read-only]
  - `pattern show <id> [--json]`: [read-only]
  - `pattern update <id> [options]`: [mutating]
  - `pattern close <id>`: [mutating]
  - `pattern delete <id> [--confirm]`: [interactive, mutating]
  - `pattern add-member <id> <stitch-id>`: [mutating]
  - `pattern remove-member <id> <stitch-id>`: [mutating]
  - `pattern add-query <id> <query>`: [mutating]
  - `pattern remove-query <id> <query>`: [mutating]
- `reflection`: [group]
  - `reflection export [--format] [--out] [--addr] [--dry-run]`: [mutating]
    - The normal path writes `MEMORY.md`, per-entry markdown files, and an
      export log. `--dry-run` is the read-only variant.
- `init`: [interactive, rejects-no-interactive, mutating]

## Interaction and confirmation details

### Commands with yes/no prompts

These seven leaf commands read a yes/no response in their normal execution
path:

- `scan`
- `projects scan`
- `remove`
- `projects remove`
- `restore`
- `pattern delete`
- `init`

`pattern delete` uses its own optional `--confirm` flag to skip its prompt. It
does not consume the global `--no-interactive` flag, so that global flag does
not suppress this prompt.

`new` is interactive in the broader sense that it opens an editor, but it has
no yes/no prompt and is intentionally not included in the `[interactive]`
count.

### Commands requiring `--confirm`

- `remove` and `projects remove` require `--confirm` when
  `--no-interactive` is set; otherwise they prompt.
- `restore` has the same conditional requirement. Its `--dry-run` path exits
  before the confirmation check because it makes no changes.
- `migrate run`, `migrate major-upgrade`, and `migrate rollback` always require
  `--confirm`, independent of `--no-interactive`.
- `pattern delete` does **not** require `--confirm`; the flag is optional and
  only bypasses its prompt.

### `--no-interactive` behavior

`Cli.no_interactive` is a global Clap flag (`-y` / `--no-interactive`), so it
is syntactically available throughout the command tree. Handler behavior is
more specific:

| Behavior | Commands | Count |
|---|---|---:|
| Honors the flag to suppress prompts or auto-confirm safely | `scan`, `projects scan`, `remove`, `projects remove`, `restore` | 5 |
| Explicitly rejects the flag at execution time | `init` | 1 |
| Parses the flag but does not consume it | All other leaf commands, including `pattern delete` and `new` | 45 |

The final row is why `[rejects-no-interactive]` means an explicit handler
rejection, not a Clap parse failure. The flag is global and therefore is not
absent from individual command definitions.

## Category counts

Counts below are over the **51 executable leaf invocations**. Categories can
overlap; for example, `restore` is both interactive and requires confirmation.

| Category | Count | Commands |
|---|---:|---|
| `[interactive]` | 7 | `scan`, `projects scan`, `remove`, `projects remove`, `restore`, `pattern delete`, `init` |
| `[requires-confirm]` | 6 | `remove`, `projects remove`, `restore`, `migrate run`, `migrate major-upgrade`, `migrate rollback` |
| `[read-only]` | 19 | `serve`, `projects list`, `projects show`, `list`, `status`, `audit check`, `audit verify`, `stitch`, `backup status`, `migrate status`, `script list`, `script show`, `config diff`, `config validate`, `risk-patterns list`, `skills list`, `skills show`, `pattern list`, `pattern show` |
| `[rejects-no-interactive]` | 1 | `init` |
| `[mutating]` | 31 | All state-changing leaves listed with the supplementary tag above |
| `[unimplemented]` | 2 | `agent`, `stitch` |

The four requested categories are not mutually exclusive. The supplementary
`[mutating]` count makes commands such as `add`, `backup trigger`, and
`reflection export` visible rather than incorrectly classifying them as
read-only.

## Source map

- Top-level and local nested enums: [`hoop-cli/src/main.rs`](../hoop-cli/src/main.rs)
- Backup commands: [`hoop-cli/src/backup.rs`](../hoop-cli/src/backup.rs)
- Script commands: [`hoop-cli/src/script.rs`](../hoop-cli/src/script.rs)
- Config commands: [`hoop-cli/src/config.rs`](../hoop-cli/src/config.rs)
- Risk-pattern commands: [`hoop-cli/src/risk_patterns.rs`](../hoop-cli/src/risk_patterns.rs)
- Skills commands: [`hoop-cli/src/skills.rs`](../hoop-cli/src/skills.rs)
- Pattern commands: [`hoop-cli/src/patterns.rs`](../hoop-cli/src/patterns.rs)
- Reflection commands: [`hoop-cli/src/reflection.rs`](../hoop-cli/src/reflection.rs)
