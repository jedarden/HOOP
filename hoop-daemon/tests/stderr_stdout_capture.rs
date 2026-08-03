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

    // Use the reusable large stdout generation function
    let config = LargeOutputConfig {
        target_size_bytes: 15_000, // Target ~15KB
        prefix: "STDOUT_LINE".to_string(),
        include_line_numbers: true,
    };

    let (total_bytes, line_count) = generate_and_print_large_stdout(&config);

    // Verify the output size meets the >10KB requirement
    assert!(verify_size_requirement(total_bytes, 10_240), "Generated stdout must be at least 10KB, got {} bytes", total_bytes);

    // Log verification information for the wrapper script
    eprintln!("VERIFICATION_METADATA: Generated {} lines ({} bytes)", line_count, total_bytes);
    eprintln!("VERIFICATION_METADATA: Size requirement (>10KB): MET");

    println!("=== Substantial stdout generation test completed - Generated {} bytes in {} lines ===", total_bytes, line_count);
}

#[test]
fn test_reusable_large_stdout_generation() {
    println!("=== Testing reusable large stdout generation function ===");

    // Test 1: Generate with default configuration (>10KB)
    let default_config = LargeOutputConfig::default();
    let (bytes1, lines1) = generate_and_print_large_stdout(&default_config);
    assert!(verify_size_requirement(bytes1, 10_240), "Default config must generate >10KB");

    // Test 2: Generate with custom larger size
    let large_config = LargeOutputConfig {
        target_size_bytes: 20_000, // ~20KB
        ..Default::default()
    };
    let (bytes2, lines2) = generate_and_print_large_stdout(&large_config);
    assert!(verify_size_requirement(bytes2, 20_000), "Large config must generate >20KB");
    assert!(bytes2 > bytes1, "Large config should generate more output than default");

    // Test 3: Verify deterministic behavior - same config produces same size
    let config_clone = large_config.clone();
    let (bytes3, _lines3) = generate_and_print_large_stdout(&config_clone);
    assert_eq!(bytes2, bytes3, "Same configuration should produce identical output size");

    eprintln!("VERIFICATION_METADATA: Reusable function test completed successfully");
    eprintln!("VERIFICATION_METADATA: Test 1 (default): {} bytes in {} lines", bytes1, lines1);
    eprintln!("VERIFICATION_METADATA: Test 2 (large): {} bytes in {} lines", bytes2, lines2);
    eprintln!("VERIFICATION_METADATA: Test 3 (determinism): {} bytes == {} bytes", bytes2, bytes3);

    println!("=== Reusable large stdout generation test completed ===");
}
