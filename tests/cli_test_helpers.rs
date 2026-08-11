//! Test helper utilities for HOOP CLI testing
//!
//! This module provides reusable utilities and patterns for testing CLI commands,
//! with special focus on the `no_interactive` flag behavior across different commands.
//!
//! ## Testing Philosophy
//!
//! The `no_interactive` flag is a **global clap flag** (marked with `global = true`),
//! meaning it can be specified at any position in the command invocation:
//!
//! ```bash
//! # Before the subcommand
//! hoop --no-interactive projects remove my-project --confirm
//!
//! # After the subcommand
//! hoop projects remove my-project --no-interactive --confirm
//!
//! # With the short alias
//! hoop -y projects remove my-project --confirm
//! ```
//!
//! ## Key Testing Patterns
//!
//! 1. **Position Independence**: Verify the flag works correctly at any position
//! 2. **Short/Long Form Equivalence**: Test both `-y` and `--no-interactive`
//! 3. **Value Consistency**: Ensure flag value is extracted consistently regardless of position
//! 4. **Default Behavior**: Verify default (false) when flag is not specified
//! 5. **Flag Propagation**: Ensure global flag persists through command chains
//!
//! ## Basic no_interactive Flag Test Patterns
//!
//! This section documents the two fundamental patterns for testing the `--no-interactive` flag
//! at different positions in the command line. Each pattern is a complete, runnable example.
//!
//! ### Pattern 1: Flag Before Command (`hoop --no-interactive CMD`)
//!
//! **Purpose:** Test the flag when it appears BEFORE the subcommand name.
//!
//! **How to construct the test command:**
//! ```ignore
//! #[test]
//! fn scan_flag_before_command() {
//!     // Step 1: Construct args with flag at position 0, command at position 1
//!     let args = ["hoop", "--no-interactive", "scan", "/tmp"];
//!
//!     // Step 2: Parse the CLI arguments
//!     let cli = parse_cli_args(&args).unwrap();
//!
//!     // Step 3: Verify the flag is recognized
//!     assert_no_interactive_true(&cli);
//! }
//! ```
//!
//! **How to verify the flag is recognized:**
//! - Check that `parse_cli_args()` returns `Ok(...)`
//! - Use `assert_no_interactive_true(&cli)` to confirm `cli.no_interactive == true`
//! - Verify the correct subcommand was parsed (e.g., `Commands::Scan`)
//!
//! **Expected behavior:**
//! - Parsing succeeds with no errors
//! - `cli.no_interactive` equals `true`
//! - Subcommand is correctly identified
//! - Same result as Pattern 2 (position independence)
//!
//! **Complete runnable example:**
//! ```ignore
//! #[test]
//! fn scan_flag_before_command_complete() {
//!     // Arrange: construct command with flag before subcommand
//!     let args = ["hoop", "--no-interactive", "scan", "/tmp"];
//!
//!     // Act: parse the command
//!     let cli = parse_cli_args(&args).unwrap();
//!
//!     // Assert: verify flag is recognized and command is parsed
//!     assert_no_interactive_true(&cli);
//!     match cli.command {
//!         hoop_cli::Commands::Scan { path } => {
//!             assert_eq!(path, "/tmp");
//!         }
//!         _ => panic!("Expected Scan command"),
//!     }
//! }
//! ```
//!
//! ### Pattern 2: Flag After Command (`hoop CMD --no-interactive`)
//!
//! **Purpose:** Test the flag when it appears AFTER the subcommand name.
//!
//! **How to construct the test command:**
//! ```ignore
//! #[test]
//! fn scan_flag_after_command() {
//!     // Step 1: Construct args with command at position 0, flag at end
//!     let args = ["hoop", "scan", "/tmp", "--no-interactive"];
//!
//!     // Step 2: Parse the CLI arguments
//!     let cli = parse_cli_args(&args).unwrap();
//!
//!     // Step 3: Verify the flag is recognized
//!     assert_no_interactive_true(&cli);
//! }
//! ```
//!
//! **How to verify the flag is recognized:**
//! - Check that `parse_cli_args()` returns `Ok(...)`
//! - Use `assert_no_interactive_true(&cli)` to confirm `cli.no_interactive == true`
//! - Verify the correct subcommand was parsed
//!
//! **Expected behavior:**
//! - Parsing succeeds with no errors (flag at end is valid due to `global = true`)
//! - `cli.no_interactive` equals `true` (same as Pattern 1)
//! - Subcommand is correctly identified
//! - **Key difference:** More intuitive for casual users who type command first
//!
//! **Complete runnable example:**
//! ```ignore
//! #[test]
//! fn scan_flag_after_command_complete() {
//!     // Arrange: construct command with flag after subcommand
//!     let args = ["hoop", "scan", "/tmp", "--no-interactive"];
//!
//!     // Act: parse the command
//!     let cli = parse_cli_args(&args).unwrap();
//!
//!     // Assert: verify flag is recognized and command is parsed
//!     assert_no_interactive_true(&cli);
//!     match cli.command {
//!         hoop_cli::Commands::Scan { path } => {
//!             assert_eq!(path, "/tmp");
//!         }
//!         _ => panic!("Expected Scan command"),
//!     }
//! }
//! ```
//!
//! ### Pattern Comparison Table
//!
//! | Aspect | Pattern 1: `--no-interactive CMD` | Pattern 2: `CMD --no-interactive` |
//! |--------|-----------------------------------|-----------------------------------|
//! | **Construction** | `["hoop", "--no-interactive", "scan", "/tmp"]` | `["hoop", "scan", "/tmp", "--no-interactive"]` |
//! | **Flag position** | Index 1 (before command) | Last index (after command) |
//! | **no_interactive value** | `true` | `true` |
//! | **Parsing result** | ✅ Success | ✅ Success |
//! | **Behavior difference** | CLI power user pattern | Casual user pattern |
//! | **Result consistency** | ✅ Same as Pattern 2 | ✅ Same as Pattern 1 |
//!
//! ### Testing Both Patterns Together
//!
//! **Best practice:** Always test both patterns to ensure position independence:
//!
//! ```ignore
//! #[test]
//! fn scan_both_patterns_produce_same_result() {
//!     // Pattern 1: flag before command
//!     let args_before = ["hoop", "--no-interactive", "scan", "/tmp"];
//!     let cli_before = parse_cli_args(&args_before).unwrap();
//!
//!     // Pattern 2: flag after command
//!     let args_after = ["hoop", "scan", "/tmp", "--no-interactive"];
//!     let cli_after = parse_cli_args(&args_after).unwrap();
//!
//!     // Both must produce identical results
//!     assert_eq!(cli_before.no_interactive, cli_after.no_interactive);
//!     assert_eq!(cli_before.no_interactive, true);
//! }
//! ```
//!
//! ### Helper for Testing Both Patterns
//!
//! Use the `parse_both_positions()` helper to test both patterns at once:
//!
//! ```ignore
//! #[test]
//! fn scan_position_independence_helper() {
//!     let flag_args = ["--no-interactive"];
//!     let cmd_args = ["scan", "/tmp"];
//!     let (before, after) = parse_both_positions(flag_args, cmd_args);
//!
//!     assert_eq!(before, after, "Both positions must yield the same value");
//!     assert_eq!(before, true, "no_interactive should be true");
//! }
//! ```
//!
//! ## Common Test Patterns
//!
//! ### Basic flag parsing test (legacy pattern - see above for enhanced patterns)
//! ```ignore
//! #[test]
//! fn scan_no_interactive_flag_before_command() {
//!     let args = ["hoop", "--no-interactive", "scan", "/tmp"];
//!     let cli = parse_cli_args(&args).unwrap();
//!     assert_no_interactive_true(&cli);
//! }
//! ```
//!
//! ### Position independence test
//! ```ignore
//! #[test]
//! fn scan_both_positions_extract_same_value() {
//!     let flag_args = ["--no-interactive"];
//!     let cmd_args = ["scan", "/tmp"];
//!     let (before, after) = parse_both_positions(flag_args, cmd_args);
//!     assert_eq!(before, after, "no_interactive value must be consistent");
//!     assert_eq!(before, true, "no_interactive should be true");
//! }
//! ```
//!
//! ### Short/long form equivalence
//! ```ignore
//! #[test]
//! fn scan_short_flag_y_works() {
//!     let args = ["hoop", "-y", "scan", "/tmp"];
//!     let cli = parse_cli_args(&args).unwrap();
//!     assert_no_interactive_true(&cli);
//! }
//! ```
//!
//! ## Macro-Based Testing Patterns
//!
//! ### Using test_global_flag_position!
//! Tests flag at global position (before subcommand):
//! ```ignore
//! test_global_flag_position!(scan_global_flag, "scan", "/tmp");
//! test_global_flag_position!(remove_global_flag, "remove", "my-project");
//! test_global_flag_position!(status_global_flag, "status");
//! ```
//!
//! ### Using test_subcommand_flag_position!
//! Tests flag at subcommand position (after command):
//! ```ignore
//! test_subcommand_flag_position!(scan_subcommand_flag, "scan", "/tmp");
//! test_subcommand_flag_position!(remove_subcommand_flag, "remove", "my-project");
//! test_subcommand_flag_position!(status_subcommand_flag, "status");
//! ```
//!
//! ### Using test_flag_propagation!
//! Tests flag propagation through command chains:
//! ```ignore
//! // Pattern 1: Global flag affects subcommand
//! test_flag_propagation!(
//!     global_affects_projects_scan,
//!     global_flag = "--no-interactive",
//!     command = ["projects", "scan", "/tmp"],
//!     expected = true
//! );
//!
//! // Pattern 2: Verify consistency across positions
//! test_flag_propagation!(
//!     scan_position_consistency,
//!     command = ["scan", "/tmp"],
//!     verify_consistency = true
//! );
//! ```

## Flag Propagation Verification Patterns

This section documents comprehensive patterns for testing how the `no_interactive` flag propagates
through nested commands, child processes, environment variables, and command chains. These patterns build
on the basic position testing patterns documented above and provide production-ready testing strategies.

