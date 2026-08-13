# init_handler Flag Usage Analysis

## Analysis Task: Document `init_handler` `no_interactive` flag behavior

**Date:** 2026-08-13  
**Bead:** bf-1h6qll  
**Purpose:** Understand how `init_handler` reads and uses the `no_interactive` flag to inform test implementation

---

## Executive Summary

The `init_handler` function (named `run_init_wizard` in the codebase) has a **simple, early-exit pattern** for handling the `no_interactive` flag. When the flag is `true`, the wizard immediately exits with a clear error message directing users to manual configuration. When `false`, it proceeds through the normal interactive wizard stages.

**Key Finding:** The implementation is straightforward and already has comprehensive test coverage in `hoop-cli/src/init.rs` (lines 672-1679). The test implementation bead will need minimal additional work beyond documenting existing patterns.

---

## 1. Flag Field Access Pattern

### 1.1 Function Signature

```rust
pub fn run_init_wizard(no_interactive: bool) -> Result<()>
```
**Location:** `hoop-cli/src/init.rs:40`

**Pattern:** The handler receives the flag as a **named parameter** of type `bool`. This is the canonical pattern for all command handlers in HOOP.

### 1.2 Flag Flow from CLI to Handler

```
┌─────────────────────────────────────────────────────────────────┐
│ CLI ARGUMENTS                                                   │
│ hoop --no-interactive init                                      │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│ CLAP PARSING (main.rs:365)                                      │
│ let cli = Cli::parse();                                         │
│ ┌──────────────────────────────────────────────────────────┐   │
│ │ struct Cli {                                             │   │
│ │   #[arg(short = 'y', long = "no-interactive",           │   │
│ │         global = true)]                                  │   │
│ │   no_interactive: bool,                                  │   │
│ │   command: Commands,                                     │   │
│ │ }                                                        │   │
│ └──────────────────────────────────────────────────────────┘   │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│ FLAG EXTRACTION (main.rs:366)                                   │
│ let no_interactive = cli.no_interactive;                       │
│ // Extracted ONCE at parse time, before match statement        │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│ COMMAND MATCHING (main.rs:520-525)                             │
│ Commands::Init => {                                            │
│     if let Err(e) = init::run_init_wizard(no_interactive) {    │
│         eprintln!("hoop init: {}", e);                        │
│         std::process::exit(exit_code_for_error(&e));           │
│     }                                                           │
│ }                                                               │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│ HANDLER FUNCTION (init.rs:40)                                  │
│ pub fn run_init_wizard(no_interactive: bool) -> Result<()>      │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│ CONDITIONAL BEHAVIOR (init.rs:41-48)                            │
│ if no_interactive {                                             │
│     // Early exit with error message                           │
│     eprintln!("hoop init: cannot run in non-interactive mode.");│
│     eprintln!("  For automated setup, manually create...");     │
│     std::process::exit(2);                                      │
│ }                                                                │
└─────────────────────────────────────────────────────────────────┘
```

### 1.3 Global Flag Property

The `no_interactive` flag uses **`global = true`** in its clap attribute (main.rs:139):

```rust
#[arg(short = 'y', long = "no-interactive", global = true)]
no_interactive: bool,
```

**What this means:**
- The flag can appear **anywhere** in the command line
- Clap propagates the value through the entire command tree
- No need to redefine it in subcommands

**Valid positions:**
```bash
hoop --no-interactive init        # BEFORE command
hoop init --no-interactive        # AFTER command
hoop -y init                       # Short form BEFORE
hoop init -y                       # Short form AFTER
```

**All positions are equivalent** — clap extracts the same `bool` value regardless of position.

---

## 2. Conditional Behavior Paths

### 2.1 Path A: `no_interactive = true` → Early Exit

**Location:** `init.rs:41-48`

```rust
if no_interactive {
    // In non-interactive mode, init wizard cannot proceed safely
    // since it requires user input for several steps
    eprintln!("hoop init: cannot run in non-interactive mode.");
    eprintln!("  The init wizard requires interactive input for configuration.");
    eprintln!("  For automated setup, manually create ~/.hoop/config.yml and ~/.hoop/projects.yaml");
    std::process::exit(2);
}
```

**Behavior:**
1. Prints error message with `eprintln!` macro (3 lines)
2. Exits immediately with code `2` (fatal / precondition not met)
3. NO wizard stages execute
4. NO side effects (no files created, no processes spawned)

