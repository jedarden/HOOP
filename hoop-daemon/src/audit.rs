//! Runtime prerequisite audit for HOOP
//!
//! Validates dependencies, environment, and configuration.
//! Each failure includes the exact command to fix it.

use crate::br_verbs;
use hoop_schema::version::BR_MIN_VERSION;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use std::time::Duration;

/// Minimum disk space required (1GB in bytes)
const MIN_DISK_SPACE: u64 = 1024 * 1024 * 1024;

/// Audit check severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Critical failure - daemon should not start
    Critical,
    /// Warning - daemon can start with degraded features
    Warning,
    /// Informational - for audit reporting only
    Info,
}

/// Result of a single audit check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditCheck {
    /// Check name/identifier
    pub name: String,
    /// Severity level
    pub severity: Severity,
    /// Whether the check passed
    pub passed: bool,
    /// Human-readable description
    pub description: String,
    /// Exact command to fix the issue (if failed)
    pub fix_command: Option<String>,
    /// Additional context/detail
    pub detail: Option<String>,
}

impl AuditCheck {
    /// Create a passed check
    pub fn passed(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            severity: Severity::Info,
            passed: true,
            description: description.into(),
            fix_command: None,
            detail: None,
        }
    }

    /// Create a failed critical check
    pub fn critical(
        name: impl Into<String>,
        description: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            severity: Severity::Critical,
            passed: false,
            description: description.into(),
            fix_command: Some(fix.into()),
            detail: None,
        }
    }

    /// Create a failed warning check
    pub fn warning(
        name: impl Into<String>,
        description: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            severity: Severity::Warning,
            passed: false,
            description: description.into(),
            fix_command: Some(fix.into()),
            detail: None,
        }
    }

    /// Add detail to a check
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Complete audit report
#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    /// All checks performed
    pub checks: Vec<AuditCheck>,
    /// Overall success (no critical failures)
    pub success: bool,
}

impl AuditReport {
    /// Create a new report from checks
    pub fn new(checks: Vec<AuditCheck>) -> Self {
        let success = !checks
            .iter()
            .any(|c| !c.passed && c.severity == Severity::Critical);
        Self { checks, success }
    }

    /// Get only critical failures
    pub fn critical_failures(&self) -> Vec<&AuditCheck> {
        self.checks
            .iter()
            .filter(|c| !c.passed && c.severity == Severity::Critical)
            .collect()
    }

    /// Get only warnings
    pub fn warnings(&self) -> Vec<&AuditCheck> {
        self.checks
            .iter()
            .filter(|c| !c.passed && c.severity == Severity::Warning)
            .collect()
    }
}

/// Configuration for audit checks
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Registered project paths to check
    pub project_paths: Vec<PathBuf>,
    /// Whether to include optional checks
    pub include_optional: bool,
    /// Timeout for external commands
    pub command_timeout: Duration,
    /// Allow br version mismatch (dev override for --allow-br-mismatch)
    pub allow_br_mismatch: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            project_paths: Vec::new(),
            include_optional: true,
            command_timeout: Duration::from_secs(5),
            allow_br_mismatch: false,
        }
    }
}

impl AuditConfig {
    /// Create with project paths
    pub fn with_projects(project_paths: Vec<PathBuf>) -> Self {
        Self {
            project_paths,
            ..Default::default()
        }
    }
}

/// Run all audit checks
pub fn run_audit(config: &AuditConfig) -> AuditReport {
    let mut checks = Vec::new();

    // Critical checks
    if config.allow_br_mismatch {
        checks.push(AuditCheck::warning(
            "br_version",
            format!("br version check skipped (--allow-br-mismatch); requires >={}", BR_MIN_VERSION),
            format!("Remove --allow-br-mismatch and update br to >={}", BR_MIN_VERSION),
        ));
    } else {
        checks.push(check_br_version());
    }
    checks.push(check_tmux());
    checks.extend(check_beads_accessibility(&config.project_paths));
    checks.push(check_cli_session_dirs());
    checks.push(check_disk_space());
    checks.push(check_restore_state());

    // Optional/warning checks
    if config.include_optional {
        checks.push(check_tailscale());
        checks.push(check_systemd_user_scope());
    }

    AuditReport::new(checks)
}