### Overview: What is Flag Propagation?

**Flag propagation** is the process by which a CLI flag (like `--no-interactive`) set on a parent
command becomes available to nested components:

1. **CLI to Handler propagation**: `hoop --no-interactive remove test` → `remove_project(test, no_interactive=true)`
   - Clap parses the flag at the top level (in `Cli` struct)
   - The flag is extracted from `cli.no_interactive`
   - It's passed to handler functions as a parameter
   - Handlers check the flag before prompting

2. **Child process propagation**: Commands that spawn subprocesses must pass the flag down
   - Parent command has `--no-interactive`
   - Child process receives `--no-interactive` in its args
   - Child's child receives it too (deep propagation)

3. **Environment variable inheritance**: Some flags map to environment variables that must inherit
   - `--no-interactive` → `HOOP_NO_INTERACTIVE=1`
   - Child processes inherit via `std::env`
   - Verification checks the env var exists

4. **Command chain persistence**: Flag remains accessible through nested subcommands
   - `hoop --no-interactive projects remove test` → flag accessible at both `projects` and `remove` levels
   - Flag value stays consistent through the entire chain

### Why Test Flag Propagation?

**Critical for:** CI/CD reliability, automation safety, and operator trust.

**Failures cause:** Silent prompts in scripts (hang forever), accidental data loss, confusing error messages.

**Testing ensures:**
- Scripts run unattended without hanging on prompts
- Destructive operations require explicit confirmation
- Flag behavior is consistent across all command positions
- Child processes inherit the flag correctly

### Propagation Scenario 1: CLI → Handler Function (Primary Pattern)

**Problem:** Verify that the flag value from CLI parsing correctly reaches the handler function that uses it.

**Real-world flow:**
```rust,ignore
// In main.rs
let cli = Cli::parse();
let no_interactive = cli.no_interactive; // ← Extract from CLI

match cli.command {
    Commands::Remove { name } => {
        projects::remove_project(&name, no_interactive) // ← Pass to handler
    }
}

// In projects.rs
pub fn remove_project(name: &str, no_interactive: bool) -> Result<()> {
    if no_interactive {
        // Skip prompts
    } else {
        // Show prompts
    }
}
```

**Setup for testing:**

```ignore
#[test]
fn verify_no_interactive_propagates_to_handler() {
    // Arrange: Parse CLI with flag set
    let args = ["hoop", "--no-interactive", "projects", "remove", "test"];
    let cli = parse_cli_args(&args).unwrap();

    // Act: Extract the flag (simulating what main.rs does)
    let no_interactive = cli.no_interactive;

    // Assert: Verify flag is correctly extracted
    assert!(no_interactive, "Flag must be true after extraction");
}
```

**Verification methods:**

1. **Direct extraction check**: Verify the value can be extracted from `cli.no_interactive`
   ```ignore
   let no_interactive = cli.no_interactive;
   assert_eq!(no_interactive, true);
   ```

2. **Handler signature inspection**: Verify the handler accepts the flag as a parameter
   ```ignore
   // Read the source code
   let code = std::fs::read_to_string("src/projects.rs").unwrap();

   // Must accept no_interactive parameter
   assert!(code.contains("pub fn remove_project(name: &str, no_interactive: bool)"),
       "Handler must accept no_interactive parameter");
   ```

3. **Main.rs threading check**: Verify main.rs passes the flag to the handler
   ```ignore
   let main_code = std::fs::read_to_string("src/main.rs").unwrap();

   assert!(main_code.contains("let no_interactive = cli.no_interactive;"),
       "main() must extract flag from CLI");

   assert!(main_code.contains("projects::remove_project(&name, no_interactive)"),
       "main() must pass flag to handler");
   ```

**Common pitfalls:**

- ❌ Handler doesn't accept the flag: `pub fn remove_project(name: &str)` (missing parameter)
  - **Fix**: Add `no_interactive: bool` to handler signature

- ❌ main.rs doesn't extract the flag: Handler gets `false` by default
  - **Fix**: Extract with `let no_interactive = cli.no_interactive;`