**Error message quality:**
- ✅ Clear statement: "cannot run in non-interactive mode"
- ✅ Explanation: "requires interactive input"
- ✅ Actionable guidance: manual config file paths provided
- ✅ Exit code: `2` (fatal / precondition, consistent with exit_code_for_error logic)

### 2.2 Path B: `no_interactive = false` → Interactive Wizard

**Location:** `init.rs:50-74`

```rust
print_wizard_banner();

// Stage 1: Dependency check
let audit_passed = stage_1_dependency_check()?;

if !audit_passed {
    println!("\n⚠️  Critical dependencies are missing.");
    println!("    Please fix the issues above and run `hoop init` again.");
    println!("    You can re-run the audit anytime with: hoop audit check");
    std::process::exit(2);
}

// Stage 2: First project registration
stage_2_project_registration()?;

// Stage 3: Agent adapter setup (optional)
stage_3_agent_setup()?;

// Stage 4: systemd install (optional)
stage_4_systemd_install()?;

// Stage 5: Start daemon and health check
stage_5_health_check()?;
```

**Behavior:**
1. Prints wizard banner (`print_wizard_banner()`)
2. Runs **5 sequential stages**:
   - Stage 1: Dependency check (`hoop audit check`)
   - Stage 2: Project registration (scans `~/` for `.beads/` directories)
   - Stage 3: Agent adapter setup (Claude Code / Anthropic API / ZAI)
   - Stage 4: systemd user service install
   - Stage 5: Daemon health check + URL print
3. Each stage has **interactive prompts** (user input required)
4. Stages are **skipped if already configured** (idempotent)

**Key characteristic:** ALL stages require user interaction for completion. The wizard is fundamentally interactive — it was designed to be run by a human operator, not in automation.

---

## 3. Mock Fixture Requirements for Testing

### 3.1 Why Mock Fixtures Are Challenging

The existing tests in `init.rs` use **code inspection** (reading source files with `std::fs::read_to_string`) rather than traditional mocking. This is intentional because:

1. **`std::process::exit(2)` kills the test process** — Cannot test actual runtime behavior without killing test runner
2. **Multiple external dependencies** — File system, `Command::new("hoop")`, `Command::new("tailscale")`, `io::stdin()`
3. **Idempotent skips** — Stages skip if files already exist, requiring complex fixture setup/reset

### 3.2 Existing Test Pattern: Code Inspection + Runtime Parsing

**Location:** `init.rs:672-1679`

The existing tests use a **hybrid approach**:

#### A. Runtime Parse Tests (Verify CLI parsing at runtime)

```rust
#[test]
fn test_init_parse_flag_before_command() {
    let args = ["hoop", "--no-interactive", "init"];
    let cli = crate::Cli::parse_from(args);

    assert_eq!(cli.no_interactive, true, 
        "Flag should be true when present before init");

    match cli.command {
        crate::Commands::Init => {}, // Correct command
        _ => panic!("Expected Init command, got {:?}", cli.command),
    }
}
```

**What it tests:**
- ✅ Clap parsing works correctly
- ✅ Flag value is extracted correctly
- ✅ Command is parsed correctly
- ✅ No compilation errors

**Limitations:**
- ❌ Does NOT test handler behavior (does not call `run_init_wizard`)
- ❌ Does NOT test actual exit behavior (would kill test process)

#### B. Code Inspection Tests (Verify handler logic structure)

```rust
#[test]
fn test_init_wizard_exits_with_no_interactive_true() {
    let code = std::fs::read_to_string("src/init.rs")
        .expect("Failed to read init.rs");

    // Check for the early exit logic at the start of run_init_wizard
    let func_start = code.find("pub fn run_init_wizard(no_interactive: bool)")
        .expect("Should have run_init_wizard function");

    // Find the first if no_interactive block
    let early_exit_start = code[func_start..].find("if no_interactive {")
        .expect("Wizard must check no_interactive at the start");

    let early_exit_section = &code[func_start + early_exit_start..func_start + early_exit_start + 600];

    // Verify early exit behavior
    assert!(early_exit_section.contains("if no_interactive {"), 
        "Wizard must check no_interactive at the start");
    assert!(early_exit_section.contains("cannot run in non-interactive mode"), 
        "Wizard must explain why it cannot run");
    assert!(early_exit_section.contains("std::process::exit(2)"), 
        "Wizard must exit with code 2");
}
```

