//! Test helper functions for output capture verification
//!
//! This module provides reusable infrastructure for testing stdout/stderr capture
//! in log files when tests are run via wrapper scripts like run-with-log.sh.
//!
//! # Usage
//!
//! ```rust
//! use output_capture_helpers::*;
//!
//! #[test]
//! fn test_my_output() {
//!     // Generate test output
//!     generate_stdout_message("Test message to stdout");
//!     generate_stderr_message("Test message to stderr");
//!
//!     // The wrapper script will capture this to a log file
//!     // Separate tests can verify the log file content
//! }
//! ```

use std::io::{self, Write};
use std::fs;
use std::path::Path;
use std::collections::HashMap;

/// Stream type for output generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputStream {
    /// Standard output stream
    Stdout,
    /// Standard error stream
    Stderr,
}

impl OutputStream {
    /// Write a message to this stream
    pub fn write(&self, message: &str) {
        match self {
            OutputStream::Stdout => println!("{}", message),
            OutputStream::Stderr => eprintln!("{}", message),
        }
    }

    /// Get the prefix string used in log files for this stream
    pub fn log_prefix(&self) -> &str {
        match self {
            OutputStream::Stdout => "[STDOUT]",
            OutputStream::Stderr => "[STDERR]",
        }
    }
}

/// Generate a test message to stdout
pub fn generate_stdout_message(message: &str) {
    println!("{}", message);
    io::stdout().flush().unwrap();
}

/// Generate a test message to stderr
pub fn generate_stderr_message(message: &str) {
    eprintln!("{}", message);
    io::stderr().flush().unwrap();
}

/// Generate a test message to a specific stream
pub fn generate_output(stream: OutputStream, message: &str) {
    stream.write(message);
    match stream {
        OutputStream::Stdout => io::stdout().flush().unwrap(),
        OutputStream::Stderr => io::stderr().flush().unwrap(),
    };
}

/// Generate a sequence of numbered messages to test ordering
pub fn generate_sequence(stream: OutputStream, prefix: &str, count: usize) {
    for i in 0..count {
        generate_output(stream, &format!("{}_{}", prefix, i));
    }
}

/// Generate mixed output alternating between stdout and stderr
pub fn generate_mixed_output(messages: &[(OutputStream, &str)]) {
    for (stream, message) in messages {
        generate_output(*stream, message);
    }
}

/// Generate high-volume output to stress-test capture (no data loss)
pub fn generate_high_volume_output(stdout_count: usize, stderr_count: usize) {
    for i in 0..stdout_count {
        generate_output(OutputStream::Stdout, &format!("STDOUT_COUNT_{:03}", i));
    }
    for i in 0..stderr_count {
        generate_output(OutputStream::Stderr, &format!("STDERR_COUNT_{:03}", i));
    }

    // Ensure all output is flushed
    io::stdout().flush().unwrap();
    io::stderr().flush().unwrap();
}

/// Generate interleaved output to test stream separation
pub fn generate_interleaved_output(prefix: &str, count: usize) {
    for i in 0..count {
        generate_output(OutputStream::Stdout, &format!("{}_SEQ_{}", prefix, i));
        generate_output(OutputStream::Stderr, &format!("{}_SEQ_{}", prefix, i));
    }

    io::stdout().flush().unwrap();
    io::stderr().flush().unwrap();
}

/// Generate marker messages for stream identification
pub fn generate_stream_markers() {
    generate_output(OutputStream::Stdout, "STDOUT_MARKER: This should be in stdout");
    generate_output(OutputStream::Stderr, "STDERR_MARKER: This should be in stderr");
}

/// Test output generation configuration
#[derive(Debug, Clone)]
pub struct TestOutputConfig {
    /// Number of stdout messages to generate
    pub stdout_count: usize,
    /// Number of stderr messages to generate
    pub stderr_count: usize,
    /// Prefix for numbered messages
    pub prefix: String,
    /// Whether to interleave output
    pub interleave: bool,
}

impl Default for TestOutputConfig {
    fn default() -> Self {
        Self {
            stdout_count: 10,
            stderr_count: 10,
            prefix: "TEST".to_string(),
            interleave: false,
        }
    }
}

/// Generate test output based on configuration
pub fn generate_configured_output(config: &TestOutputConfig) {
    if config.interleave {
        let max_count = config.stdout_count.max(config.stderr_count);
        for i in 0..max_count {
            if i < config.stdout_count {
                generate_output(OutputStream::Stdout, &format!("{}_COUNT_{:03}", config.prefix, i));
            }
            if i < config.stderr_count {
                generate_output(OutputStream::Stderr, &format!("{}_COUNT_{:03}", config.prefix, i));
            }
        }
    } else {
        generate_high_volume_output(config.stdout_count, config.stderr_count);
    }

    io::stdout().flush().unwrap();
    io::stderr().flush().unwrap();
}