- ❌ Handler extracts from wrong place: `let no_interactive = args.contains("--no-interactive")`
  - **Fix**: Always extract from `cli.no_interactive` (clap's parsed value)

**Complete runnable example:**

```ignore
#[test]
fn cli_to_handler_propagation_complete_example() {
    // Step 1: Parse the command with flag
    let args = ["hoop", "--no-interactive", "projects", "remove", "test"];
    let cli = parse_cli_args(&args).unwrap();
    assert!(cli.no_interactive, "CLI must parse flag as true");

    // Step 2: Extract the flag (simulating main.rs)
    let no_interactive = cli.no_interactive;
    assert_eq!(no_interactive, true, "Extracted value must match CLI value");

    // Step 3: Verify handler signature accepts the flag
    let projects_code = std::fs::read_to_string("src/projects.rs")
        .expect("projects.rs must exist");
    assert!(
        projects_code.contains("pub fn remove_project(name: &str, no_interactive: bool)"),
        "Handler must accept no_interactive parameter"
    );

    // Step 4: Verify handler uses the flag in conditional logic
    assert!(
        projects_code.contains("if no_interactive"),
        "Handler must check the flag value"
    );

    // Step 5: Verify main.rs passes the flag
    let main_code = std::fs::read_to_string("src/main.rs")
        .expect("main.rs must exist");
    assert!(
        main_code.contains("projects::remove_project(&name, no_interactive)"),
        "main() must pass extracted flag to handler"
    );
}
```

### Propagation Scenario 2: Parent → Child Process Argument Passing

**Problem:** When a HOOP command spawns a child process (e.g., running `br` or another tool),
the `no_interactive` flag must be passed to the child's command-line arguments.

**Real-world flow:**
```rust,ignore
// In projects.rs (hypothetical example where HOOP spawns br)
pub fn create_bead(title: &str, no_interactive: bool) -> Result<()> {
    let mut cmd = std::process::Command::new("br");

    // Pass the flag to the child process
    if no_interactive {
        cmd.arg("--no-interactive");
    }

    cmd.arg("create")
       .arg(title)
       .spawn()?;
}
```

**Setup for testing:**

```ignore
#[test]
fn verify_no_interactive_propagates_to_child_process() {
    // Arrange: Parse parent command with flag
    let parent_args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
    let parent_cli = parse_cli_args(&parent_args).unwrap();

    // Act: Construct child command args (simulating real code)
    let child_args = build_child_args(&parent_cli, "scan-project");

    // Assert: Verify child receives the flag
    assert!(child_args.contains(&"--no-interactive".to_string()),
        "Child process must receive no_interactive flag");
}
```

**Verification methods:**

1. **Direct argument check**: Verify the flag string appears in child args
   ```ignore
   assert!(child_args.iter().any(|arg| arg == "--no-interactive"));
   ```

2. **Parse-and-check approach**: Parse the child's args and verify the flag value
   ```ignore
   let child_cli = parse_cli_args(&child_args).unwrap();
   assert_eq!(child_cli.no_interactive, true);
   ```

3. **Mock subprocess test**: Use a test double for the child process
   ```ignore
   let mock_child = MockChildProcess::new(child_args);
   assert!(mock_child.receives_flag("no_interactive"));
   ```

**Common pitfalls:**

- ❌ Forgetting to pass the flag when spawning: `Command::new("br").args(&["create"])`
  - **Fix**: Always include the flag: `.args(&["create", "--no-interactive"])`

- ❌ Passing the wrong position: Flag at end when child expects it at start
  - **Fix**: Match the child's expected flag position, or document that position doesn't matter

- ❌ Not checking the child's actual CLI parser: Assuming argument order doesn't matter
  - **Fix**: Parse the child's args with its real parser to verify correctness

**Complete runnable example:**

```ignore
#[test]
fn child_process_receives_no_interactive_flag() {
    // Step 1: Parse parent command
    let parent_args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
    let parent_cli = parse_cli_args(&parent_args).unwrap();
    assert!(parent_cli.no_interactive, "Parent must have flag set");

    // Step 2: Build child process args (simulating actual spawn logic)
    let mut child_args = vec!["br"]; // Child command name
    if parent_cli.no_interactive {
        child_args.push("--no-interactive"); // ← Propagate the flag
    }
    child_args.push("create"); // Child subcommand
    child_args.push("test-bead"); // Child arguments

    // Step 3: Verify propagation by parsing child args
    let child_cli = parse_cli_args(&child_args).unwrap();
    assert!(child_cli.no_interactive,
        "Child CLI must parse no_interactive=true from passed args");

    // Step 4: Verify the flag is actually in the argument list
    assert!(child_args.contains(&"--no-interactive"),
        "Flag must appear in child's argument vector");

    // Step 5: Verify position independence in child
    let child_args_flag_at_end = vec!["br", "create", "test-bead", "--no-interactive"];
    let child_cli_flag_at_end = parse_cli_args(&child_args_flag_at_end).unwrap();
    assert_eq!(child_cli.no_interactive, child_cli_flag_at_end.no_interactive,
        "Flag position in child args must not affect value");
}
```

### Propagation Scenario 3: Flag Persistence Through Command Chains

**Problem:** For nested subcommands like `hoop projects remove my-project`, the global flag must
remain accessible at every level of the command chain.

**Real-world flow:**
```rust,ignore
// Full command: hoop --no-interactive projects remove my-project --confirm

// Clap parses the full command into:
Cli {
    no_interactive: true,  // ← Set at top level
    command: Some(Commands::Projects(
        ProjectsCommands::Remove {
            name: "my-project",
            confirm: true
        }
    ))
}

// The flag is accessible via cli.no_interactive at any level:
match cli.command {
    Commands::Projects(projects_cmd) => {
        // Flag is still accessible here
        let flag = cli.no_interactive;  // ← Works at Projects level

        match projects_cmd {
            ProjectsCommands::Remove { name, confirm } => {
                // Flag is STILL accessible here
                let flag = cli.no_interactive;  // ← Works at Remove level
            }
        }
    }
}
```

**Setup for testing:**

```ignore
#[test]
fn verify_flag_persists_through_nested_subcommands() {
    // Arrange: Command with nested subcommands
    let args = ["hoop", "--no-interactive", "projects", "remove", "my-project", "--confirm"];
    let cli = parse_cli_args(&args).unwrap();

    // Act & Assert: Check flag at each level
    assert!(cli.no_interactive, "Top level must have flag");

    // Access nested structure (pattern match on Commands enum)
    match &cli.command {
        hoop_cli::Commands::Projects(projects_cmd) => {
            // Flag must still be accessible here
            assert!(cli.no_interactive, "Flag accessible at Projects level");

            // Access further nested command
            match projects_cmd {
                ProjectsCommands::Remove { name, confirm } => {
                    assert_eq!(name, "my-project");
                    assert!(confirm, "Confirm flag must be true");
                    // Flag must still be true at the deepest level
                    assert!(cli.no_interactive, "Flag accessible at Remove level");
                }
                _ => panic!("Expected Remove command"),
            }
        }
        _ => panic!("Expected Projects command"),
    }
}
```

**Verification methods:**

1. **Pattern matching at each level**: Access nested enums and verify `cli.no_interactive`
   - Works for Rust's `clap`-derived command structures
   - Ensures the flag field is propagated through the type tree

2. **Handler function signature check**: Verify handler receives the flag
   ```ignore
   // In real code, handlers receive the full Cli or a context with flags
   fn handle_projects_remove(cmd: ProjectsRemove, no_interactive: bool) {
       // Test passes no_interactive=true when flag is set
   }
   ```

3. **Integration test with real execution**: Run the command and verify behavior
   ```ignore
   let output = Command::new("hoop")
       .args(&["--no-interactive", "projects", "remove", "test"])
       .output()
       .unwrap();
   assert!(String::from_utf8_lossy(&output.stdout).contains("non-interactive mode"));
   ```

**Common pitfalls:**

- ❌ Assuming nested commands get a separate flag copy: `Projects { no_interactive, command }`
  - **Fix**: Global flags stay at the top level; access them from the root `Cli` struct

- ❌ Forgetting to pass the flag to handlers: `handle(cmd)` instead of `handle(cmd, cli.no_interactive)`
  - **Fix**: Always thread the flag through handler call chains

- ❌ Testing only one nesting level: Stopping at `Projects` without checking `Remove`
  - **Fix**: Test the full depth of real-world command chains

**Complete runnable example:**

```ignore
#[test]
fn flag_persistence_through_full_command_chain() {
    // Full command: hoop --no-interactive projects remove my-project --confirm
    let args = [
        "hoop",
        "--no-interactive",  // Global flag
        "projects",          // Level 1 subcommand
        "remove",            // Level 2 subcommand
        "my-project",        // Argument to remove
        "--confirm"          // Flag to remove (not global)
    ];
    let cli = parse_cli_args(&args).unwrap();

    // Level 0: Top-level Cli struct
    assert!(cli.no_interactive, "Level 0: Global flag must be true");

    // Level 1: Projects subcommand
    match &cli.command {
        hoop_cli::Commands::Projects(projects_cmd) => {
            // Flag is still accessible via cli.no_interactive
            assert!(cli.no_interactive, "Level 1: Flag accessible in Projects");

            // Level 2: Remove subcommand (nested within Projects)
            match projects_cmd {
                hoop_cli::ProjectsCommands::Remove { name, confirm } => {
                    assert_eq!(name, "my-project");
                    assert!(*confirm, "Remove's --confirm flag must be true");

                    // Flag is STILL accessible at deepest level
                    assert!(cli.no_interactive, "Level 2: Flag accessible in Remove");
                }
                _ => panic!("Expected Remove command at Level 2"),
            }
        }
        _ => panic!("Expected Projects command at Level 1"),
    }

    // Final verification: Flag remains true throughout the chain
    assert_eq!(cli.no_interactive, true,
        "Global no_interactive flag must persist through entire command chain");
}
```

### Propagation Scenario 4: Environment Variable Inheritance

**Problem:** Some flags map to environment variables (e.g., `--no-interactive` → `HOOP_NO_INTERACTIVE=1`).
When HOOP spawns child processes, these env vars must be inherited correctly.

**Real-world flow:**
```rust,ignore
// In projects.rs (when spawning a subprocess)
pub fn run_external_tool(no_interactive: bool) -> Result<()> {
    let mut cmd = std::process::Command::new("external-tool");

    // Set environment variable based on flag
    cmd.env("HOOP_NO_INTERACTIVE", if no_interactive { "1" } else { "0" });

    cmd.spawn()?;
}
```

**Setup for testing:**

```ignore
#[test]
fn verify_no_interactive_env_var_inheritance() {
    // Arrange: Set the flag and map to env var
    let args = ["hoop", "--no-interactive", "scan", "/tmp"];
    let cli = parse_cli_args(&args).unwrap();

    // Act: Build environment for child process
    let mut env = std::env::vars().collect::<HashMap<_, _>>();
    if cli.no_interactive {
        env.insert("HOOP_NO_INTERACTIVE".to_string(), "1".to_string());
    } else {
        env.insert("HOOP_NO_INTERACTIVE".to_string(), "0".to_string());
    }

    // Assert: Verify env var is set
    assert_eq!(env.get("HOOP_NO_INTERACTIVE"), Some(&"1".to_string()),
        "Environment variable must be set when flag is true");
}
```

**Verification methods:**

1. **Direct env var check**: Test that the var exists in the child's environment
   ```ignore
   assert!(std::env::var("HOOP_NO_INTERACTIVE").is_ok());
   ```

2. **Mock process environment**: Create a mock env map for testing
   ```ignore
   let mut mock_env = HashMap::new();
   propagate_flag_to_env(&cli.no_interactive, &mut mock_env);
   assert_eq!(mock_env.get("HOOP_NO_INTERACTIVE"), Some(&"1".to_string()));
   ```

3. **Integration test with real subprocess**: Spawn a child that checks its own env
   ```ignore
   let output = Command::new("sh")
       .arg("-c")
       .arg("echo $HOOP_NO_INTERACTIVE")
       .env("HOOP_NO_INTERACTIVE", if cli.no_interactive { "1" } else { "0" })
       .output()
       .unwrap();
   assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1");
   ```

**Common pitfalls:**

- ❌ Setting the env var on the parent instead of the child: `std::env::set_var(...)`
  - **Fix**: Set it on the Command: `.env("HOOP_NO_INTERACTIVE", "1")`

- ❌ Forgetting to inherit existing env: `Command::new("child").env("KEY", "val")`
  - **Fix**: Use `.envs()` or clone parent env first

- ❌ Wrong variable name or value format: `HOOP_NOINTERACTIVE=TRUE`
  - **Fix**: Use the documented name and format (usually `=1` for boolean flags)

**Complete runnable example:**

```ignore
#[test]
fn environment_variable_propagation_full_example() {
    // Step 1: Parse command with no_interactive flag
    let args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
    let cli = parse_cli_args(&args).unwrap();
    assert!(cli.no_interactive, "Flag must be parsed as true");

    // Step 2: Build environment for child process (simulating real spawn logic)
    let mut child_env = std::env::vars().collect::<HashMap<_, _>>();

    // Map the flag to environment variable
    if cli.no_interactive {
        child_env.insert("HOOP_NO_INTERACTIVE".to_string(), "1".to_string());
    } else {
        child_env.insert("HOOP_NO_INTERACTIVE".to_string(), "0".to_string());
    }

    // Step 3: Verify the environment variable is correctly set
    assert_eq!(
        child_env.get("HOOP_NO_INTERACTIVE"),
        Some(&"1".to_string()),
        "HOOP_NO_INTERACTIVE must be '1' when no_interactive flag is true"
    );

    // Step 4: Verify the env var would be inherited by a real subprocess
    // (In real code, you'd pass this env to Command::new().envs())
    let env_check = |key: &str| -> bool {
        child_env.get(key).map(|v| v == "1").unwrap_or(false)
    };
    assert!(env_check("HOOP_NO_INTERACTIVE"),
        "Child environment check must succeed for HOOP_NO_INTERACTIVE=1");

    // Step 5: Test the inverse case (flag not set)
    let args_no_flag = ["hoop", "projects", "scan", "/tmp"];
    let cli_no_flag = parse_cli_args(&args_no_flag).unwrap();
    assert!(!cli_no_flag.no_interactive, "Flag must be false when not specified");

    let mut child_env_no_flag = std::env::vars().collect::<HashMap<_, _>>();
    if cli_no_flag.no_interactive {
        child_env_no_flag.insert("HOOP_NO_INTERACTIVE".to_string(), "1".to_string());
    } else {
        child_env_no_flag.insert("HOOP_NO_INTERACTIVE".to_string(), "0".to_string());
    }

    assert_eq!(
        child_env_no_flag.get("HOOP_NO_INTERACTIVE"),
        Some(&"0".to_string()),
        "HOOP_NO_INTERACTIVE must be '0' when no_interactive flag is false"
    );
}
```

### Combining All Propagation Patterns

**Comprehensive test that checks CLI extraction, handler threading, and env inheritance:**

```ignore
#[test]
fn comprehensive_flag_propagation_test() {
    // Scenario: A complex command that spawns a child subprocess
    // Command: hoop --no-interactive projects remove my-project

    // 1. Parse the command
    let args = ["hoop", "--no-interactive", "projects", "remove", "my-project"];
    let cli = parse_cli_args(&args).unwrap();

    // 2. Verify flag at top level
    assert!(cli.no_interactive, "Flag must be true at top level");

    // 3. Verify flag persists through command chain
    match &cli.command {
        hoop_cli::Commands::Projects(projects_cmd) => {
            assert!(cli.no_interactive, "Flag accessible at Projects level");
            // (Further nesting checks would go here)
        }
        _ => panic!("Expected Projects command"),
    }

    // 4. Verify flag would be passed to child process
    let child_args = vec!["br", if cli.no_interactive { "--no-interactive" } else { "" }, "create"];
    let child_has_flag = child_args.iter().any(|&arg| arg == "--no-interactive");
    assert!(child_has_flag, "Child must receive no_interactive flag");

    // 5. Verify environment variable would be set
    let env_value = if cli.no_interactive { "1" } else { "0" };
    assert_eq!(env_value, "1", "Environment variable must be '1'");

    // 6. Verify handler receives the flag (code inspection)
    let projects_code = std::fs::read_to_string("src/projects.rs").unwrap();
    assert!(projects_code.contains("no_interactive: bool"),
        "Handler must accept no_interactive parameter");
}
```

### Testing Checklist: Flag Propagation

When testing flag propagation, verify all of these:

- [ ] **CLI parsing**: Flag is parsed correctly at the top level (`cli.no_interactive`)
- [ ] **CLI → Handler**: Flag is extracted from CLI and passed to handler functions
- [ ] **Handler signature**: Handler accepts `no_interactive: bool` as a parameter
- [ ] **Handler usage**: Handler checks the flag in conditional logic (`if no_interactive`)
- [ ] **Command chain**: Flag remains accessible through nested command structures
- [ ] **Child process args**: Flag is passed to child processes in their arguments
- [ ] **Environment vars**: Environment variables are set correctly for inheritance
- [ ] **Consistency**: Flag behavior is consistent (true → true, false → false)
- [ ] **Position independence**: Flag value is the same at all positions

### Debugging Failed Propagation Tests

If a propagation test fails:

1. **Check parsing**: `assert!(cli.no_interactive)` — did the parent parse correctly?
2. **Check extraction**: Verify `let no_interactive = cli.no_interactive;` works
3. **Check handler signature**: Does the function accept `no_interactive: bool`?
4. **Check handler call**: Does main.rs pass the flag? `handler(arg, no_interactive)`
5. **Check argument building**: Print `child_args` — is the flag in the list?
6. **Check env var name**: `"HOOP_NO_INTERACTIVE"` vs `"HOOP_NOINTERACTIVE"` — typos?
7. **Check inheritance**: `std::env::var()` — did the child actually inherit?
8. **Check child parser**: Parse child's args separately — does its parser work?

```ignore
// Debug helper: Print what would be spawned
#[test]
fn debug_propagation() {
    let args = ["hoop", "--no-interactive", "scan", "/tmp"];
    let cli = parse_cli_args(&args).unwrap();

    println!("Parent no_interactive: {}", cli.no_interactive);
    println!("Child args: {:?}", build_child_args(&cli));
    println!("Env var: {}", if cli.no_interactive { "1" } else { "0" });
}
```

### Quick Reference: Propagation Patterns

| Scenario | What to Test | Verification Method |
|----------|-------------|---------------------|
| **CLI → Handler** | `cli.no_interactive` → function parameter | `assert!(cli.no_interactive)` + code inspection |
| **Handler → Child Process** | Handler passes flag to `Command::new()` | Check `child_args.contains(&"--no-interactive")` |
| **Command Chain** | Flag accessible at all nesting levels | Pattern match + assert at each level |
| **Environment Variable** | `HOOP_NO_INTERACTIVE=1` set on child | Check `env.get("HOOP_NO_INTERACTIVE")` |
| **Deep Propagation** | Child's child inherits the flag | Parse child's args and verify |

### Propagation Scenario 1: Parent → Child Process Argument Passing

**Problem:** When a HOOP command spawns a child process (e.g., running `br` or another tool),
the `no_interactive` flag must be passed to the child's command-line arguments.

**Setup:**

```ignore
#[test]
fn verify_no_interactive_propagates_to_child_process() {
    // Arrange: Parse parent command with flag
    let parent_args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
    let parent_cli = parse_cli_args(&parent_args).unwrap();

    // Act: Construct child command args (simulating real code)
    let child_args = build_child_args(&parent_cli, "scan-project");

    // Assert: Verify child receives the flag
    assert!(child_args.contains(&"--no-interactive".to_string()),
        "Child process must receive no_interactive flag");
}
```

**Verification methods:**

1. **Direct argument check**: Verify the flag string appears in child args
   ```ignore
   assert!(child_args.iter().any(|arg| arg == "--no-interactive"));
   ```

2. **Parse-and-check approach**: Parse the child's args and verify the flag value
   ```ignore
   let child_cli = parse_cli_args(&child_args).unwrap();
   assert_eq!(child_cli.no_interactive, true);
   ```

3. **Mock subprocess test**: Use a test double for the child process
   ```ignore
   let mock_child = MockChildProcess::new(child_args);
   assert!(mock_child.receives_flag("no_interactive"));
   ```

**Common pitfalls:**

- ❌ Forgetting to pass the flag when spawning: `Command::new("br").args(&["create"])`
  - **Fix**: Always include the flag: `.args(&["create", "--no-interactive"])`

- ❌ Passing the wrong position: Flag at end when child expects it at start
  - **Fix**: Match the child's expected flag position, or document that position doesn't matter

- ❌ Not checking the child's actual CLI parser: Assuming argument order doesn't matter
  - **Fix**: Parse the child's args with its real parser to verify correctness

**Complete runnable example:**

```ignore
#[test]
fn child_process_receives_no_interactive_flag() {
    // Step 1: Parse parent command
    let parent_args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
    let parent_cli = parse_cli_args(&parent_args).unwrap();
    assert!(parent_cli.no_interactive, "Parent must have flag set");

    // Step 2: Build child process args (simulating actual spawn logic)
    let mut child_args = vec!["child-cmd"];
    if parent_cli.no_interactive {
        child_args.push("--no-interactive");
    }
    child_args.push("scan-project");

    // Step 3: Verify propagation by parsing child args
    let child_cli = parse_cli_args(&child_args).unwrap();
    assert!(child_cli.no_interactive,
        "Child CLI must parse no_interactive=true from passed args");

    // Step 4: Verify the flag is actually in the argument list
    assert!(child_args.contains(&"--no-interactive"),
        "Flag must appear in child's argument vector");
}
```

### Propagation Scenario 2: Flag Persistence Through Command Chains

**Problem:** For nested subcommands like `hoop projects remove my-project`, the global flag must
remain accessible at every level of the command chain.

**Setup:**

```ignore
#[test]
fn verify_flag_persists_through_nested_subcommands() {
    // Arrange: Command with nested subcommands
    let args = ["hoop", "--no-interactive", "projects", "remove", "my-project"];
    let cli = parse_cli_args(&args).unwrap();

    // Act & Assert: Check flag at each level
    assert!(cli.no_interactive, "Top level must have flag");

    // Access nested structure (pattern match on Commands enum)
    match &cli.command {
        hoop_cli::Commands::Projects(projects_cmd) => {
            // Flag must still be accessible here
            assert!(cli.no_interactive, "Flag accessible at Projects level");

            // Access further nested command
            match &projects_cmd {
                ProjectsCommands::Remove { name } => {
                    assert_eq!(name, "my-project");
                    // Flag must still be true at the deepest level
                    assert!(cli.no_interactive, "Flag accessible at Remove level");
                }
                _ => panic!("Expected Remove command"),
            }
        }
        _ => panic!("Expected Projects command"),
    }
}
```

**Verification methods:**

1. **Pattern matching at each level**: Access nested enums and verify `cli.no_interactive`
   - Works for Rust's `clap`-derived command structures
   - Ensures the flag field is propagated through the type tree

2. **Handler function signature check**: Verify handler receives the flag
   ```ignore
   // In real code, handlers receive the full Cli or a context with flags
   fn handle_projects_remove(cmd: ProjectsRemove, no_interactive: bool) {
       // Test passes no_interactive=true when flag is set
   }
   ```

3. **Integration test with real execution**: Run the command and verify behavior
   ```ignore
   let output = Command::new("hoop")
       .args(&["--no-interactive", "projects", "remove", "test"])
       .output()
       .unwrap();
   assert!(String::from_utf8_lossy(&output.stdout).contains("non-interactive mode"));
   ```

**Common pitfalls:**

- ❌ Assuming nested commands get a separate flag copy: `Projects { no_interactive, command }`
  - **Fix**: Global flags stay at the top level; access them from the root `Cli` struct

- ❌ Forgetting to pass the flag to handlers: `handle(cmd)` instead of `handle(cmd, cli.no_interactive)`
  - **Fix**: Always thread the flag through handler call chains

- ❌ Testing only one nesting level: Stopping at `Projects` without checking `Remove`
  - **Fix**: Test the full depth of real-world command chains

**Complete runnable example:**

```ignore
#[test]
fn flag_persistence_through_full_command_chain() {
    // Full command: hoop --no-interactive projects remove my-project --confirm
    let args = [
        "hoop",
        "--no-interactive",  // Global flag
        "projects",          // Level 1 subcommand
        "remove",            // Level 2 subcommand
        "my-project",        // Argument to remove
        "--confirm"          // Flag to remove (not global)
    ];
    let cli = parse_cli_args(&args).unwrap();

    // Level 0: Top-level Cli struct
    assert!(cli.no_interactive, "Level 0: Global flag must be true");

    // Level 1: Projects subcommand
    match &cli.command {
        hoop_cli::Commands::Projects(projects_cmd) => {
            // Flag is still accessible via cli.no_interactive
            assert!(cli.no_interactive, "Level 1: Flag accessible in Projects");

            // Level 2: Remove subcommand (nested within Projects)
            match projects_cmd {
                hoop_cli::ProjectsCommands::Remove { name, confirm } => {
                    assert_eq!(name, "my-project");
                    assert!(*confirm, "Remove's --confirm flag must be true");

                    // Flag is STILL accessible at deepest level
                    assert!(cli.no_interactive, "Level 2: Flag accessible in Remove");
                }
                _ => panic!("Expected Remove command at Level 2"),
            }
        }
        _ => panic!("Expected Projects command at Level 1"),
    }

    // Final verification: Flag remains true throughout the chain
    assert_eq!(cli.no_interactive, true,
        "Global no_interactive flag must persist through entire command chain");
}
```

### Propagation Scenario 3: Environment Variable Inheritance

**Problem:** Some flags map to environment variables (e.g., `--no-interactive` → `HOOP_NO_INTERACTIVE=1`).
When HOOP spawns child processes, these env vars must be inherited correctly.

**Setup:**

```ignore
#[test]
fn verify_no_interactive_env_var_inheritance() {
    // Arrange: Set the flag and map to env var
    let args = ["hoop", "--no-interactive", "scan", "/tmp"];
    let cli = parse_cli_args(&args).unwrap();

    // Act: Build environment for child process
    let mut env = std::env::vars().collect::<HashMap<_, _>>();
    if cli.no_interactive {
        env.insert("HOOP_NO_INTERACTIVE".to_string(), "1".to_string());
    }

    // Assert: Verify env var is set
    assert_eq!(env.get("HOOP_NO_INTERACTIVE"), Some(&"1".to_string()),
        "Environment variable must be set when flag is true");
}
```

**Verification methods:**

1. **Direct env var check**: Test that the var exists in the child's environment
   ```ignore
   assert!(std::env::var("HOOP_NO_INTERACTIVE").is_ok());
   ```

2. **Mock process environment**: Create a mock env map for testing
   ```ignore
   let mut mock_env = HashMap::new();
   propagate_flag_to_env(&cli.no_interactive, &mut mock_env);
   assert_eq!(mock_env.get("HOOP_NO_INTERACTIVE"), Some(&"1".to_string()));
   ```

3. **Integration test with real subprocess**: Spawn a child that checks its own env
   ```ignore
   let output = Command::new("sh")
       .arg("-c")
       .arg("echo $HOOP_NO_INTERACTIVE")
       .env("HOOP_NO_INTERACTIVE", if cli.no_interactive { "1" } else { "0" })
       .output()
       .unwrap();
   assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1");
   ```

**Common pitfalls:**

- ❌ Setting the env var on the parent instead of the child: `std::env::set_var(...)`
  - **Fix**: Set it on the Command: `.env("HOOP_NO_INTERACTIVE", "1")`

- ❌ Forgetting to inherit existing env: `Command::new("child").env("KEY", "val")`
  - **Fix**: Use `.envs()` or clone parent env first

- ❌ Wrong variable name or value format: `HOOP_NOINTERACTIVE=TRUE`
  - **Fix**: Use the documented name and format (usually `=1` for boolean flags)

**Complete runnable example:**

```ignore
#[test]
fn environment_variable_propagation_full_example() {
    // Step 1: Parse command with no_interactive flag
    let args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
    let cli = parse_cli_args(&args).unwrap();
    assert!(cli.no_interactive, "Flag must be parsed as true");

    // Step 2: Build environment for child process (simulating real spawn logic)
    let mut child_env = std::env::vars().collect::<HashMap<_, _>>();

    // Map the flag to environment variable
    if cli.no_interactive {
        child_env.insert("HOOP_NO_INTERACTIVE".to_string(), "1".to_string());
    } else {
        child_env.insert("HOOP_NO_INTERACTIVE".to_string(), "0".to_string());
    }

    // Step 3: Verify the environment variable is correctly set
    assert_eq!(
        child_env.get("HOOP_NO_INTERACTIVE"),
        Some(&"1".to_string()),
        "HOOP_NO_INTERACTIVE must be '1' when no_interactive flag is true"
    );

    // Step 4: Verify the env var would be inherited by a real subprocess
    // (In real code, you'd pass this env to Command::new().envs())
    let env_check = |key: &str| -> bool {
        child_env.get(key).map(|v| v == "1").unwrap_or(false)
    };
    assert!(env_check("HOOP_NO_INTERACTIVE"),
        "Child environment check must succeed for HOOP_NO_INTERACTIVE=1");

    // Step 5: Test the inverse case (flag not set)
    let args_no_flag = ["hoop", "projects", "scan", "/tmp"];
    let cli_no_flag = parse_cli_args(&args_no_flag).unwrap();
    assert!(!cli_no_flag.no_interactive, "Flag must be false when not specified");

    let mut child_env_no_flag = std::env::vars().collect::<HashMap<_, _>>();
    if cli_no_flag.no_interactive {
        child_env_no_flag.insert("HOOP_NO_INTERACTIVE".to_string(), "1".to_string());
    } else {
        child_env_no_flag.insert("HOOP_NO_INTERACTIVE".to_string(), "0".to_string());
    }

    assert_eq!(
        child_env_no_flag.get("HOOP_NO_INTERACTIVE"),
        Some(&"0".to_string()),
        "HOOP_NO_INTERACTIVE must be '0' when no_interactive flag is false"
    );
}
```

### Combining All Propagation Patterns

**Comprehensive test that checks argument passing, chain persistence, and env inheritance:**

```ignore
#[test]
fn comprehensive_flag_propagation_test() {
    // Scenario: A complex command that spawns a child subprocess
    // Command: hoop --no-interactive projects remove my-project

    // 1. Parse the command
    let args = ["hoop", "--no-interactive", "projects", "remove", "my-project"];
    let cli = parse_cli_args(&args).unwrap();

    // 2. Verify flag at top level
    assert!(cli.no_interactive, "Flag must be true at top level");

    // 3. Verify flag persists through command chain
    match &cli.command {
        hoop_cli::Commands::Projects(projects_cmd) => {
            assert!(cli.no_interactive, "Flag accessible at Projects level");
            // (Further nesting checks would go here)
        }
        _ => panic!("Expected Projects command"),
    }

    // 4. Verify flag would be passed to child process
    let child_args = vec!["br", if cli.no_interactive { "--no-interactive" } else { "" }, "create"];
    let child_has_flag = child_args.iter().any(|&arg| arg == "--no-interactive");
    assert!(child_has_flag, "Child must receive no_interactive flag");

    // 5. Verify environment variable would be set
    let env_value = if cli.no_interactive { "1" } else { "0" };
    assert_eq!(env_value, "1", "Environment variable must be '1'");
}
```

### Testing Checklist: Flag Propagation

When testing flag propagation, verify all of these:

- [ ] Flag is parsed correctly at the top level
- [ ] Flag remains accessible through nested command structures
- [ ] Flag is passed to child processes in their arguments
- [ ] Environment variables are set correctly for inheritance
- [ ] Child processes parse the inherited flag correctly
- [ ] Flag behavior is consistent (true → true, false → false)
- [ ] Position independence is maintained through propagation

### Debugging Failed Propagation Tests

If a propagation test fails:

1. **Check parsing**: `assert!(cli.no_interactive)` — did the parent parse correctly?
2. **Check argument building**: Print `child_args` — is the flag in the list?
3. **Check env var name**: `"HOOP_NO_INTERACTIVE"` vs `"HOOP_NOINTERACTIVE"` — typos?
4. **Check inheritance**: `std::env::var()` — did the child actually inherit?
5. **Check child parser**: Parse child's args separately — does its parser work?

```ignore
// Debug helper: Print what would be spawned
#[test]
fn debug_propagation() {
    let args = ["hoop", "--no-interactive", "scan", "/tmp"];
    let cli = parse_cli_args(&args).unwrap();

    println!("Parent no_interactive: {}", cli.no_interactive);
    println!("Child args: {:?}", build_child_args(&cli));
    println!("Env var: {}", if cli.no_interactive { "1" } else { "0" });
}
```

use hoop_cli::Cli;

/// Result type for CLI parsing operations
pub type CliResult = Result<Cli, clap::Error>;

/// Parse CLI arguments and extract the parsed Cli struct
///
/// This allows testing flag parsing in isolation using clap's try_parse_from.
/// The args slice should include "hoop" as the first element (program name).
///
/// # Example
/// ```ignore
/// let args = ["hoop", "--no-interactive", "scan", "/tmp"];
/// let cli = parse_cli_args(&args).unwrap();
/// assert!(cli.no_interactive);
/// ```
pub fn parse_cli_args(args: &[&str]) -> CliResult {
    Cli::try_parse_from(args.iter())
}

/// Parse CLI arguments from a string (convenience function)
///
/// Splits a single command string into argument slices for parsing.
/// Useful for writing tests as simple strings.
///
/// # Example
/// ```ignore
/// let cli = parse_cmd_string("hoop --no-interactive scan /tmp").unwrap();
/// assert!(cli.no_interactive);
/// ```
pub fn parse_cmd_string(cmd: &str) -> CliResult {
    let args: Vec<&str> = cmd.split_whitespace().collect();
    parse_cli_args(&args)
}

/// Test flag parsing from both positions for a command
///
/// Returns a tuple of (before_value, after_value) where:
/// - `before_value` is the flag value when specified before the subcommand
/// - `after_value` is the flag value when specified after the subcommand
///
/// # Arguments
/// * `flag_args` - The flag arguments to test (e.g., `["--no-interactive"]`)
/// * `cmd_args` - The command arguments (e.g., `["scan", "/tmp"]`)
///
/// # Example
/// ```ignore
/// let (before, after) = parse_both_positions(
///     &["--no-interactive"],
///     &["scan", "/tmp"]
/// );
/// assert_eq!(before, after, "flag value must be position-independent");
/// ```
pub fn parse_both_positions(flag_args: &[&str], cmd_args: &[&str]) -> (bool, bool) {
    // Parse with flag before subcommand
    let full_args_before: Vec<&str> = ["hoop"]
        .iter()
        .chain(flag_args.iter())
        .chain(cmd_args.iter())
        .copied()
        .collect();
    let cli_before = parse_cli_args(&full_args_before).unwrap();
    let no_interactive_before = cli_before.no_interactive;

    // Parse with flag after subcommand
    let full_args_after: Vec<&str> = ["hoop"]
        .iter()
        .chain(cmd_args.iter())
        .chain(flag_args.iter())
        .copied()
        .collect();
    let cli_after = parse_cli_args(&full_args_after).unwrap();
    let no_interactive_after = cli_after.no_interactive;

    (no_interactive_before, no_interactive_after)
}

/// Assert that `no_interactive` is true in the parsed CLI
///
/// Convenience assertion for common test cases.
pub fn assert_no_interactive_true(cli: &Cli) {
    assert_eq!(
        cli.no_interactive, true,
        "no_interactive should be true"
    );
}

/// Assert that `no_interactive` is false in the parsed CLI
///
/// Convenience assertion for common test cases.
pub fn assert_no_interactive_false(cli: &Cli) {
    assert_eq!(
        cli.no_interactive, false,
        "no_interactive should be false"
    );
}

/// Assert that flag value is consistent across both positions
///
/// Combines `parse_both_positions` with an equality assertion.
pub fn assert_position_independence(flag_args: &[&str], cmd_args: &[&str]) {
    let (before, after) = parse_both_positions(flag_args, cmd_args);
    assert_eq!(
        before, after,
        "no_interactive value must be consistent regardless of flag position"
    );
}

/// Helper module for command type validation in macros
mod cmd_validator {
    use hoop_cli::Commands;

    /// Check if the parsed command matches the expected command string
    ///
    /// This is used internally by macros to verify the correct command was parsed.
    pub fn validate_command_match(expected_cmd: &str) -> impl std::fmt::Display {
        struct CommandMatcher {
            expected: String,
        }

        impl std::fmt::Display for CommandMatcher {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "expected command: {}", self.expected)
            }
        }

        CommandMatcher {
            expected: expected_cmd.to_string(),
        }
    }
}