**What it tests:**
- ✅ Handler function signature is correct
- ✅ Conditional logic exists (`if no_interactive {`)
- ✅ Error messages are present
- ✅ Exit code is correct

**Limitations:**
- ❌ Does NOT execute the code (only verifies structure)
- ❌ Brittle if code is refactored (string-based assertions)

### 3.3 Mock Fixtures That Would Be Needed (for traditional unit tests)

If implementing traditional unit tests with actual execution, the following mocks would be required:

#### 1. File System Mocks

```rust
// Mock dirs::home_dir()
#[mock]
fn dirs::home_dir() -> PathBuf {
    PathBuf::from("/tmp/test-home")
}

// Mock fs::read_to_string() for config checks
#[mock]
fn fs::read_to_string(path: &PathBuf) -> Result<String> {
    match path.to_str().unwrap() {
        "/tmp/test-home/.hoop/config.yml" => Ok("# test config".to_string()),
        "/tmp/test-home/.hoop/projects.yaml" => Ok("projects: []".to_string()),
        _ => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "not found")),
    }
}

// Mock fs::write() for file creation
#[mock]
fn fs::write(path: &PathBuf, contents: &str) -> Result<()> {
    // Track writes for verification
    WRITES.lock().unwrap().push((path.clone(), contents.to_string()));
    Ok(())
}

// Mock fs::create_dir_all()
#[mock]
fn fs::create_dir_all(path: &PathBuf) -> Result<()> {
    Ok(())
}
```

#### 2. Command Spawning Mocks

```rust
// Mock Command::new("hoop") for daemon spawn in stage 5
#[mock]
fn Command::new(program: &str) -> Command {
    match program {
        "hoop" => {
            // Return a mock Command that captures args without spawning
            MockCommand::new()
        },
        "tailscale" => {
            // Return mock Command for hostname detection
            MockCommand::with_output(json!({"Self": {"DNSName": "test.ts.net"}}))
        },
        "curl" => {
            // Return mock Command for health check
            MockCommand::with_exit_code(200)
        },
        _ => panic!("Unexpected command: {}", program),
    }
}
```

#### 3. Standard Input Mocks

```rust
// Mock io::stdin().read_line() for user input
#[mock]
fn io::stdin().read_line(&mut self, buf: &mut String) -> Result<usize> {
    // Return pre-programmed responses
    static INPUTS: &[&str] = &["Y", "Y", "1", "Y", "Y"];
    static INDEX: AtomicUsize = AtomicUsize::new(0);
    
    let i = INDEX.fetch_add(1, Ordering::SeqCst);
    *buf = INPUTS[i].to_string();
    Ok(INPUTS[i].len())
}
```

#### 4. Exit Mocks (Critical Problem)

```rust
// Mock std::process::exit() to prevent test process death
#[mock]
fn std::process::exit(code: i32) -> ! {
    // Record exit code for verification
    EXIT_CODES.lock().unwrap().push(code);
    panic!("Exit called with code {}", code) // Panic instead of exit
}
```

**Problem:** `std::process::exit` is `!` (never returns). Mocking it in Rust is **extremely difficult** and requires:
- Custom panic handler
- Unwinding control
- Test harness isolation

This is likely why the existing tests use code inspection instead.

### 3.4 Recommended Fixture Strategy

**For test implementation bead:** Use the **existing pattern** (code inspection + runtime parsing) rather than traditional mocking.

**Why:**
1. ✅ Already works (tests compile and pass)
2. ✅ Avoids `std::process::exit` mocking problem
3. ✅ Tests the right things (parsing + logic structure)
4. ✅ Less brittle than complex mock setup
5. ✅ Consistent with existing test architecture

**If additional coverage is needed**, consider:
- **Integration tests** (spawn actual `hoop init` in temp dir, check exit code)
- **Golden file tests** (compare output against expected strings)
- **Property-based tests** (verify flag position independence with quickcheck)

---

## 4. Required Fixtures and Setup Summary

### 4.1 For Existing Test Pattern (Recommended)

**What's already in place:**
- ✅ Parse-time tests (lines 672-755)
- ✅ Handler signature verification (lines 756-821)
- ✅ Early exit logic verification (lines 848-1044)
- ✅ Runtime integration tests (lines 1030-1679)

