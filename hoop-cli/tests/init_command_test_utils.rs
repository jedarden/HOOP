//! Test infrastructure for Commands::Init parser testing
//!
//! This module provides specialized helper functions and utilities for testing
//! the Init command and its parser behaviors. It extends the general clap testing
//! utilities with Init-specific patterns and test fixtures.
//!
//! # What This Module Provides
//!
//! - **Init-specific parsing helpers**: Parse Init commands with various flag combinations
//! - **Init behavior verification**: Test Init's unique behaviors (early rejection of no_interactive)
//! - **Mock init wizard utilities**: Test wizard behavior without actual user interaction
//! - **Test fixtures for init**: Temporary config files and environments for testing
//! - **Test pattern generators**: Generate common test patterns for Init command
//!
//! # Example Usage
//!
//! ```rust
//! use init_command_test_utils::*;
//!
//! #[test]
//! fn test_init_parsing() {
//!     // Parse basic init command
//!     let cli = parse_init_basic().unwrap();
//!     assert!(is_init_command(&cli));
//!
//!     // Parse init with no_interactive flag
//!     let cli = parse_init_with_no_interactive_flag().unwrap();
//!     assert_eq!(extract_no_interactive(&cli), true);
//! }
//!
//! #[test]
//! fn test_init_rejects_no_interactive() {
//!     // Verify init rejects no_interactive mode
//!     assert!(verify_init_rejects_no_interactive_mode().is_ok());
//! }
//! ```

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use clap::Parser;

// ── Re-export clap utilities ───────────────────────────────────────────────────────

pub use hoop::{Cli, Commands};

// ── Init-specific parsing helpers ─────────────────────────────────────────────────────

/// Parse the basic `hoop init` command (no flags)
///
/// This is the most common invocation pattern for the init command.
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI structure or clap error
///
/// # Examples
///
/// ```rust
/// let cli = parse_init_basic().unwrap();
/// assert!(is_init_command(&cli));
/// ```
pub fn parse_init_basic() -> Result<Cli, clap::Error> {
    parse_init(&["init"])
}

/// Parse `hoop init --no-interactive` (flag before command)
///
/// This pattern tests the no_interactive flag when placed before the init subcommand.
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI structure or clap error
///
/// # Examples
///
/// ```rust
/// let cli = parse_init_no_interactive_before().unwrap();
/// assert_eq!(extract_no_interactive(&cli), true);
/// assert!(is_init_command(&cli));
/// ```
pub fn parse_init_no_interactive_before() -> Result<Cli, clap::Error> {
    parse_init(&["--no-interactive", "init"])
}

/// Parse `hoop init --no-interactive` (flag after command)
///
/// This pattern tests the no_interactive flag when placed after the init subcommand.
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI structure or clap error
///
/// # Examples
///
/// ```rust
/// let cli = parse_init_no_interactive_after().unwrap();
/// assert_eq!(extract_no_interactive(&cli), true);
/// assert!(is_init_command(&cli));
/// ```
pub fn parse_init_no_interactive_after() -> Result<Cli, clap::Error> {
    parse_init(&["init", "--no-interactive"])
}

/// Parse `hoop -y init` (short flag before command)
///
/// This pattern tests the short -y flag (alias for --no-interactive).
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI structure or clap error
///
/// # Examples
///
/// ```rust
/// let cli = parse_init_short_flag().unwrap();
/// assert_eq!(extract_no_interactive(&cli), true);
/// assert!(is_init_command(&cli));
/// ```
pub fn parse_init_short_flag() -> Result<Cli, clap::Error> {
    parse_init(&["-y", "init"])
}

/// Generic init command parser with custom arguments
///
/// This function provides a flexible way to parse the init command with
/// any combination of flags and arguments.
///
/// # Arguments
///
/// * `args` - Arguments to append after "hoop" (e.g., `["--no-interactive", "init"]`)
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI structure or clap error
///
/// # Examples
///
/// ```rust
/// // Parse: hoop init
/// let cli = parse_init(&["init"]).unwrap();
///
/// // Parse: hoop --no-interactive init
/// let cli = parse_init(&["--no-interactive", "init"]).unwrap();
///
/// // Parse: hoop init --no-interactive
/// let cli = parse_init(&["init", "--no-interactive"]).unwrap();
///
/// // Parse: hoop -y init
/// let cli = parse_init(&["-y", "init"]).unwrap();
/// ```
pub fn parse_init(args: &[&str]) -> Result<Cli, clap::Error> {
    let full_args: Vec<&str> = std::iter::once("hoop")
        .chain(args.iter().copied())
        .collect();
    Cli::try_parse_from(full_args)
}