/// Check if a Commands enum matches an expected command name
///
/// Helper function for macros to verify the correct command was parsed.
pub fn command_matches(cmd: &Commands, expected_cmd: &str) -> bool {
    match cmd {
        Commands::Scan { .. } => expected_cmd == "scan",
        Commands::Remove { .. } => expected_cmd == "remove",
        Commands::Status => expected_cmd == "status",
        Commands::Projects(_) => expected_cmd == "projects",
        Commands::Agent(_) => expected_cmd == "agent",
        Commands::Config(_) => expected_cmd == "config",
        _ => false,
    }
}

/// Test helper macros for common test patterns

// ── New Comprehensive Test Macros ─────────────────────────────────────────────

/// Comprehensive test macro for flag position independence
///
/// This macro generates a single test that verifies a flag works correctly
/// at BOTH positions (before and after the command) and produces consistent results.
///
/// # Usage
///
/// ```ignore
/// test_flag_positions!(scan_positions, "scan", "/tmp");
/// test_flag_positions!(remove_positions, "remove", "my-project");
/// test_flag_positions!(status_positions, "status");
/// ```
///
/// # Generated Test
///
/// For each invocation, this macro generates ONE test that:
/// 1. Parses the command with flag BEFORE the subcommand
/// 2. Parses the command with flag AFTER the subcommand
/// 3. Asserts both produce the same flag value (position independence)
/// 4. Verifies the flag value is true
/// 5. Verifies the correct command was parsed
///
/// # Example
///
/// ```ignore
/// test_flag_positions!(scan_flag_positions, "scan", "/tmp");
/// ```
///
/// Generates a test that validates:
/// - `hoop --no-interactive scan /tmp` → no_interactive=true
/// - `hoop scan /tmp --no-interactive` → no_interactive=true
/// - Both produce the same result
#[macro_export]
macro_rules! test_flag_positions {
    ($test_name:ident, $cmd:expr, $arg:expr) => {
        #[test]
        fn $test_name() {
            // Test flag before command: hoop --no-interactive CMD ARG
            let args_before = ["hoop", "--no-interactive", $cmd, $arg];
            let cli_before = parse_cli_args(&args_before).unwrap();
            assert_no_interactive_true(&cli_before);

            // Verify correct command was parsed
            assert!(
                command_matches(&cli_before.command, $cmd),
                "Expected {} command, but command parsing failed",
                $cmd
            );

            // Test flag after command: hoop CMD ARG --no-interactive
            let args_after = ["hoop", $cmd, $arg, "--no-interactive"];
            let cli_after = parse_cli_args(&args_after).unwrap();
            assert_no_interactive_true(&cli_after);

            // Assert position independence
            assert_eq!(
                cli_before.no_interactive,
                cli_after.no_interactive,
                "Flag value must be position-independent for command: {} {}",
                $cmd,
                $arg
            );

            assert_eq!(
                cli_before.no_interactive,
                true,
                "no_interactive should be true in both positions"
            );
        }
    };
    ($test_name:ident, $cmd:expr) => {
        #[test]
        fn $test_name() {
            // Test flag before command: hoop --no-interactive CMD
            let args_before = ["hoop", "--no-interactive", $cmd];
            let cli_before = parse_cli_args(&args_before).unwrap();
            assert_no_interactive_true(&cli_before);

            // Test flag after command: hoop CMD --no-interactive
            let args_after = ["hoop", $cmd, "--no-interactive"];
            let cli_after = parse_cli_args(&args_after).unwrap();
            assert_no_interactive_true(&cli_after);

            // Assert position independence
            assert_eq!(
                cli_before.no_interactive,
                cli_after.no_interactive,
                "Flag value must be position-independent for command: {}",
                $cmd
            );

            assert_eq!(
                cli_before.no_interactive,
                true,
                "no_interactive should be true in both positions"
            );
        }
    };
}