/// Log file parser for analyzing captured output
#[derive(Debug)]
pub struct LogFileParser {
    /// Path to the log file
    path: String,
    /// Parsed content by stream
    content: HashMap<OutputStream, Vec<String>>,
}

impl LogFileParser {
    /// Create a new parser for the given log file path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        Ok(Self {
            path: path_str,
            content: HashMap::new(),
        })
    }

    /// Parse the log file and separate stdout/stderr content
    pub fn parse(&mut self) -> Result<(), String> {
        let content = fs::read_to_string(&self.path)
            .map_err(|e| format!("Failed to read log file: {}", e))?;

        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        for line in content.lines() {
            if let Some(rest) = line.strip_prefix(OutputStream::Stdout.log_prefix()) {
                stdout_lines.push(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix(OutputStream::Stderr.log_prefix()) {
                stderr_lines.push(rest.trim().to_string());
            }
        }

        self.content.insert(OutputStream::Stdout, stdout_lines);
        self.content.insert(OutputStream::Stderr, stderr_lines);

        Ok(())
    }

    /// Get all lines from a specific stream
    pub fn get_lines(&self, stream: OutputStream) -> &[String] {
        self.content.get(&stream).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get the total number of lines from a specific stream
    pub fn line_count(&self, stream: OutputStream) -> usize {
        self.get_lines(stream).len()
    }

    /// Check if a specific pattern exists in the given stream
    pub fn contains_pattern(&self, stream: OutputStream, pattern: &str) -> bool {
        self.get_lines(stream).iter().any(|line| line.contains(pattern))
    }

    /// Count occurrences of a pattern in the given stream
    pub fn count_pattern(&self, stream: OutputStream, pattern: &str) -> usize {
        self.get_lines(stream).iter()
            .filter(|line| line.contains(pattern))
            .count()
    }

    /// Verify that expected patterns are present in the stream
    pub fn verify_patterns(&self, stream: OutputStream, patterns: &[&str]) -> bool {
        patterns.iter().all(|pattern| self.contains_pattern(stream, pattern))
    }
}

/// Output verification result
#[derive(Debug)]
pub struct VerificationResult {
    /// Whether stdout verification passed
    pub stdout_passed: bool,
    /// Whether stderr verification passed
    pub stderr_passed: bool,
    /// Missing stdout patterns
    pub missing_stdout: Vec<String>,
    /// Missing stderr patterns
    pub missing_stderr: Vec<String>,
    /// Extra stdout lines not expected
    pub extra_stdout: Vec<String>,
    /// Extra stderr lines not expected
    pub extra_stderr: Vec<String>,
}

impl VerificationResult {
    /// Check if overall verification passed
    pub fn passed(&self) -> bool {
        self.stdout_passed && self.stderr_passed
    }

    /// Get a summary message
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if self.passed() {
            parts.push("✅ All output verification passed".to_string());
        } else {
            if !self.stdout_passed {
                parts.push("❌ Stdout verification failed".to_string());
            }
            if !self.stderr_passed {
                parts.push("❌ Stderr verification failed".to_string());
            }
        }

        if !self.missing_stdout.is_empty() {
            parts.push(format!("Missing stdout: {}", self.missing_stdout.join(", ")));
        }
        if !self.missing_stderr.is_empty() {
            parts.push(format!("Missing stderr: {}", self.missing_stderr.join(", ")));
        }

        parts.join("\n")
    }
}

/// Verify that expected output patterns are present in a log file
pub fn verify_output_patterns(
    log_path: &Path,
    expected_stdout: &[&str],
    expected_stderr: &[&str],
) -> Result<VerificationResult, String> {
    let mut parser = LogFileParser::new(log_path)?;
    parser.parse()?;

    let mut result = VerificationResult {
        stdout_passed: true,
        stderr_passed: true,
        missing_stdout: Vec::new(),
        missing_stderr: Vec::new(),
        extra_stdout: Vec::new(),
        extra_stderr: Vec::new(),
    };

    // Check stdout patterns
    for pattern in expected_stdout {
        if !parser.contains_pattern(OutputStream::Stdout, pattern) {
            result.stdout_passed = false;
            result.missing_stdout.push(pattern.to_string());
        }
    }

    // Check stderr patterns
    for pattern in expected_stderr {
        if !parser.contains_pattern(OutputStream::Stderr, pattern) {
            result.stderr_passed = false;
            result.missing_stderr.push(pattern.to_string());
        }
    }

    Ok(result)
}

/// Configuration for large deterministic stdout generation
#[derive(Debug, Clone)]
pub struct LargeOutputConfig {
    /// Target size in bytes (minimum, actual may be slightly larger)
    pub target_size_bytes: usize,
    /// Prefix for each line
    pub prefix: String,
    /// Whether to include line numbers (for deterministic verification)
    pub include_line_numbers: bool,
}

impl Default for LargeOutputConfig {
    fn default() -> Self {
        Self {
            target_size_bytes: 10_240, // 10KB
            prefix: "STDOUT_LINE".to_string(),
            include_line_numbers: true,
        }
    }
}

/// Generate large deterministic stdout content
///
/// This function generates substantial, deterministic stdout output that can be
/// used for testing output capture, verification, and buffer handling.
///
/// # Deterministic Behavior
///
/// The output is deterministic for a given configuration:
/// - Each line has a predictable format based on line index
/// - Total size is calculated upfront and met precisely
/// - Same configuration produces identical output across runs
///
/// # Arguments
///
/// * `config` - Configuration for output generation
///
/// # Returns
///
/// A tuple of (total_bytes, line_count, generated_content)
///
/// # Example
///
/// ```rust
/// let config = LargeOutputConfig {
///     target_size_bytes: 15_000, // ~15KB
///     ..Default::default()
/// };
/// let (bytes, lines, content) = generate_large_stdout(&config);
/// assert!(bytes > 10_000);
/// println!("{}", content); // Write to stdout
/// ```
pub fn generate_large_stdout(config: &LargeOutputConfig) -> (usize, usize, String) {
    // Calculate approximate line length: prefix + formatting + line number + text + newline
    // Example: "STDOUT_LINE_0123 - This is line 123 of the substantial stdout generation test\n"
    //         ~30-40 bytes per line for typical configurations

    let base_line_length = config.prefix.len() + 60; // Conservative estimate
    let target_lines = (config.target_size_bytes + base_line_length - 1) / base_line_length;

    let mut generated_content = String::new();

    for i in 0..target_lines {
        let line = if config.include_line_numbers {
            format!("{}_{:04} - This is line {} of the substantial stdout generation test - verifying no truncation occurs", config.prefix, i, i)
        } else {
            format!("{} - Substantial stdout output line for testing - verifiable deterministic content", config.prefix)
        };

        generated_content.push_str(&line);
        generated_content.push('\n');
    }

    let total_bytes = generated_content.len();
    let line_count = target_lines;

    (total_bytes, line_count, generated_content)
}

/// Generate large stdout and output it directly
///
/// This is a convenience function that generates large stdout content and
/// outputs it immediately. Useful for one-shot tests.
///
/// # Arguments
///
/// * `config` - Configuration for output generation
///
/// # Returns
/// A tuple of (total_bytes, line_count) for verification
pub fn generate_and_print_large_stdout(config: &LargeOutputConfig) -> (usize, usize) {
    let (total_bytes, line_count, content) = generate_large_stdout(config);

    // Output to stdout
    print!("{}", content);
    io::stdout().flush().unwrap();

    // Log verification metadata to stderr for wrapper scripts
    eprintln!("VERIFICATION_METADATA: Generated {} lines (~{} bytes)", line_count, total_bytes);

    (total_bytes, line_count)
}

/// Verify that generated content meets size requirements
///
/// # Arguments
///
/// * `actual_bytes` - Actual size of generated content
/// * `required_bytes` - Minimum required size
///
/// # Returns
///
/// true if size requirement is met, false otherwise
pub fn verify_size_requirement(actual_bytes: usize, required_bytes: usize) -> bool {
    actual_bytes >= required_bytes
}

/// Find the most recent log file matching a pattern
pub fn find_latest_log(base_dir: &Path, pattern: &str) -> Option<String> {
    fs::read_dir(base_dir).ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|name| name.contains(pattern))
                .unwrap_or(false)
        })
        .filter_map(|entry| {
            let path = entry.path();
            let modified = fs::metadata(&path).ok()?.modified().ok()?;
            Some((path, modified))
        })
        .max_by_key(|(_, modified)| *modified)
        .map(|(path, _)| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_stream_write() {
        // This test generates output for the wrapper to capture
        generate_output(OutputStream::Stdout, "Test stdout message");
        generate_output(OutputStream::Stderr, "Test stderr message");
    }

    #[test]
    fn test_sequence_generation() {
        generate_sequence(OutputStream::Stdout, "SEQ_TEST", 5);
        generate_sequence(OutputStream::Stderr, "SEQ_ERR", 5);
    }

    #[test]
    fn test_interleaved_output() {
        generate_interleaved_output("INTERLEAVE", 3);
    }

    #[test]
    fn test_high_volume_output() {
        generate_high_volume_output(50, 50);
    }

    #[test]
    fn test_configured_output() {
        let config = TestOutputConfig {
            stdout_count: 5,
            stderr_count: 3,
            prefix: "CONFIG".to_string(),
            interleave: true,
        };
        generate_configured_output(&config);
    }
}