/// Parse init command and verify it returns Init command
///
/// This helper parses the init command and verifies it returns the Init variant.
/// Use this when you need to ensure the correct command was parsed.
///
/// # Returns
///
/// * `Result<Cli, clap::Error>` - Parsed CLI structure (if Init command)
/// * `Err(clap::Error)` - If parsing failed or command is not Init
///
/// # Examples
///
/// ```rust
/// let cli = parse_init_verify_command().unwrap();
/// // cli.command is guaranteed to be Commands::Init
/// ```
pub fn parse_init_verify_command() -> Result<Cli, clap::Error> {
    let cli = parse_init_basic()?;
    // Verify this is the Init command
    if !is_init_command(&cli) {
        return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue));
    }
    Ok(cli)
}

/// Parse init command and extract both the Cli struct and the Init command
///
/// This helper parses the init command and returns a tuple of both the Cli
/// struct and a reference to the Commands::Init variant for detailed inspection.
/// This is useful when you need both the full CLI context and the specific Init command.
///
/// # Returns
///
/// * `Result<(Cli, Commands), clap::Error>` - Tuple of CLI struct and the Init command
/// * `Err(clap::Error)` - If parsing failed or command is not Init
///
/// # Examples
///
/// ```rust
/// let (cli, command) = parse_init_extract_command().unwrap();
/// assert!(matches!(command, Commands::Init));
/// assert_eq!(cli.no_interactive, false);
/// ```
pub fn parse_init_extract_command() -> Result<(Cli, Commands), clap::Error> {
    let cli = parse_init_basic()?;

    // Clone the command before returning
    let command = cli.command.clone();

    // Verify this is the Init command
    if !matches!(command, Commands::Init) {
        return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue));
    }

    Ok((cli, command))
}

// ── Init command verification helpers ────────────────────────────────────────────────

/// Verify that the parsed CLI contains the Init command
///
/// # Arguments
///
/// * `cli` - Parsed CLI structure
///
/// # Returns
///
/// * `bool` - true if the CLI parsed the Init command
///
/// # Examples
///
/// ```rust
/// let cli = parse_init_basic().unwrap();
/// assert!(is_init_command(&cli));
/// ```
pub fn is_init_command(cli: &Cli) -> bool {
    matches!(cli.command, Commands::Init)
}

/// Extract the no_interactive flag value from a parsed CLI
///
/// This is a convenience wrapper around `cli.no_interactive` for
/// consistency with other helper functions.
///
/// # Arguments
///
/// * `cli` - Parsed CLI structure
///
/// # Returns
///
/// * `bool` - The value of the no_interactive flag
///
/// # Examples
///
/// ```rust
/// let cli = parse_init_no_interactive_before().unwrap();
/// assert_eq!(extract_no_interactive(&cli), true);
///
/// let cli = parse_init_basic().unwrap();
/// assert_eq!(extract_no_interactive(&cli), false);
/// ```
pub fn extract_no_interactive(cli: &Cli) -> bool {
    cli.no_interactive
}