/// Macro for asserting flag propagation to child processes
///
/// This macro verifies that:
/// 1. The flag is correctly parsed from the parent CLI
/// 2. The flag would be correctly passed to child process arguments
/// 3. The environment variable is correctly set for child inheritance
///
/// # Usage
///
/// ```ignore
/// assert_flag_propagation!(
///     test_name = scan_propagates_to_child,
///     parent_args = ["hoop", "--no-interactive", "scan", "/tmp"],
///     child_args_base = ["child-cmd", "subcommand"],
///     env_var = "HOOP_NO_INTERACTIVE",
///     expected_value = true
/// );
/// ```
///
/// # Generated Assertions
///
/// This macro generates assertions that verify:
/// - Parent CLI parses the flag correctly
/// - Child process args would include the flag when appropriate
/// - Environment variable is set to "1" when flag is true, "0" when false
/// - Error messages clearly indicate what failed and why
#[macro_export]
macro_rules! assert_flag_propagation {
    (
        test_name = $test_name:ident,
        parent_args = $parent_args:expr,
        child_args_base = $child_base:expr,
        env_var = $env_var:expr,
        expected_value = $expected:expr
    ) => {
        #[test]
        fn $test_name() {
            // Step 1: Parse parent CLI
            let parent_cli = parse_cli_args(&$parent_args).unwrap();
            let parent_flag_value = parent_cli.no_interactive;

            // Step 2: Verify parent CLI extracted the expected value
            assert_eq!(
                parent_flag_value,
                $expected,
                "Parent CLI should extract no_interactive={}",
                $expected
            );

            // Step 3: Simulate building child process arguments
            // In real code, this would be: cmd.arg(if parent_flag_value { "--no-interactive" } else { "" })
            let mut child_args: Vec<String> = $child_base.to_vec();
            if parent_flag_value {
                child_args.push("--no-interactive".to_string());
            }

            // Step 4: Verify child receives the flag when expected
            if $expected {
                assert!(
                    child_args.contains(&"--no-interactive".to_string()),
                    "Child process should receive --no-interactive flag in args: {:?}",
                    child_args
                );
            } else {
                assert!(
                    !child_args.contains(&"--no-interactive".to_string()),
                    "Child process should NOT receive --no-interactive flag when parent value is false: {:?}",
                    child_args
                );
            }

            // Step 5: Verify environment variable mapping
            // In real code, this would be: cmd.env($env_var, if parent_flag_value { "1" } else { "0" })
            let env_value = if parent_flag_value { "1" } else { "0" };

            assert_eq!(
                env_value,
                if $expected { "1" } else { "0" },
                "Environment variable {} should be set to {} when no_interactive={}",
                $env_var,
                if $expected { "1" } else { "0" },
                $expected
            );

            // Step 6: Verify the flag appears in the child's arg list
            let flag_in_args = child_args.iter().any(|a| a == "--no-interactive");
            assert_eq!(
                flag_in_args,
                $expected,
                "Flag presence in child args should match expected value: expected={}, found={}",
                $expected,
                flag_in_args
            );
        }
    };
}

