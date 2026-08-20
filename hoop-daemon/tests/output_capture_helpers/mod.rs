//! Test helper functions for output capture verification
//!
//! This module provides reusable infrastructure for testing stdout/stderr capture
//! in log files when tests are run via wrapper scripts like run-with-log.sh.
//!
//! # Usage
//!
//! ```rust
//! use hoop_daemon::tests::output_capture_helpers::*;
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

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Read log file contents into memory
///
/// This is a basic infrastructure function for reading log file contents.
/// It handles file not found errors gracefully by returning an `io::Error`,
/// which allows callers to handle errors appropriately (e.g., distinguishing
/// between "file not found" vs "permission denied" vs other I/O errors).
///
/// # Arguments
///
/// * `path` - Path to the log file to read
///
/// # Returns
///
/// * `Ok(String)` - The file contents as a string
/// * `Err(io::Error)` - The I/O error if reading fails
///
/// # Example
///
/// ```rust
/// use std::path::Path;
///
/// match read_log_file(Path::new("/tmp/test.log")) {
///     Ok(contents) => println!("Read {} bytes", contents.len()),
///     Err(e) if e.kind() == io::ErrorKind::NotFound => {
///         println!("Log file not found - may not have been created yet");
///     }
///     Err(e) => eprintln!("Failed to read log file: {}", e),
/// }
/// ```
pub fn read_log_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    fs::read_to_string(path.as_ref())
}

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
    generate_output(
        OutputStream::Stdout,
        "STDOUT_MARKER: This should be in stdout",
    );
    generate_output(
        OutputStream::Stderr,
        "STDERR_MARKER: This should be in stderr",
    );
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
                generate_output(
                    OutputStream::Stdout,
                    &format!("{}_COUNT_{:03}", config.prefix, i),
                );
            }
            if i < config.stderr_count {
                generate_output(
                    OutputStream::Stderr,
                    &format!("{}_COUNT_{:03}", config.prefix, i),
                );
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
        self.content
            .get(&stream)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get the total number of lines from a specific stream
    pub fn line_count(&self, stream: OutputStream) -> usize {
        self.get_lines(stream).len()
    }

    /// Check if a specific pattern exists in the given stream
    pub fn contains_pattern(&self, stream: OutputStream, pattern: &str) -> bool {
        self.get_lines(stream)
            .iter()
            .any(|line| line.contains(pattern))
    }

    /// Count occurrences of a pattern in the given stream
    pub fn count_pattern(&self, stream: OutputStream, pattern: &str) -> usize {
        self.get_lines(stream)
            .iter()
            .filter(|line| line.contains(pattern))
            .count()
    }

    /// Verify that expected patterns are present in the stream
    pub fn verify_patterns(&self, stream: OutputStream, patterns: &[&str]) -> bool {
        patterns
            .iter()
            .all(|pattern| self.contains_pattern(stream, pattern))
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
            parts.push(format!(
                "Missing stdout: {}",
                self.missing_stdout.join(", ")
            ));
        }
        if !self.missing_stderr.is_empty() {
            parts.push(format!(
                "Missing stderr: {}",
                self.missing_stderr.join(", ")
            ));
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
///
/// A tuple of (total_bytes, line_count) for verification
pub fn generate_and_print_large_stdout(config: &LargeOutputConfig) -> (usize, usize) {
    let (total_bytes, line_count, content) = generate_large_stdout(config);

    // Output to stdout
    print!("{}", content);
    io::stdout().flush().unwrap();

    // Log verification metadata to stderr for wrapper scripts
    eprintln!(
        "VERIFICATION_METADATA: Generated {} lines (~{} bytes)",
        line_count, total_bytes
    );

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
    fs::read_dir(base_dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
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

/// Character-by-character verification result
#[derive(Debug)]
pub struct CharVerificationResult {
    /// Whether the verification passed (exact match)
    pub passed: bool,
    /// Total characters in expected content
    pub expected_chars: usize,
    /// Total characters in actual content
    pub actual_chars: usize,
    /// Position of first mismatch (0-based index)
    pub first_mismatch_pos: Option<usize>,
    /// Line number of first mismatch (1-based)
    pub first_mismatch_line: Option<usize>,
    /// Column number of first mismatch (1-based)
    pub first_mismatch_column: Option<usize>,
    /// Expected character at mismatch position
    pub expected_char: Option<char>,
    /// Actual character at mismatch position
    pub actual_char: Option<char>,
    /// Context around the first mismatch (20 characters before and after)
    pub mismatch_context: Option<String>,
}

impl CharVerificationResult {
    /// Create a successful verification result
    pub fn success(expected_chars: usize, actual_chars: usize) -> Self {
        Self {
            passed: true,
            expected_chars,
            actual_chars,
            first_mismatch_pos: None,
            first_mismatch_line: None,
            first_mismatch_column: None,
            expected_char: None,
            actual_char: None,
            mismatch_context: None,
        }
    }

    /// Create a failed verification result
    pub fn failure(
        expected_chars: usize,
        actual_chars: usize,
        pos: usize,
        line: usize,
        column: usize,
        expected_char: char,
        actual_char: char,
        context: String,
    ) -> Self {
        Self {
            passed: false,
            expected_chars,
            actual_chars,
            first_mismatch_pos: Some(pos),
            first_mismatch_line: Some(line),
            first_mismatch_column: Some(column),
            expected_char: Some(expected_char),
            actual_char: Some(actual_char),
            mismatch_context: Some(context),
        }
    }

    /// Get a detailed failure message
    pub fn failure_message(&self) -> String {
        if self.passed {
            return "✅ Character-by-character verification passed".to_string();
        }

        let mut msg = format!("❌ Character-by-character verification failed\n");
        msg.push_str(&format!(
            "Expected length: {} characters\n",
            self.expected_chars
        ));
        msg.push_str(&format!(
            "Actual length: {} characters\n",
            self.actual_chars
        ));

        if let (Some(pos), Some(line), Some(col), Some(exp), Some(act)) = (
            self.first_mismatch_pos,
            self.first_mismatch_line,
            self.first_mismatch_column,
            self.expected_char,
            self.actual_char,
        ) {
            msg.push_str(&format!(
                "First mismatch at position {} (line {}, column {})\n",
                pos, line, col
            ));
            msg.push_str(&format!("Expected character: {:?}\n", exp));
            msg.push_str(&format!("Actual character: {:?}\n", act));

            if let Some(ref context) = self.mismatch_context {
                msg.push_str(&format!("Context: \"{}\"\n", context));
                msg.push_str(&format!(
                    "              {}^\n",
                    " ".repeat(context.len().min(20))
                ));
            }
        }

        msg
    }
}

/// Extract raw stdout content from a log file (without [STDOUT] prefix)
pub fn extract_raw_stdout_from_log(log_path: &Path) -> Result<String, String> {
    let content =
        fs::read_to_string(log_path).map_err(|e| format!("Failed to read log file: {}", e))?;

    let mut stdout_lines = Vec::new();

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("[STDOUT]") {
            stdout_lines.push(rest.trim().to_string());
        }
    }

    // Reconstruct the original stdout by joining lines with newlines
    // This matches what was originally printed (each println adds a newline)
    let reconstructed = stdout_lines.join("\n") + if !stdout_lines.is_empty() { "\n" } else { "" };
    Ok(reconstructed)
}

/// Verify character-by-character that expected stdout matches logged stdout
///
/// This function performs exact character-by-character comparison between:
/// 1. The expected stdout content (what should have been generated)
/// 2. The actual stdout content found in the log file
///
/// # Arguments
///
/// * `expected_content` - The exact stdout content that should have been generated
/// * `log_path` - Path to the log file containing the captured output
///
/// # Returns
///
/// A `CharVerificationResult` containing detailed match/mismatch information
///
/// # Example
///
/// ```rust
/// let expected = "Line 1\nLine 2\n";
/// let result = verify_stdout_char_by_char(expected, Path::new("/tmp/test.log"));
/// assert!(result.passed);
/// ```
pub fn verify_stdout_char_by_char(
    expected_content: &str,
    log_path: &Path,
) -> Result<CharVerificationResult, String> {
    let actual_content = extract_raw_stdout_from_log(log_path)?;

    let expected_chars: Vec<char> = expected_content.chars().collect();
    let actual_chars: Vec<char> = actual_content.chars().collect();

    let expected_len = expected_chars.len();
    let actual_len = actual_chars.len();

    // Find the first mismatch
    let max_len = expected_len.max(actual_len);
    for (pos, (&exp_char, &act_char)) in expected_chars.iter().zip(actual_chars.iter()).enumerate()
    {
        if exp_char != act_char {
            // Calculate line and column (1-based for user readability)
            let (line, column) = calculate_line_column(&expected_chars[..pos]);

            // Get context around the mismatch
            let context_start = pos.saturating_sub(20);
            let context_end = (pos + 20).min(expected_chars.len());
            let context: String = expected_chars[context_start..context_end].iter().collect();

            return Ok(CharVerificationResult::failure(
                expected_len,
                actual_len,
                pos,
                line,
                column,
                exp_char,
                act_char,
                context,
            ));
        }
    }

    // If we haven't found a mismatch yet, check if lengths differ
    if expected_len != actual_len {
        let pos = max_len.min(expected_len);
        let (line, column) = if pos > 0 {
            calculate_line_column(&expected_chars[..pos])
        } else {
            (1, 1)
        };

        let exp_char = expected_chars.get(pos).copied().unwrap_or('\0');
        let act_char = actual_chars.get(pos).copied().unwrap_or('\0');

        let context_start = pos.saturating_sub(20);
        let context_end = (pos + 20).min(expected_chars.len());
        let context: String = expected_chars[context_start..context_end].iter().collect();

        return Ok(CharVerificationResult::failure(
            expected_len,
            actual_len,
            pos,
            line,
            column,
            exp_char,
            act_char,
            context,
        ));
    }

    Ok(CharVerificationResult::success(expected_len, actual_len))
}

/// Calculate line and column (1-based) from a character position
fn calculate_line_column(chars: &[char]) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for &ch in chars {
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

/// Verify stdout content from a LargeOutputConfig generation
///
/// Convenience function that generates expected content from a config
/// and verifies it against the log file.
///
/// # Arguments
///
/// * `config` - The LargeOutputConfig used to generate the stdout
/// * `log_path` - Path to the log file containing captured output
///
/// # Returns
///
/// A `CharVerificationResult` with detailed match/mismatch information
pub fn verify_large_stdout_output(
    config: &LargeOutputConfig,
    log_path: &Path,
) -> Result<CharVerificationResult, String> {
    let (_bytes, _lines, expected_content) = generate_large_stdout(config);
    verify_stdout_char_by_char(&expected_content, log_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

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

    #[test]
    fn test_char_verification_success() {
        // Create a temporary log file with matching content
        let log_content = "[STDOUT] Line 1\n[STDOUT] Line 2\n";
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_char_verify_success.log");
        fs::write(&log_path, log_content).unwrap();

        let expected = "Line 1\nLine 2\n";
        let result = verify_stdout_char_by_char(expected, &log_path).unwrap();

        assert!(
            result.passed,
            "Verification should pass when content matches"
        );
        assert_eq!(result.expected_chars, expected.chars().count());
        assert!(result.first_mismatch_pos.is_none());

        // Cleanup
        fs::remove_file(&log_path).ok();
    }

    #[test]
    fn test_char_verification_failure() {
        // Create a log file with different content
        let log_content = "[STDOUT] Line 1\n[STDOUT] Different content\n";
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_char_verify_failure.log");
        fs::write(&log_path, log_content).unwrap();

        let expected = "Line 1\nLine 2\n";
        let result = verify_stdout_char_by_char(expected, &log_path).unwrap();

        assert!(
            !result.passed,
            "Verification should fail when content differs"
        );
        assert!(result.first_mismatch_pos.is_some());
        assert!(result.first_mismatch_line.is_some());
        assert!(result.expected_char.is_some());
        assert!(result.actual_char.is_some());

        // Cleanup
        fs::remove_file(&log_path).ok();
    }

    #[test]
    fn test_char_verification_length_mismatch() {
        // Create a log file with shorter content
        let log_content = "[STDOUT] Short\n";
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_char_verify_length.log");
        fs::write(&log_path, log_content).unwrap();

        let expected = "Short\nExtra\n";
        let result = verify_stdout_char_by_char(expected, &log_path).unwrap();

        assert!(
            !result.passed,
            "Verification should fail when lengths differ"
        );
        assert_eq!(result.expected_chars, expected.chars().count());
        assert_ne!(result.expected_chars, result.actual_chars);

        // Cleanup
        fs::remove_file(&log_path).ok();
    }

    #[test]
    fn test_char_verification_exact_match() {
        // Test exact character-for-character match with special characters
        let log_content = "[STDOUT] Hello, 世界!\n[STDOUT] Tab\there\n";
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_char_verify_exact.log");
        fs::write(&log_path, log_content).unwrap();

        let expected = "Hello, 世界!\nTab\there\n";
        let result = verify_stdout_char_by_char(expected, &log_path).unwrap();

        assert!(
            result.passed,
            "Should handle unicode and special characters"
        );
        assert_eq!(result.actual_chars, expected.chars().count());

        // Cleanup
        fs::remove_file(&log_path).ok();
    }

    #[test]
    fn test_extract_raw_stdout() {
        // Test extraction of stdout content without prefix
        let log_content = "[STDOUT] Line 1\n[STDOUT] Line 2\n[STDERR] Error 1\n[STDOUT] Line 3\n";
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_extract_stdout.log");
        fs::write(&log_path, log_content).unwrap();

        let extracted = extract_raw_stdout_from_log(&log_path).unwrap();
        assert_eq!(extracted, "Line 1\nLine 2\nLine 3\n");

        // Cleanup
        fs::remove_file(&log_path).ok();
    }

    #[test]
    fn test_large_output_verification() {
        // Test verification with large generated content
        let config = LargeOutputConfig {
            target_size_bytes: 1024, // 1KB for quick test
            ..Default::default()
        };

        let (_bytes, _lines, expected_content) = generate_large_stdout(&config);

        // Create a log file with the generated content
        let log_lines: String = expected_content
            .lines()
            .map(|line| format!("[STDOUT] {}", line))
            .collect::<Vec<_>>()
            .join("\n");
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_large_verify.log");
        fs::write(&log_path, log_lines).unwrap();

        let result = verify_large_stdout_output(&config, &log_path).unwrap();
        assert!(result.passed, "Large output verification should pass");

        // Cleanup
        fs::remove_file(&log_path).ok();
    }

    #[test]
    fn test_calculate_line_column() {
        // Test line/column calculation
        let text: Vec<char> = "Line 1\nLine 2\nLine 3".chars().collect();

        // Position 0 should be line 1, column 1
        let (line, col) = calculate_line_column(&text[..0]);
        assert_eq!(line, 1);
        assert_eq!(col, 1);

        // Position 6 (after "Line 1\n") should be line 2, column 1
        let (line, col) = calculate_line_column(&text[..6]);
        assert_eq!(line, 2);
        assert_eq!(col, 1);

        // Position 7 should be line 2, column 2
        let (line, col) = calculate_line_column(&text[..7]);
        assert_eq!(line, 2);
        assert_eq!(col, 2);
    }

    #[test]
    fn test_verification_result_message() {
        let result = CharVerificationResult::success(100, 100);
        assert!(result.failure_message().contains("✅"));

        let result =
            CharVerificationResult::failure(100, 95, 50, 3, 10, 'x', 'y', "context...".to_string());
        let msg = result.failure_message();
        assert!(msg.contains("❌"));
        assert!(msg.contains("position 50"));
        assert!(msg.contains("line 3"));
        assert!(msg.contains("column 10"));
    }
}