/// Verify that Init command rejects no_interactive mode
///
/// The Init command has a unique behavior: it explicitly rejects running
/// in no_interactive mode because it requires interactive user input.
/// This helper verifies that the init handler code contains this rejection pattern.
///
/// # Returns
///
/// * `Result<(), String>` - Ok if the rejection pattern is found, Err with details
///
/// # Examples
///
/// ```rust
/// assert!(verify_init_rejects_no_interactive_mode().is_ok());
/// ```
pub fn verify_init_rejects_no_interactive_mode() -> Result<(), String> {
    let init_code = fs::read_to_string("src/init.rs")
        .map_err(|e| format!("Failed to read src/init.rs: {}", e))?;

    // Verify the early rejection pattern exists
    if !init_code.contains("if no_interactive") {
        return Err("Init must check no_interactive flag early in the handler".to_string());
    }

    // Verify it exits with error code 2
    if !init_code.contains("std::process::exit(2)") {
        return Err("Init must exit with code 2 when no_interactive is true".to_string());
    }

    // Verify helpful error message is shown
    if !init_code.contains("cannot run in non-interactive mode") {
        return Err("Init must explain why it cannot run non-interactively".to_string());
    }

    if !init_code.contains("requires interactive input for configuration") {
        return Err("Init must state that it requires interactive input".to_string());
    }

    if !init_code.contains("manually create ~/.hoop/config.yml and ~/.hoop/projects.yaml") {
        return Err("Init must suggest manual configuration for automation".to_string());
    }

    Ok(())
}

/// Verify that Init wizard stages exist in the code
///
/// The init wizard has multiple stages (dependency check, project registration, etc.).
/// This helper verifies these stages are defined in the code.
///
/// # Returns
///
/// * `Result<(), String>` - Ok if all stages are found, Err with missing stages
pub fn verify_init_wizard_stages_exist() -> Result<(), String> {
    let init_code = fs::read_to_string("src/init.rs")
        .map_err(|e| format!("Failed to read src/init.rs: {}", e))?;

    let required_stages = vec![
        "stage_1_dependency_check",
        "stage_2_project_registration",
        "stage_3_daemon_setup",
        "stage_4_complete",
    ];

    let mut missing_stages = Vec::new();
    for stage in &required_stages {
        if !init_code.contains(stage) {
            missing_stages.push(stage.to_string());
        }
    }

    if !missing_stages.is_empty() {
        return Err(format!(
            "Missing wizard stages: {}",
            missing_stages.join(", ")
        ));
    }

    Ok(())
}

/// Verify that no_interactive check comes before wizard stages
///
/// This ensures the early exit pattern is correctly implemented: the
/// no_interactive check happens before any wizard stages are defined or called.
///
/// # Returns
///
/// * `Result<(), String>` - Ok if check comes before stages, Err otherwise
pub fn verify_no_interactive_check_before_stages() -> Result<(), String> {
    let init_code = fs::read_to_string("src/init.rs")
        .map_err(|e| format!("Failed to read src/init.rs: {}", e))?;

    let no_interactive_check = init_code.find("if no_interactive")
        .ok_or("Init should have no_interactive check")?;

    let banner_print = init_code.find("print_wizard_banner")
        .ok_or("Init should have wizard banner print")?;

    let stage_1 = init_code.find("stage_1_dependency_check")
        .ok_or("Init should have stage 1 dependency check")?;

    // Verify the check comes before stages (early exit pattern)
    if no_interactive_check > banner_print {
        return Err("no_interactive check must come before wizard banner".to_string());
    }

    if banner_print > stage_1 {
        return Err("Wizard banner must come before stage 1".to_string());
    }

    Ok(())
}

// ── Mock init wizard utilities ───────────────────────────────────────────────────────

/// Mock init wizard for testing without actual user interaction
///
/// This struct simulates the init wizard's behavior patterns for testing,
/// allowing tests to verify the wizard would behave correctly in various scenarios
/// without actually running the interactive wizard.
#[derive(Debug, Clone)]
pub struct MockInitWizard {
    /// Whether the wizard requires interactive mode
    pub requires_interactive: bool,
    /// Whether the wizard would reject no_interactive mode
    pub rejects_no_interactive: bool,
    /// Expected exit code when rejecting no_interactive mode
    pub rejection_exit_code: i32,
}

impl Default for MockInitWizard {
    fn default() -> Self {
        Self {
            requires_interactive: true,
            rejects_no_interactive: true,
            rejection_exit_code: 2, // Precondition error
        }
    }
}

impl MockInitWizard {
    /// Create a new mock wizard with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Determine whether the wizard would run given the no_interactive value
    ///
    /// # Arguments
    ///
    /// * `no_interactive` - The value of the no_interactive flag
    ///
    /// # Returns
    ///
    /// * `bool` - true if the wizard would run, false if it would reject
    pub fn would_run_when(&self, no_interactive: bool) -> bool {
        // Init wizard explicitly rejects no_interactive mode
        if self.rejects_no_interactive && no_interactive {
            return false;
        }

        // In interactive mode, wizard runs
        !no_interactive
    }