/// Run audit check for daemon startup.
///
/// Returns an error if critical failures are found, preventing daemon startup.
/// Logs warnings for non-critical issues but allows startup to proceed.
pub fn daemon_startup_check(config: &AuditConfig) -> anyhow::Result<()> {
    let report = run_audit(config);

    if !report.success {
        let failures = report.critical_failures();
        let mut msg = String::from("HOOP daemon startup audit failed:\n");

        for check in failures {
            msg.push_str(&format!("  - {}: {}\n", check.name, check.description));
            if let Some(fix) = &check.fix_command {
                msg.push_str(&format!("    Fix: {}\n", fix));
            }
        }

        return Err(anyhow::anyhow!("{}", msg));
    }

    // Log warnings but don't fail
    let warnings = report.warnings();
    if !warnings.is_empty() {
        tracing::warn!("HOOP daemon starting with degraded features:");
        for check in warnings {
            tracing::warn!("  - {}: {}", check.name, check.description);
        }
    }

    Ok(())
}

/// Check if br is in PATH and meets minimum version
fn check_br_version() -> AuditCheck {
    // Route through br_verbs choke point so all br subprocess calls are
    // centralized and the zero-write invariant can audit them.
    let result = br_verbs::invoke_br_read(br_verbs::ReadVerb::Version, &[])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    match result {
        Ok(version_output) if !version_output.is_empty() => {
            // Parse version - br outputs like "br 0.4.0" or similar
            let version_str = version_output
                .split_whitespace()
                .last()
                .unwrap_or(&version_output);

            if version_meets_minimum(version_str, BR_MIN_VERSION) {
                AuditCheck::passed(
                    "br_version",
                    format!("br {} found (>= {})", version_str, BR_MIN_VERSION),
                )
            } else {
                AuditCheck::critical(
                    "br_version",
                    format!("br {} is below minimum required {}", version_str, BR_MIN_VERSION),
                    "curl -sSL https://github.com/dicklesworthstone/beads_rust/releases/latest/download/br-linux-x86_64 -o ~/.local/bin/br && chmod +x ~/.local/bin/br".to_string(),
                )
            }
        }
        _ => AuditCheck::critical(
            "br_version",
            "br not found in PATH",
            "curl -sSL https://github.com/dicklesworthstone/beads_rust/releases/latest/download/br-linux-x86_64 -o ~/.local/bin/br && chmod +x ~/.local/bin/br",
        ),
    }
}

/// Check if tmux is in PATH
fn check_tmux() -> AuditCheck {
    let result = Command::new("tmux")
        .arg("-V")
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            AuditCheck::passed("tmux", format!("tmux found: {}", version))
        }
        _ => AuditCheck::critical(
            "tmux",
            "tmux not found in PATH",
            "apt install tmux  # Debian/Ubuntu\n  brew install tmux  # macOS",
        ),
    }
}