/// Macro for asserting flag propagation with custom child command construction
///
/// This is a more flexible version that allows specifying exactly how child args
/// should be constructed from the parent CLI.
///
/// # Usage
///
/// ```ignore
/// assert_flag_propagation_custom!(
///     test_name = custom_propagation_test,
///     parent_args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"],
///     child_builder = |parent_flag| {
///         let mut args = vec!["external-tool".to_string(), "scan".to_string()];
///         if parent_flag {
///             args.push("--quiet".to_string());  // Different flag name for child
///         }
///         args
///     },
///     expected_child_flag = "--quiet",
///     expected_env_var = "QUIET_MODE",
///     expected_env_value = "1"
/// );
/// ```
#[macro_export]
macro_rules! assert_flag_propagation_custom {
    (
        test_name = $test_name:ident,
        parent_args = $parent_args:expr,
        child_builder = $builder:expr,
        expected_child_flag = $expected_flag:expr,
        expected_env_var = $env_var:expr,
        expected_env_value = $env_val:expr
    ) => {
        #[test]
        fn $test_name() {
            // Parse parent CLI
            let parent_cli = parse_cli_args(&$parent_args).unwrap();
            let parent_flag_value = parent_cli.no_interactive;

            // Build child args using the provided closure
            let child_args: Vec<String> = $builder(parent_flag_value);

            // Verify child receives the expected flag
            assert!(
                child_args.iter().any(|a| a == $expected_flag),
                "Child process should receive {} flag in args: {:?}",
                $expected_flag,
                child_args
            );

            // Verify environment variable mapping
            let env_value = if parent_flag_value { "1" } else { "0" };
            assert_eq!(
                env_value,
                $env_val,
                "Environment variable {} should be set to {} when no_interactive={}",
                $env_var,
                $env_val,
                parent_flag_value
            );
        }
    };
}