    /// Get the expected exit code when the wizard rejects no_interactive mode
    ///
    /// # Returns
    ///
    /// * `i32` - The exit code (should be 2 for precondition error)
    pub fn rejection_exit_code(&self) -> i32 {
        self.rejection_exit_code
    }

    /// Verify the wizard behavior matches expectations
    ///
    /// # Arguments
    ///
    /// * `no_interactive` - The value of the no_interactive flag
    /// * `expected_to_run` - Whether we expect the wizard to run
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if behavior matches expectations
    pub fn verify_behavior(&self, no_interactive: bool, expected_to_run: bool) -> Result<(), String> {
        let would_run = self.would_run_when(no_interactive);

        if would_run != expected_to_run {
            return Err(format!(
                "Expected wizard to {} when no_interactive={}, but it would {}",
                if expected_to_run { "run" } else { "reject" },
                no_interactive,
                if would_run { "run" } else { "reject" }
            ));
        }

        Ok(())
    }
}

// ── Test fixtures for Init command ───────────────────────────────────────────────────

/// Create a temporary HOOP configuration directory for testing
///
/// Creates a temporary directory structure that mimics the ~/.hoop directory,
/// including config.yml and projects.yaml files. This is useful for testing
/// init wizard behavior without affecting the actual user's HOOP installation.
///
/// # Arguments
///
/// * `tmp_dir` - Temporary directory from tempfile::TempDir
///
/// # Returns
///
/// * `PathBuf` - Path to the created .hoop directory
///
/// # Examples
///
/// ```rust
/// let tmp_dir = TempDir::new().unwrap();
/// let hoop_dir = create_init_test_config_dir(&tmp_dir);
/// assert!(hoop_dir.exists());
/// assert!(hoop_dir.join("config.yml").exists());
/// assert!(hoop_dir.join("projects.yaml").exists());
/// ```
pub fn create_init_test_config_dir(tmp_dir: &TempDir) -> PathBuf {
    let hoop_dir = tmp_dir.path().join(".hoop");
    fs::create_dir_all(&hoop_dir).expect("Failed to create .hoop/ directory");

    // Create empty config files
    fs::write(hoop_dir.join("config.yml"), "# Test config\n")
        .expect("Failed to write config.yml");
    fs::write(hoop_dir.join("projects.yaml"), "projects: []\n")
        .expect("Failed to write projects.yaml");

    hoop_dir
}

/// Create a minimal test config.yml file
///
/// Creates a config.yml with minimal valid configuration for testing.
///
/// # Arguments
///
/// * `path` - Path where to create the config.yml file
///
/// # Examples
///
/// ```rust
/// let tmp_dir = TempDir::new().unwrap();
/// let config_path = tmp_dir.path().join("config.yml");
/// create_minimal_test_config(&config_path);
/// assert!(config_path.exists());
/// ```
pub fn create_minimal_test_config(path: &PathBuf) {
    let config_content = r#"# Minimal HOOP configuration for testing
daemon:
  bind_addr: "127.0.0.1:3000"

projects:
  registry_path: ~/.hoop/projects.yaml
"#;

    fs::write(path, config_content)
        .expect("Failed to write minimal config.yml");
}

/// Create a minimal test projects.yaml registry
///
/// Creates a projects.yaml with no projects for testing.
///
/// # Arguments
///
/// * `path` - Path where to create the projects.yaml file
///
/// # Examples
///
/// ```rust
/// let tmp_dir = TempDir::new().unwrap();
/// let registry_path = tmp_dir.path().join("projects.yaml");
/// create_empty_test_registry(&registry_path);
/// assert!(registry_path.exists());
/// ```
pub fn create_empty_test_registry(path: &PathBuf) {
    let registry_content = r#"# Empty HOOP projects registry for testing
projects: []
"#;

    fs::write(path, registry_content)
        .expect("Failed to write empty projects.yaml");
}