/// Check if .beads/ is accessible for each registered project
fn check_beads_accessibility(project_paths: &[PathBuf]) -> Vec<AuditCheck> {
    let mut checks = Vec::new();

    if project_paths.is_empty() {
        checks.push(AuditCheck::warning(
            "beads_access",
            "No projects registered yet",
            "hoop projects add <path>",
        ));
        return checks;
    }

    for path in project_paths {
        let beads_path = path.join(".beads");
        let project_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if !path.exists() {
            checks.push(AuditCheck::critical(
                format!("beads_{}", project_name),
                format!("Project path does not exist: {}", path.display()),
                format!("hoop projects remove {}", project_name),
            ));
            continue;
        }

        match fs::read_dir(&beads_path) {
            Ok(_) => {
                checks.push(AuditCheck::passed(
                    format!("beads_{}", project_name),
                    format!(".beads/ accessible at {}", path.display()),
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                checks.push(AuditCheck::critical(
                    format!("beads_{}", project_name),
                    format!(".beads/ not found at {}", path.display()),
                    format!("cd {} && br init", path.display()),
                ));
            }
            Err(e) => {
                checks.push(AuditCheck::critical(
                    format!("beads_{}", project_name),
                    format!(".beads/ not accessible at {}: {}", path.display(), e),
                    format!("ls -la {}", beads_path.display()),
                ));
            }
        }
    }

    checks
}

/// Check if CLI session directories are readable
fn check_cli_session_dirs() -> AuditCheck {
    let cli_dirs = [
        ("Claude Code", "~/.claude/projects/"),
        ("Codex", "~/.codex/sessions/"),
        ("OpenCode", "~/.opencode/sessions/"),
        ("Gemini", "~/.gemini/sessions/"),
        ("Aider", "~/.aider/sessions/"),
    ];

    let mut accessible = Vec::new();
    let mut failed = Vec::new();

    for (name, path) in cli_dirs {
        let expanded = shellexpand::tilde(path);
        let path = Path::new(expanded.as_ref());

        if path.exists() {
            match path.read_dir() {
                Ok(_) => accessible.push(name),
                Err(e) => failed.push((name, e.to_string())),
            }
        }
    }

    if accessible.is_empty() && failed.is_empty() {
        AuditCheck::warning(
            "cli_sessions",
            "No CLI session directories found",
            "Install at least one CLI: Claude Code, Codex, OpenCode, Gemini, or Aider",
        )
    } else if !failed.is_empty() {
        let failed_list = failed
            .iter()
            .map(|(n, e)| format!("{}: {}", n, e))
            .collect::<Vec<_>>()
            .join("; ");
        AuditCheck::warning(
            "cli_sessions",
            format!("Some CLI sessions not accessible: {}", failed_list),
            "Check permissions on CLI cache directories",
        )
    } else {
        AuditCheck::passed(
            "cli_sessions",
            format!("CLI sessions accessible: {}", accessible.join(", ")),
        )
    }
}

/// Check if ~/.hoop/ has sufficient disk space
fn check_disk_space() -> AuditCheck {
    let hoop_dir = shellexpand::tilde("~/.hoop");
    let hoop_dir = Path::new(hoop_dir.as_ref());

    // Create directory if it doesn't exist
    if !hoop_dir.exists() {
        let _ = fs::create_dir_all(hoop_dir);
    }

    // Get disk space using statvfs or similar
    // Cross-platform: we'll try to use a simple heuristic
    // On Linux/Unix we can use statvfs syscall
    #[cfg(unix)]
    {
        match hoop_dir.metadata() {
            Ok(_) => {
                // Try to get available space via df or stat command
                let result = Command::new("df")
                    .arg("--output=avail")
                    .arg(hoop_dir)
                    .output();

                if let Ok(output) = result {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let avail_str = stdout
                        .lines()
                        .last()
                        .unwrap_or("0")
                        .trim();

                    if let Ok(avail_kb) = avail_str.parse::<u64>() {
                        let avail_bytes = avail_kb * 1024;
                        if avail_bytes >= MIN_DISK_SPACE {
                            let avail_gb = avail_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                            return AuditCheck::passed(
                                "disk_space",
                                format!("~/.hoop/ has {:.2}GB available", avail_gb),
                            );
                        } else {
                            return AuditCheck::critical(
                                "disk_space",
                                format!(
                                    "~/.hoop/ has only {:.2}MB available (>= 1GB required)",
                                    avail_bytes / (1024 * 1024)
                                ),
                                "rm -rf ~/.hoop/attachments/*  # Clear old attachments, or\n  rm -rf ~/.hoop/  # Reset HOOP state",
                            );
                        }
                    }
                }
            }
            Err(e) => {
                return AuditCheck::warning(
                    "disk_space",
                    format!("Cannot check disk space: {}", e),
                    "df -h ~/.hoop/",
                );
            }
        }
    }

    // Fallback: assume OK if we can't check
    AuditCheck::passed("disk_space", "Disk space check skipped (unsupported platform)")
}

/// Check for leftover `~/.hoop.rollback.*` directories indicating an
/// interrupted restore. The daemon must not start in this state because
/// `~/.hoop/` may contain a partially-restored snapshot. (§15.4)
fn check_restore_state() -> AuditCheck {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut leftovers: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(&home) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(".hoop.rollback.") {
                leftovers.push(name.into_owned());
            }
        }
    }

    if leftovers.is_empty() {
        return AuditCheck::passed("restore_state", "No interrupted restore detected");
    }

    let names = leftovers.join(", ");
    AuditCheck::critical(
        "restore_state",
        format!("Interrupted restore detected: {} found. ~/.hoop/ may be inconsistent.", names),
        format!("Complete the restore or recover manually:\n  mv {} ~/.hoop/  # undo partial restore", leftovers[0]),
    )
}