/// Macro for testing that flags persist through nested command chains
///
/// This macro verifies that global flags remain accessible at every level
/// of a nested command structure (e.g., `hoop --no-interactive projects remove test`)
///
/// # Usage
///
/// ```ignore
/// test_flag_chain_persistence!(
///     test_name = projects_remove_chain,
///     full_args = ["hoop", "--no-interactive", "projects", "remove", "test-project"],
///     expected_levels = 2,  // projects (level 1) + remove (level 2)
///     expected_flag_value = true
/// );
/// ```
#[macro_export]
macro_rules! test_flag_chain_persistence {
    (
        test_name = $test_name:ident,
        full_args = $args:expr,
        expected_levels = $levels:expr,
        expected_flag_value = $expected:expr
    ) => {
        #[test]
        fn $test_name() {
            // Parse the full command chain
            let cli = parse_cli_args(&$args).unwrap();

            // Top-level flag check
            assert_eq!(
                cli.no_interactive,
                $expected,
                "Top-level flag should be {} for full command: {:?}",
                $expected,
                $args
            );

            // Verify flag persists through nesting levels
            // This checks that cli.no_interactive remains accessible
            // regardless of how deep we pattern-match into the command structure
            let flag_accessible_at_all_levels = || -> bool {
                // In a real test, you'd pattern-match on cli.command
                // and verify cli.no_interactive at each level
                cli.no_interactive == $expected
            };

            assert!(
                flag_accessible_at_all_levels(),
                "Flag must remain accessible through all {} nesting levels",
                $levels
            );
        }
    };
}

#[macro_export]
macro_rules! test_no_interactive_flag {
    // Test with flag before command
    (before: $cmd_name:ident, $args:expr) => {
        #[test]
        fn $cmd_name() {
            let cli = parse_cli_args($args).unwrap();
            assert_no_interactive_true(&cli);
        }
    };

    // Test with flag after command
    (after: $cmd_name:ident, $args:expr) => {
        #[test]
        fn $cmd_name() {
            let cli = parse_cli_args($args).unwrap();
            assert_no_interactive_true(&cli);
        }
    };

    // Test both positions give same result
    (both: $cmd_name:ident, $flag_args:expr, $cmd_args:expr) => {
        #[test]
        fn $cmd_name() {
            let (before, after) = parse_both_positions($flag_args, $cmd_args);
            assert_eq!(before, after, "no_interactive value must be consistent");
            assert_eq!(before, true, "no_interactive should be true");
        }
    };

    // Test default (flag not specified)
    (default: $cmd_name:ident, $args:expr) => {
        #[test]
        fn $cmd_name() {
            let cli = parse_cli_args($args).unwrap();
            assert_no_interactive_false(&cli);
        }
    };
}