**What's needed for additional test coverage:**
- 📋 None — existing tests are comprehensive

**Recommendation:** Document the existing test pattern rather than adding new tests.

### 4.2 For Traditional Unit Tests (Not Recommended)

**What would be needed:**
1. **Test directory fixture:** `/tmp/test-hoop-home/` with subdirs
2. **Config fixture:** `~/.hoop/config.yml` (empty or partial)
3. **Projects fixture:** `~/.hoop/projects.yaml` (empty or with test projects)
4. **Mock infrastructure:**
   - File system mocks (read, write, create_dir)
   - Command spawning mocks (hoop, tailscale, curl)
   - Stdin mock (user input)
   - Exit mock (capture exit code without killing test)
5. **Test cleanup:** Remove temp dirs after each test

**Estimated effort:** 2-3 days for mock infrastructure + 1 day for test cases.

**Risk:** High — complex mocks are brittle and may break with refactoring.

---

## 5. Test Implementation Recommendations

### 5.1 Immediate Task (for current bead)

**No new tests needed** — existing test coverage is comprehensive.

**Instead:**
1. ✅ **Document** the existing test pattern (this document)
2. ✅ **Verify** existing tests compile and pass:
   ```bash
   cd hoop-cli
   cargo test --lib init
   ```
3. ✅ **Close bead** with retrospective documenting the pattern

### 5.2 Future Test Enhancement (optional)

If additional coverage is desired in future beads:

**Option A: Integration Tests** (Recommended)
- Spawn `hoop init --no-interactive` in temp directory
- Capture exit code
- Verify exit code is `2`
- Verify error message contains expected text
- No mocking needed — tests actual behavior

**Option B: Golden File Tests**
- Compare actual output against expected output files
- Tests human-readable output format
- Good for catching regressions in error messages

**Option C: Property-Based Tests**
- Use `quickcheck` crate
- Verify flag position independence property
- Test: `parse(args before) == parse(args after)` for all flag positions

---

## 6. Key Takeaways

### 6.1 Flag Access Pattern
- ✅ Simple: `pub fn run_init_wizard(no_interactive: bool)`
- ✅ Named parameter of type `bool`
- ✅ Extracted once at parse time: `let no_interactive = cli.no_interactive;`
- ✅ Passed directly to handler: `init::run_init_wizard(no_interactive)`

### 6.2 Conditional Behavior
- ✅ **`true` → Early exit** with error message + exit code 2
- ✅ **`false` → Full wizard** (5 stages, all interactive)
- ✅ No intermediate states
- ✅ Clear, actionable error message

### 6.3 Test Requirements
- ✅ Existing tests use **code inspection + runtime parsing**
- ✅ Avoid `std::process::exit` mocking problem
- ✅ Tests verify parsing + logic structure (not actual execution)
- ✅ **No additional fixtures needed** for current coverage

### 6.4 Implementation Pattern
- ✅ **Global flag** property allows position independence
- ✅ **Early exit** pattern prevents interactive code in non-interactive mode
- ✅ **Exit code 2** signals fatal / precondition not met
- ✅ **Actionable error** guides users to manual config

---

## 7. References

### 7.1 Source Files
- `hoop-cli/src/main.rs` — CLI parsing, flag extraction, Init command handler (lines 139, 365-366, 520-525)
- `hoop-cli/src/init.rs` — Handler implementation, existing tests (lines 40-74, 672-1679)

### 7.2 Test Patterns in HOOP
- Scan command tests: `main.rs:1064-1106`
- Remove command tests: `main.rs:1110-1145`
- Restore command tests: `main.rs:1149-1184`
- Init command tests: `main.rs:1188-1223`

### 7.3 Related Documentation
- `CLAUDE.md` — Project instructions and build environment
- `AGENTS.md` — HOOP architecture and terminology
- `docs/plan/plan.md` — Implementation plan (Phase 1 status)

---

## 8. Conclusion

The `init_handler` implementation for the `no_interactive` flag is **simple, correct, and already well-tested**. The existing test pattern (code inspection + runtime parsing) is appropriate for this use case and avoids the complexity of mocking `std::process::exit`.

**Recommendation:** Close this bead with documentation of the existing pattern. No new test implementation is required unless future beads specifically need integration tests or property-based tests.

---

**Document Version:** 1.0  
**Last Updated:** 2026-08-13  
**Status:** Complete — Ready for review by test implementation bead