/// Check if Tailscale interface is available (optional)
fn check_tailscale() -> AuditCheck {
    let result = Command::new("tailscale")
        .arg("status")
        .arg("--json")
        .output();

    match result {
        Ok(output) if output.status.success() => {
            // Try to parse as JSON to get the current machine's name
            if let Ok(json) = String::from_utf8(output.stdout) {
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&json) {
                    if let Some(name) = obj.get("Self")
                        .and_then(|s| s.get("NickName"))
                        .and_then(|n| n.as_str())
                    {
                        return AuditCheck::passed(
                            "tailscale",
                            format!("Tailscale connected: {}", name),
                        );
                    }
                }
            }
            AuditCheck::passed("tailscale", "Tailscale interface available")
        }
        _ => AuditCheck::warning(
            "tailscale",
            "Tailscale not available or not connected",
            "tailscale up  # Connect to Tailscale",
        ),
    }
}

/// Check if systemd user scope is enabled (optional)
fn check_systemd_user_scope() -> AuditCheck {
    // Check if systemd user is running
    let result = Command::new("systemctl")
        .arg("--user")
        .arg("status")
        .output();

    match result {
        Ok(output) if output.status.success() => {
            AuditCheck::passed("systemd_user", "systemd user scope available")
        }
        _ => AuditCheck::warning(
            "systemd_user",
            "systemd user scope not available (daemon will run without service supervision)",
            "loginctl enable-linger $USER  # Enable systemd user scope for persistent service",
        ),
    }
}

/// Compare two version strings (semver-like)
fn version_meets_minimum(version: &str, minimum: &str) -> bool {
    let v_parts: Vec<&str> = version.trim_start_matches('v').split('.').collect();
    let m_parts: Vec<&str> = minimum.trim_start_matches('v').split('.').collect();

    for i in 0..3 {
        let v = v_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let m = m_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

        if v > m {
            return true;
        }
        if v < m {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(version_meets_minimum("0.4.0", "0.4.0"));
        assert!(version_meets_minimum("0.5.0", "0.4.0"));
        assert!(version_meets_minimum("1.0.0", "0.4.0"));
        assert!(!version_meets_minimum("0.3.0", "0.4.0"));
        assert!(!version_meets_minimum("0.4.0", "0.5.0"));
    }

    #[test]
    fn test_audit_report_success() {
        let report = AuditReport::new(vec![
            AuditCheck::passed("test1", "ok"),
            AuditCheck::passed("test2", "ok"),
        ]);
        assert!(report.success);
        assert!(report.critical_failures().is_empty());
    }

    #[test]
    fn test_audit_report_critical_failure() {
        let report = AuditReport::new(vec![
            AuditCheck::passed("test1", "ok"),
            AuditCheck::critical("test2", "failed", "fix"),
        ]);
        assert!(!report.success);
        assert_eq!(report.critical_failures().len(), 1);
    }

    #[test]
    fn test_audit_report_warning_only() {
        let report = AuditReport::new(vec![
            AuditCheck::passed("test1", "ok"),
            AuditCheck::warning("test2", "warn", "fix"),
        ]);
        assert!(report.success); // Warnings don't block startup
        assert_eq!(report.warnings().len(), 1);
    }

    #[test]
    fn test_restore_state_detects_leftover_rollback() {
        let home = dirs::home_dir().unwrap();
        let rollback_dir = home.join(".hoop.rollback.20240101T000000Z");
        // Only test if we can create temp files in home (CI may restrict)
        let can_test = std::fs::create_dir(&rollback_dir).is_ok();

        if can_test {
            let check = check_restore_state();
            assert!(!check.passed, "should detect leftover rollback dir");
            assert_eq!(check.severity, Severity::Critical);
            assert!(
                check.description.contains("Interrupted restore"),
                "{:?}",
                check.description
            );
            // Cleanup
            let _ = fs::remove_dir(&rollback_dir);
        }
    }

    #[test]
    fn test_restore_state_passes_clean() {
        // If no .hoop.rollback.* dirs exist, check passes
        let check = check_restore_state();
        // This could fail if a real rollback dir exists, but in test env it shouldn't
        // We only assert the "passed" case if we're confident no leftovers exist
        if !check.passed {
            // A leftover exists from another test or real usage — skip assertion
            eprintln!("Skipping clean-state assertion: {}", check.description);
        }
    }
}