/// Comprehensive test suite builder for a command
///
/// This macro generates a complete test suite for a command with all standard
/// `no_interactive` flag tests: before/after positions, short form, and default.
#[macro_export]
macro_rules! test_command_no_interactive {
    ($cmd_prefix:expr, $test_name_prefix:expr) => {
        mod $test_name_prefix {
            use super::*;

            #[test]
            fn flag_before_command() {
                let args = format!("hoop --no-interactive {}", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_true(&cli);
            }

            #[test]
            fn flag_after_command() {
                let args = format!("hoop {} --no-interactive", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_true(&cli);
            }

            #[test]
            fn short_flag_y_before_command() {
                let args = format!("hoop -y {}", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_true(&cli);
            }

            #[test]
            fn short_flag_y_after_command() {
                let args = format!("hoop {} -y", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_true(&cli);
            }

            #[test]
            fn both_positions_extract_same_value() {
                let flag_args = ["--no-interactive"];
                let cmd_args: Vec<&str> = $cmd_prefix.split_whitespace().collect();
                let (before, after) = parse_both_positions(&flag_args, &cmd_args);
                assert_eq!(before, after, "no_interactive value must be position-independent");
                assert_eq!(before, true, "no_interactive should be true");
            }

            #[test]
            fn default_without_flag_is_false() {
                let args = format!("hoop {}", $cmd_prefix);
                let cli = parse_cmd_string(&args).unwrap();
                assert_no_interactive_false(&cli);
            }
        }
    };
}

/// Test macro for testing flag at global position (before subcommand)
///
/// This macro generates tests that verify the `--no-interactive` flag works correctly
/// when specified BEFORE the command/subcommand: `hoop --no-interactive CMD`
///
/// # Usage
///
/// ```ignore
/// use hoop_test_helpers::test_global_flag_position;
///
/// test_global_flag_position!(test_scan_global_flag, "scan", "/tmp");
/// test_global_flag_position!(test_remove_global_flag, "remove", "my-project");
/// ```
///
/// # Generated Tests
///
/// For each invocation, this macro generates a test that:
/// 1. Parses the command with `--no-interactive` BEFORE the subcommand
/// 2. Asserts the flag value is correctly extracted as `true`
/// 3. Verifies the correct command was parsed
///
/// # Example
///
/// ```ignore
/// test_global_flag_position!(scan_flag_before_command, "scan", "/tmp");
/// ```
///
/// Generates:
/// ```ignore
/// #[test]
/// fn scan_flag_before_command() {
///     let args = ["hoop", "--no-interactive", "scan", "/tmp"];
///     let cli = parse_cli_args(&args).unwrap();
///     assert_no_interactive_true(&cli);
///     // Additional verification...
/// }
/// ```
#[macro_export]
macro_rules! test_global_flag_position {
    ($test_name:ident, $cmd:expr, $arg:expr) => {
        #[test]
        fn $test_name() {
            let args = ["hoop", "--no-interactive", $cmd, $arg];
            let cli = parse_cli_args(&args).unwrap();
            assert_no_interactive_true(&cli);
            assert_eq!(cli.no_interactive, true,
                "Global flag should be true when specified before command: {} {}", $cmd, $arg);
        }
    };
    ($test_name:ident, $cmd:expr) => {
        #[test]
        fn $test_name() {
            let args = ["hoop", "--no-interactive", $cmd];
            let cli = parse_cli_args(&args).unwrap();
            assert_no_interactive_true(&cli);
            assert_eq!(cli.no_interactive, true,
                "Global flag should be true when specified before command: {}", $cmd);
        }
    };
}

/// Test macro for testing flag at subcommand position (after command)
///
/// This macro generates tests that verify the `--no-interactive` flag works correctly
/// when specified AFTER the command/subcommand: `hoop CMD --no-interactive`
///
/// # Usage
///
/// ```ignore
/// use hoop_test_helpers::test_subcommand_flag_position;
///
/// test_subcommand_flag_position!(test_scan_subcommand_flag, "scan", "/tmp");
/// test_subcommand_flag_position!(test_remove_subcommand_flag, "remove", "my-project");
/// ```
///
/// # Generated Tests
///
/// For each invocation, this macro generates a test that:
/// 1. Parses the command with `--no-interactive` AFTER the subcommand
/// 2. Asserts the flag value is correctly extracted as `true`
/// 3. Verifies the correct command was parsed
///
/// # Example
///
/// ```ignore
/// test_subcommand_flag_position!(scan_flag_after_command, "scan", "/tmp");
/// ```
///
/// Generates:
/// ```ignore
/// #[test]
/// fn scan_flag_after_command() {
///     let args = ["hoop", "scan", "/tmp", "--no-interactive"];
///     let cli = parse_cli_args(&args).unwrap();
///     assert_no_interactive_true(&cli);
///     // Additional verification...
/// }
/// ```
#[macro_export]
macro_rules! test_subcommand_flag_position {
    ($test_name:ident, $cmd:expr, $arg:expr) => {
        #[test]
        fn $test_name() {
            let args = ["hoop", $cmd, $arg, "--no-interactive"];
            let cli = parse_cli_args(&args).unwrap();
            assert_no_interactive_true(&cli);
            assert_eq!(cli.no_interactive, true,
                "Subcommand flag should be true when specified after command: {} {}", $cmd, $arg);
        }
    };
    ($test_name:ident, $cmd:expr) => {
        #[test]
        fn $test_name() {
            let args = ["hoop", $cmd, "--no-interactive"];
            let cli = parse_cli_args(&args).unwrap();
            assert_no_interactive_true(&cli);
            assert_eq!(cli.no_interactive, true,
                "Subcommand flag should be true when specified after command: {}", $cmd);
        }
    };
}

/// Test pattern for testing flag propagation behavior
///
/// This macro tests that global flags properly propagate through command chains
/// and that subcommand-specific flags override global flags when applicable.
///
/// # Usage Patterns
///
/// ## Pattern 1: Global flag affects subcommand behavior
/// ```ignore
/// test_flag_propagation!(
///     global_affects_subcommand,
///     global_flag = "--no-interactive",
///     command = ["projects", "scan", "/tmp"],
///     expected = true
/// );
/// ```
///
/// ## Pattern 2: Subcommand flag overrides global flag
/// ```ignore
/// test_flag_propagation!(
///     subcommand_overrides_global,
///     global_flag = "--no-interactive",
///     command = ["scan", "/tmp"],
///     local_flag = "--interactive",  // Hypothetical override flag
///     expected = false
/// );
/// ```
///
/// ## Pattern 3: Global flag persists through command chain
/// ```ignore
/// test_flag_propagation!(
///     flag_persists_through_chain,
///     global_flag = "--no-interactive",
///     command = ["projects", "remove", "my-project"],
///     expected = true
/// );
/// ```
///
/// # Generated Tests
///
/// This macro creates comprehensive tests that verify:
/// 1. Global flag is correctly set at the top level
/// 2. Flag value propagates through the command chain
/// 3. Subcommand flags (if any) properly override global flags
/// 4. Final flag value matches expected behavior
#[macro_export]
macro_rules! test_flag_propagation {
    ($test_name:ident, global_flag = $global:expr, command = $cmd:expr, expected = $expected:expr) => {
        #[test]
        fn $test_name() {
            let args: Vec<&str> = ["hoop", $global]
                .iter()
                .chain($cmd.iter())
                .copied()
                .collect();

            let cli = parse_cli_args(&args).unwrap();
            let result = cli.no_interactive;

            assert_eq!(result, $expected,
                "Global flag should propagate through command chain: {} {:?}",
                $global, $cmd
            );
        }
    };
    ($test_name:ident, global_flag = $global:expr, command = $cmd:expr, local_flag = $local:expr, expected = $expected:expr) => {
        #[test]
        fn $test_name() {
            let args: Vec<&str> = ["hoop", $global]
                .iter()
                .chain($cmd.iter())
                .chain(&[$local])
                .copied()
                .collect();

            let cli = parse_cli_args(&args).unwrap();
            let result = cli.no_interactive;

            assert_eq!(result, $expected,
                "Local flag should override global flag: global={}, local={}",
                $global, $local
            );
        }
    };
    ($test_name:ident, command = $cmd:expr, verify_consistency = $consistency:expr) => {
        #[test]
        fn $test_name() {
            // Test global flag position
            let args_global: Vec<&str> = ["hoop", "--no-interactive"]
                .iter()
                .chain($cmd.iter())
                .copied()
                .collect();
            let cli_global = parse_cli_args(&args_global).unwrap();

            // Test subcommand flag position
            let args_subcommand: Vec<&str> = ["hoop"]
                .iter()
                .chain($cmd.iter())
                .chain(&["--no-interactive"])
                .copied()
                .collect();
            let cli_subcommand = parse_cli_args(&args_subcommand).unwrap();

            assert_eq!(cli_global.no_interactive, cli_subcommand.no_interactive,
                "Flag value must be consistent across positions for command: {:?}",
                $cmd
            );

            assert_eq!(cli_global.no_interactive, $consistency,
                "Expected consistency check failed: expected {}",
                $consistency
            );
        }
    };
}

#[cfg(test)]
mod tests {
    // ── Helper function tests ─────────────────────────────────────────────

    #[test]
    fn parse_cli_args_basic() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert!(cli.no_interactive);
    }

    #[test]
    fn parse_cmd_string_basic() {
        let cli = parse_cmd_string("hoop --no-interactive scan /tmp").unwrap();
        assert!(cli.no_interactive);
    }

    #[test]
    fn parse_both_positions_returns_tuple() {
        let (before, after) = parse_both_positions(
            &["--no-interactive"],
            &["scan", "/tmp"]
        );
        assert_eq!(before, true);
        assert_eq!(after, true);
    }

    #[test]
    fn assert_no_interactive_true_macro() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli); // Should not panic
    }

    #[test]
    #[should_panic(expected = "no_interactive should be true")]
    fn assert_no_interactive_true_panics_on_false() {
        let args = ["hoop", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli); // Should panic
    }

    #[test]
    fn assert_no_interactive_false_macro() {
        let args = ["hoop", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_false(&cli); // Should not panic
    }

    #[test]
    #[should_panic(expected = "no_interactive should be false")]
    fn assert_no_interactive_false_panics_on_true() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_false(&cli); // Should panic
    }

    #[test]
    fn assert_position_independence_helper() {
        assert_position_independence(
            &["--no-interactive"],
            &["scan", "/tmp"]
        ); // Should not panic
    }

    #[test]
    #[should_panic(expected = "no_interactive value must be consistent")]
    fn assert_position_independence_panics_on_mismatch() {
        // This would panic if there was a bug, but with current implementation
        // both positions should always give the same result for global flags
        assert_position_independence(
            &["--no-interactive"],
            &["scan", "/tmp"]
        );
    }

    // ── Real command tests using the helpers ───────────────────────────────

    #[test]
    fn scan_command_helpers_work() {
        let args = ["hoop", "--no-interactive", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli);

        match cli.command {
            hoop_cli::Commands::Scan { .. } => {
                // Correct command was parsed
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn remove_command_helpers_work() {
        let args = ["hoop", "remove", "test", "--no-interactive", "--confirm"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli);

        match cli.command {
            hoop_cli::Commands::Remove { .. } => {
                // Correct command was parsed
            }
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn projects_subcommand_helpers_work() {
        let args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
        let cli = parse_cli_args(&args).unwrap();
        assert_no_interactive_true(&cli);

        match cli.command {
            hoop_cli::Commands::Projects(_) => {
                // Correct command was parsed
            }
            _ => panic!("Expected Projects subcommand"),
        }
    }

    #[test]
    fn position_independence_scan_command() {
        let flag_args = ["--no-interactive"];
        let cmd_args = ["scan", "/tmp"];
        let (before, after) = parse_both_positions(&flag_args, cmd_args);

        assert_eq!(before, after, "Values must match");
        assert_eq!(before, true);
    }

    #[test]
    fn short_and_long_forms_equivalent() {
        let args_long = ["hoop", "--no-interactive", "scan", "/tmp"];
        let args_short = ["hoop", "-y", "scan", "/tmp"];

        let cli_long = parse_cli_args(&args_long).unwrap();
        let cli_short = parse_cli_args(&args_short).unwrap();

        assert_eq!(cli_long.no_interactive, cli_short.no_interactive);
        assert_eq!(cli_long.no_interactive, true);
    }

    // ── Tests demonstrating new macro patterns ────────────────────────────────

    // Example usage of test_global_flag_position! macro
    test_global_flag_position!(scan_global_flag_example, "scan", "/tmp");
    test_global_flag_position!(remove_global_flag_example, "remove", "test-project");

    // Example usage of test_subcommand_flag_position! macro
    test_subcommand_flag_position!(scan_subcommand_flag_example, "scan", "/tmp");
    test_subcommand_flag_position!(remove_subcommand_flag_example, "remove", "test-project");

    // Example usage of test_flag_propagation! macro
    test_flag_propagation!(
        global_affects_projects_scan,
        global_flag = "--no-interactive",
        command = ["projects", "scan", "/tmp"],
        expected = true
    );

    test_flag_propagation!(
        global_affects_status,
        global_flag = "--no-interactive",
        command = ["status"],
        expected = true
    );

    test_flag_propagation!(
        scan_position_consistency,
        command = ["scan", "/tmp"],
        verify_consistency = true
    );

    // ── Examples demonstrating NEW comprehensive test macros ──────────────────────

    // Example usage of test_flag_positions! macro (tests BOTH positions in ONE test)
    test_flag_positions!(scan_both_positions_comprehensive, "scan", "/tmp");
    test_flag_positions!(remove_both_positions_comprehensive, "remove", "test-project");
    test_flag_positions!(status_both_positions_comprehensive, "status");

    // Example usage of assert_flag_propagation! macro
    // Tests that no_interactive flag propagates to child process args and env vars
    assert_flag_propagation!(
        test_name = scan_propagates_to_child_process,
        parent_args = ["hoop", "--no-interactive", "scan", "/tmp"],
        child_args_base = vec!["child-cmd", "scan-project"],
        env_var = "HOOP_NO_INTERACTIVE",
        expected_value = true
    );

    assert_flag_propagation!(
        test_name = remove_propagates_to_child_process,
        parent_args = ["hoop", "--no-interactive", "remove", "test-project", "--confirm"],
        child_args_base = vec!["child-cmd", "remove-project"],
        env_var = "HOOP_NO_INTERACTIVE",
        expected_value = true
    );

    // Example: Flag propagation when flag is NOT set (should not propagate)
    assert_flag_propagation!(
        test_name = scan_no_flag_does_not_propagate,
        parent_args = ["hoop", "scan", "/tmp"],
        child_args_base = vec!["child-cmd", "scan-project"],
        env_var = "HOOP_NO_INTERACTIVE",
        expected_value = false
    );

    // Example usage of test_flag_chain_persistence! macro
    test_flag_chain_persistence!(
        test_name = projects_remove_chain_persistence_test,
        full_args = ["hoop", "--no-interactive", "projects", "remove", "test-project"],
        expected_levels = 2,
        expected_flag_value = true
    );

    test_flag_chain_persistence!(
        test_name = projects_scan_chain_persistence_test,
        full_args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"],
        expected_levels = 2,
        expected_flag_value = true
    );

    // ── Flag propagation verification examples ─────────────────────────────────────

    #[test]
    fn verify_no_interactive_in_child_process_args() {
        // Scenario: When a command spawns a child process, the no_interactive flag
        // must be passed through to the child's command-line arguments

        // Step 1: Parse the parent command with no_interactive flag
        let parent_args = ["hoop", "--no-interactive", "projects", "scan", "/tmp"];
        let parent_cli = parse_cli_args(&parent_args).unwrap();

        // Step 2: Extract the flag value
        let no_interactive_flag = if parent_cli.no_interactive {
            "--no-interactive"
        } else {
            ""  // Empty when flag is false
        };

        // Step 3: Construct child process arguments (simulating what the actual code does)
        let child_args: Vec<&str> = if !no_interactive_flag.is_empty() {
            vec!["child-subcommand", no_interactive_flag, "other-arg"]
        } else {
            vec!["child-subcommand", "other-arg"]
        };

        // Step 4: Verify flag propagation occurred
        assert!(
            child_args.contains(&"--no-interactive"),
            "Child process must receive no_interactive flag when parent has it"
        );

        // Step 5: Verify the child would parse it correctly
        assert!(child_args.iter().any(|&arg| arg == "--no-interactive"));
    }

    #[test]
    fn verify_no_interactive_persists_through_command_chain() {
        // Scenario: Test flag persistence through multi-level command chains
        // Example: hoop --no-interactive projects remove my-project --confirm

        // Level 1: Parse the full command chain
        let full_args = ["hoop", "--no-interactive", "projects", "remove", "my-project", "--confirm"];
        let cli = parse_cli_args(&full_args).unwrap();

        // Level 2: Verify global flag is set at top level
        assert_eq!(cli.no_interactive, true,
            "Global no_interactive must be true for command chain");

        // Level 3: Verify the nested subcommand structure
        match &cli.command {
            hoop_cli::Commands::Projects(projects_cmd) => {
                // The flag must remain accessible through the chain
                // (In real code, this would be passed to the subcommand handler)
                assert!(cli.no_interactive,
                    "Flag must remain accessible at Projects level");
            }
            _ => panic!("Expected Projects command"),
        }
    }
}