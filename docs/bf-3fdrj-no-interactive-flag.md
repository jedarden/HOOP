# no_interactive Flag Documentation

## Task: bf-3fdrj

### Location

**File:** `hoop-cli/src/main.rs`
**Lines:** 24-35 (clap Parser struct definition)
**Extraction:** Line 258 (flag value extracted into local variable)

### Current Flag Configuration

```rust
#[derive(Parser, Debug)]
#[command(name = "hoop")]
#[command(about = "HOOP - The operator's pane of glass", long_about = None)]
struct Cli {
    /// Global flag to suppress all interactive prompts (alias: -y)
    ///
    /// The `global = true` attribute ensures this flag is available to all subcommands.
    /// It can be specified at any level: `hoop --no-interactive <subcommand>` or
    /// `hoop <subcommand> --no-interactive`. The flag value is extracted once at
    /// parse time (line 253) and passed to command handlers that need it.
    #[arg(short = 'y', long = 'no-interactive', global = true)]
    no_interactive: bool,

    #[command(subcommand)]
    command: Commands,
}
```

### Flag Attributes

| Attribute | Value | Description |
|-----------|-------|-------------|
| `short` | `'y'` | Short form alias: `-y` |
| `long` | `'no-interactive'` | Long form: `--no-interactive` |
| `global` | `true` | Available to all subcommands |
| `type` | `bool` | Boolean flag (presence = true) |

### Behavior

1. **Global availability**: The `global = true` attribute ensures the flag is available to all subcommands at any position:
   - `hoop --no-interactive <subcommand>` ✓
   - `hoop <subcommand> --no-interactive` ✓

2. **Single extraction point**: The flag value is extracted once at parse time (line 258) and passed to command handlers that need it:
   ```rust
   let cli = Cli::parse();
   let no_interactive = cli.no_interactive;
   ```

3. **Semantic meaning**: When `no_interactive = true`, all interactive prompts are suppressed and operations proceed automatically.

### Usage in Commands

The flag is passed to the following command handlers:

| Command | Handler Function | File |
|---------|-------------------|------|
| `projects` subcommands | `handle_projects()` | main.rs:424 |
| `scan` | `projects::scan_projects()` | projects.rs:583 |
| `remove` | `projects::remove_project()` | projects.rs:449 |
| `restore` | `restore::run_restore()` | restore.rs:279 |
| `init` | `init::run_init_wizard()` | init.rs:40 |

### Related Commands

Some commands have their own `--yes` flag that ORs with the global `no_interactive`:
- `hoop scan --yes` (line 297)
- `hoop projects scan --yes` (line 434)

### Testing

Acceptance test: `tests/acceptance/s6_machine_mode.rs::s6_no_interactive_required_for_read_operations`

### Key Design Point

The documentation comment in the code (lines 28-33) explicitly explains:
- The global flag behavior
- Position flexibility
- Single extraction pattern
- Handler propagation pattern

This is the authoritative definition and configuration of the `no_interactive` flag in HOOP.