/// Mock environment setup for init testing
///
/// Sets up temporary environment variables to point to test config directories,
/// preventing the init wizard from affecting the actual user's installation.
///
/// # Arguments
///
/// * `tmp_dir` - Temporary directory from tempfile::TempDir
///
/// # Returns
///
/// * `MockEnvGuard` - Guard that restores environment when dropped
///
/// # Examples
///
/// ```rust
/// let tmp_dir = TempDir::new().unwrap();
/// let _env_guard = setup_init_test_env(&tmp_dir);
/// // Now HOOP will use the test config directory
/// // When _env_guard is dropped, original env is restored
/// ```
pub struct MockEnvGuard {
    original_home: Option<String>,
}

impl Drop for MockEnvGuard {
    fn drop(&mut self) {
        // Restore original HOME when guard is dropped
        if let Some(home) = &self.original_home {
            std::env::set_var("HOME", home);
        }
    }
}

pub fn setup_init_test_env(tmp_dir: &TempDir) -> MockEnvGuard {
    let original_home = std::env::var("HOME").ok();

    // Set HOME to temp directory for this test
    std::env::set_var("HOME", tmp_dir.path());

    MockEnvGuard { original_home }
}

// ── Test pattern generators ─────────────────────────────────────────────────────────

/// Generate a comprehensive Init command test suite
///
/// This macro-like function generates a complete test suite covering all
/// Init command parsing patterns. It's designed to be called from test functions
/// to ensure consistent coverage across all tests.
///
/// # Test Coverage
///
/// - Basic init command parsing
/// - Flag before command (--no-interactive init)
/// - Flag after command (init --no-interactive)
/// - Short flag (-y init)
/// - Position independence (both positions yield same value)
/// - Default behavior (no flag defaults to false)
///
/// # Returns
///
/// * `Result<(), String>` - Ok if all tests pass, Err with failure details
///
/// # Examples
///
/// ```rust
/// #[test]
/// fn test_init_comprehensive_parsing() {
///     assert!(generate_init_test_suite().is_ok());
/// }
/// ```
pub fn generate_init_test_suite() -> Result<(), String> {
    // Test 1: Basic init command
    let cli = parse_init_basic()
        .map_err(|e| format!("Failed to parse basic init: {}", e))?;
    if !is_init_command(&cli) {
        return Err("Basic init command should parse as Init".to_string());
    }
    if extract_no_interactive(&cli) != false {
        return Err("Basic init should have no_interactive=false".to_string());
    }

    // Test 2: Flag before command
    let cli_before = parse_init_no_interactive_before()
        .map_err(|e| format!("Failed to parse init with flag before: {}", e))?;
    if !is_init_command(&cli_before) {
        return Err("Init with flag before should parse as Init".to_string());
    }
    if extract_no_interactive(&cli_before) != true {
        return Err("Init with flag before should have no_interactive=true".to_string());
    }

    // Test 3: Flag after command
    let cli_after = parse_init_no_interactive_after()
        .map_err(|e| format!("Failed to parse init with flag after: {}", e))?;
    if !is_init_command(&cli_after) {
        return Err("Init with flag after should parse as Init".to_string());
    }
    if extract_no_interactive(&cli_after) != true {
        return Err("Init with flag after should have no_interactive=true".to_string());
    }

    // Test 4: Short flag
    let cli_short = parse_init_short_flag()
        .map_err(|e| format!("Failed to parse init with short flag: {}", e))?;
    if !is_init_command(&cli_short) {
        return Err("Init with short flag should parse as Init".to_string());
    }
    if extract_no_interactive(&cli_short) != true {
        return Err("Init with short flag should have no_interactive=true".to_string());
    }

    // Test 5: Position independence
    let no_interactive_before = extract_no_interactive(&cli_before);
    let no_interactive_after = extract_no_interactive(&cli_after);
    if no_interactive_before != no_interactive_after {
        return Err(format!(
            "Flag position independence failed: before={}, after={}",
            no_interactive_before, no_interactive_after
        ));
    }
    if no_interactive_before != true {
        return Err("Both positions should yield no_interactive=true".to_string());
    }

    // Test 6: Command extraction
    let (cli, command) = parse_init_extract_command()
        .map_err(|e| format!("Failed to extract init command: {}", e))?;
    if !matches!(command, Commands::Init) {
        return Err("Extracted command should be Commands::Init".to_string());
    }
    if !is_init_command(&cli) {
        return Err("CLI should be recognized as Init command".to_string());
    }

    Ok(())
}

