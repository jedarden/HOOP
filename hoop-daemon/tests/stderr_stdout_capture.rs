//! Test stdout/stderr capture in log files
//!
//! This test verifies that both stdout and stderr streams are captured
//! and distinguishable in log files when tests are run via run-with-log.sh.

use std::io::{self, Write};

#[test]
fn test_stdout_stderr_output() {
    // Clear test environment
    println!("=== Starting stdout/stderr capture test ===");

    // Write to stdout
    println!("This is a message to STDOUT from test_stdout_stderr_output");

    // Write to stderr explicitly
    eprintln!("This is a message to STDERR from test_stdout_stderr_output");

    // Write more to stdout
    println!("Another message to STDOUT");

    // Write more to stderr
    eprintln!("Another message to STDERR");

    // Mixed output
    println!("STDOUT: Mixed output message 1");
    eprintln!("STDERR: Mixed output message 1");
    println!("STDOUT: Mixed output message 2");
    eprintln!("STDERR: Mixed output message 2");

    // Ensure output is flushed
    io::stdout().flush().unwrap();
    io::stderr().flush().unwrap();

    println!("=== Test completed successfully ===");
}

#[test]
fn test_stream_distinction() {
    // Write distinct markers for stdout vs stderr
    println!("STDOUT_MARKER: This should be in stdout");
    eprintln!("STDERR_MARKER: This should be in stderr");

    // Write a sequence to verify ordering
    for i in 0..5 {
        println!("STDOUT_SEQ_{}", i);
        eprintln!("STDERR_SEQ_{}", i);
    }
}

#[test]
fn test_no_output_loss() {
    // Generate a lot of output to stress-test capture
    for i in 0..100 {
        println!("STDOUT_COUNT_{:03}", i);
        eprintln!("STDERR_COUNT_{:03}", i);
    }

    // Ensure all output is flushed
    io::stdout().flush().unwrap();
    io::stderr().flush().unwrap();

    println!("=== High-volume output test completed ===");
}
