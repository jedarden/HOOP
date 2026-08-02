//! Test stdout/stderr capture in log files
//!
//! This test verifies that both stdout and stderr streams are captured
//! and distinguishable in log files when tests are run via run-with-log.sh.
//!
//! # Usage
//!
//! Run these tests via the wrapper script to capture output:
//! ```bash
//! ./bin/test-with-log.sh stderr_stdout_capture
//! ```
//!
//! The generated log file will contain both [STDOUT] and [STDERR] prefixed lines.

mod output_capture_helpers;
use output_capture_helpers::*;
use std::io::{self, Write};

#[test]
fn test_stdout_stderr_output() {
    println!("=== Starting stdout/stderr capture test ===");

    // Write to stdout using helper
    generate_stdout_message("This is a message to STDOUT from test_stdout_stderr_output");

    // Write to stderr using helper
    generate_stderr_message("This is a message to STDERR from test_stdout_stderr_output");

    // Write more to stdout
    generate_output(OutputStream::Stdout, "Another message to STDOUT");

    // Write more to stderr
    generate_output(OutputStream::Stderr, "Another message to STDERR");

    // Mixed output
    generate_mixed_output(&[
        (OutputStream::Stdout, "STDOUT: Mixed output message 1"),
        (OutputStream::Stderr, "STDERR: Mixed output message 1"),
        (OutputStream::Stdout, "STDOUT: Mixed output message 2"),
        (OutputStream::Stderr, "STDERR: Mixed output message 2"),
    ]);

    println!("=== Test completed successfully ===");
}

#[test]
fn test_stream_distinction() {
    // Use helper to generate stream markers
    generate_stream_markers();

    // Write a sequence to verify ordering using helper
    generate_sequence(OutputStream::Stdout, "STDOUT_SEQ", 5);
    generate_sequence(OutputStream::Stderr, "STDERR_SEQ", 5);

    io::stdout().flush().unwrap();
    io::stderr().flush().unwrap();
}

#[test]
fn test_no_output_loss() {
    // Use helper to generate high-volume output
    generate_high_volume_output(100, 100);

    println!("=== High-volume output test completed ===");
}

#[test]
fn test_interleaved_output_preservation() {
    println!("=== Testing interleaved output preservation ===");

    // Generate interleaved output
    generate_interleaved_output("INTERLEAVE_TEST", 10);

    println!("=== Interleaved output test completed ===");
}

#[test]
fn test_configured_output_generation() {
    println!("=== Testing configured output generation ===");

    let config = TestOutputConfig {
        stdout_count: 25,
        stderr_count: 25,
        prefix: "CONFIG_TEST".to_string(),
        interleave: true,
    };

    generate_configured_output(&config);

    println!("=== Configured output generation test completed ===");
}

#[test]
fn test_mixed_stream_sequences() {
    println!("=== Testing mixed stream sequences ===");

    // Test different patterns of output
    let mixed_patterns = vec![
        (OutputStream::Stdout, "Pattern A - stdout"),
        (OutputStream::Stderr, "Pattern A - stderr"),
        (OutputStream::Stdout, "Pattern B - stdout"),
        (OutputStream::Stderr, "Pattern B - stderr"),
        (OutputStream::Stdout, "Pattern C - stdout"),
        (OutputStream::Stderr, "Pattern C - stderr"),
    ];

    generate_mixed_output(&mixed_patterns);

    println!("=== Mixed stream sequences test completed ===");
}

#[test]
fn test_output_flush_behavior() {
    println!("=== Testing output flush behavior ===");

    // Generate output and explicitly flush after each
    for i in 0..10 {
        generate_output(OutputStream::Stdout, &format!("FLUSH_TEST_{}", i));
        io::stdout().flush().unwrap();
        generate_output(OutputStream::Stderr, &format!("FLUSH_ERR_{}", i));
        io::stderr().flush().unwrap();
    }

    println!("=== Flush behavior test completed ===");
}

#[test]
fn test_substantial_stdout_generation() {
    println!("=== Testing substantial stdout generation (>10KB) ===");

    // Generate >10KB of stdout output with verifiable content
    let lines_to_generate = 200; // Each line ~70 bytes => ~14KB total
    let mut generated_content = String::new();

    for i in 0..lines_to_generate {
        let line = format!("STDOUT_LINE_{:04} - This is line {} of the substantial stdout generation test - verifying no truncation occurs", i, i);
        generated_content.push_str(&line);
        generated_content.push('\n');
        println!("{}", line);
    }

    // Flush to ensure all output is written
    io::stdout().flush().unwrap();

    // Log verification information for the wrapper script
    eprintln!("VERIFICATION_METADATA: Generated {} lines (~{} bytes)", lines_to_generate, generated_content.len());
    eprintln!("VERIFICATION_METADATA: First line: {}", generated_content.lines().next().unwrap_or("N/A"));
    eprintln!("VERIFICATION_METADATA: Last line: {}", generated_content.lines().last().unwrap_or("N/A"));

    println!("=== Substantial stdout generation test completed - Generated {} bytes ===", generated_content.len());
}