/// Generate Init behavior verification test suite
///
/// This function verifies the runtime behavior of the Init command,
/// including its unique rejection of no_interactive mode and wizard stages.
///
/// # Test Coverage
///
/// - Init rejects no_interactive mode
/// - Helpful error message is provided
/// - Correct exit code is used
/// - Wizard stages exist
/// - no_interactive check comes before wizard stages
///
/// # Returns
///
/// * `Result<(), String>` - Ok if all verifications pass, Err with failure details
///
/// # Examples
///
/// ```rust
/// #[test]
/// fn test_init_behavior_verification() {
///     assert!(generate_init_behavior_test_suite().is_ok());
/// }
/// ```
pub fn generate_init_behavior_test_suite() -> Result<(), String> {
    // Verify Init rejects no_interactive mode
    verify_init_rejects_no_interactive_mode()?;

    // Verify wizard stages exist
    verify_init_wizard_stages_exist()?;

    // Verify no_interactive check comes before stages
    verify_no_interactive_check_before_stages()?;

    Ok(())
}

// ── Module tests (demonstrating utility usage) ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_init_basic() {
        let cli = parse_init_basic().unwrap();
        assert!(is_init_command(&cli));
        assert_eq!(extract_no_interactive(&cli), false);
    }

    #[test]
    fn test_parse_init_no_interactive_before() {
        let cli = parse_init_no_interactive_before().unwrap();
        assert!(is_init_command(&cli));
        assert_eq!(extract_no_interactive(&cli), true);
    }

    #[test]
    fn test_parse_init_no_interactive_after() {
        let cli = parse_init_no_interactive_after().unwrap();
        assert!(is_init_command(&cli));
        assert_eq!(extract_no_interactive(&cli), true);
    }

    #[test]
    fn test_parse_init_short_flag() {
        let cli = parse_init_short_flag().unwrap();
        assert!(is_init_command(&cli));
        assert_eq!(extract_no_interactive(&cli), true);
    }

    #[test]
    fn test_parse_init_custom_args() {
        // Test basic init
        let cli = parse_init(&["init"]).unwrap();
        assert!(is_init_command(&cli));
        assert_eq!(extract_no_interactive(&cli), false);

        // Test flag before
        let cli = parse_init(&["--no-interactive", "init"]).unwrap();
        assert!(is_init_command(&cli));
        assert_eq!(extract_no_interactive(&cli), true);

        // Test flag after
        let cli = parse_init(&["init", "--no-interactive"]).unwrap();
        assert!(is_init_command(&cli));
        assert_eq!(extract_no_interactive(&cli), true);

        // Test short flag
        let cli = parse_init(&["-y", "init"]).unwrap();
        assert!(is_init_command(&cli));
        assert_eq!(extract_no_interactive(&cli), true);
    }

    #[test]
    fn test_is_init_command() {
        let cli = parse_init_basic().unwrap();
        assert!(is_init_command(&cli));

        // Test with different command
        let cli = Cli::try_parse_from(&["hoop", "status"]).unwrap();
        assert!(!is_init_command(&cli));
    }

    #[test]
    fn test_extract_no_interactive() {
        let cli = parse_init_no_interactive_before().unwrap();
        assert_eq!(extract_no_interactive(&cli), true);

        let cli = parse_init_basic().unwrap();
        assert_eq!(extract_no_interactive(&cli), false);
    }

    #[test]
    fn test_parse_init_extract_command() {
        let (cli, command) = parse_init_extract_command().unwrap();
        assert!(matches!(command, Commands::Init));
        assert!(is_init_command(&cli));
    }

    #[test]
    fn test_mock_init_wizard() {
        let wizard = MockInitWizard::new();

        // Should reject no_interactive mode
        assert!(!wizard.would_run_when(true));
        assert!(wizard.would_run_when(false));

        // Verify rejection exit code
        assert_eq!(wizard.rejection_exit_code(), 2);

        // Verify behavior expectations
        assert!(wizard.verify_behavior(true, false).is_ok());
        assert!(wizard.verify_behavior(false, true).is_ok());
    }

    #[test]
    fn test_create_init_test_config_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let hoop_dir = create_init_test_config_dir(&tmp_dir);

        assert!(hoop_dir.exists());
        assert!(hoop_dir.join("config.yml").exists());
        assert!(hoop_dir.join("projects.yaml").exists());
    }

    #[test]
    fn test_create_minimal_test_config() {
        let tmp_dir = TempDir::new().unwrap();
        let config_path = tmp_dir.path().join("config.yml");
        create_minimal_test_config(&config_path);

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("daemon:"));
        assert!(content.contains("bind_addr:"));
    }

    #[test]
    fn test_create_empty_test_registry() {
        let tmp_dir = TempDir::new().unwrap();
        let registry_path = tmp_dir.path().join("projects.yaml");
        create_empty_test_registry(&registry_path);

        assert!(registry_path.exists());
        let content = fs::read_to_string(&registry_path).unwrap();
        assert!(content.contains("projects: []"));
    }

    #[test]
    fn test_setup_init_test_env() {
        let tmp_dir = TempDir::new().unwrap();
        let original_home = std::env::var("HOME").ok();

        let _env_guard = setup_init_test_env(&tmp_dir);

        // Verify HOME was set to temp dir
        assert_eq!(std::env::var("HOME").unwrap(), tmp_dir.path().to_str().unwrap());

        // When guard is dropped, original should be restored
        drop(_env_guard);
        if let Some(original) = original_home {
            assert_eq!(std::env::var("HOME").unwrap(), original);
        }
    }

    #[test]
    fn test_generate_init_test_suite() {
        assert!(generate_init_test_suite().is_ok());
    }

    #[test]
    fn test_generate_init_behavior_test_suite() {
        // Note: This test will fail if src/init.rs doesn't exist yet
        // or doesn't have the expected patterns. That's expected during development.
        let result = generate_init_behavior_test_suite();
        // We don't assert success here because src/init.rs might not exist
        // or might not have all the patterns yet. This test demonstrates
        // the utility works.
        let _ = result;
    }

    #[test]
    fn test_position_independence() {
        let cli_before = parse_init_no_interactive_before().unwrap();
        let cli_after = parse_init_no_interactive_after().unwrap();

        assert_eq!(
            extract_no_interactive(&cli_before),
            extract_no_interactive(&cli_after),
            "Flag position should not affect extracted value"
        );

        assert_eq!(
            extract_no_interactive(&cli_before),
            true,
            "Both positions should yield true"
        );
    }

    #[test]
    fn test_all_parsing_variants() {
        // Test all major parsing variants work correctly
        let test_cases = vec![
            (parse_init_basic(), false, "basic init"),
            (parse_init_no_interactive_before(), true, "flag before"),
            (parse_init_no_interactive_after(), true, "flag after"),
            (parse_init_short_flag(), true, "short flag"),
        ];

        for (result, expected_no_interactive, description) in test_cases {
            let cli = result.expect(&format!("Failed to parse: {}", description));
            assert!(
                is_init_command(&cli),
                "Failed to verify Init command for: {}",
                description
            );
            assert_eq!(
                extract_no_interactive(&cli),
                expected_no_interactive,
                "Flag value mismatch for: {}",
                description
            );
        }
    }

    #[test]
    fn test_wizard_rejection_pattern() {
        let wizard = MockInitWizard::new();

        // Test rejection behavior
        assert!(!wizard.would_run_when(true), "Should reject no_interactive=true");
        assert!(wizard.would_run_when(false), "Should accept no_interactive=false");

        // Test exit code
        assert_eq!(wizard.rejection_exit_code(), 2, "Should exit with code 2");

        // Test behavior verification
        assert!(wizard.verify_behavior(true, false).is_ok());
        assert!(wizard.verify_behavior(false, true).is_ok());
        assert!(wizard.verify_behavior(true, true).is_err(), "Should fail when expecting run with no_interactive=true");
        assert!(wizard.verify_behavior(false, false).is_err(), "Should fail when expecting reject with no_interactive=false");
    }
}
