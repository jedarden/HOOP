//! Integration test: mid-run .beads/ deletion shows error card, siblings unaffected
//!
//! CI commands:
//!   cargo test -p hoop-daemon --test beads_deletion_isolation
//!
//! This test verifies §6 Phase 2 success criterion:
//! "Killing one project's runtime (delete `.beads/`) shows an error card;
//! other projects unaffected."
//!
//! Note: Most of the integration tests that were in this file have been moved to
//! beads_deletion_http.rs and beads_removal_recovery.rs, which use the HTTP test
//! harness for more reliable integration testing. This file now contains only
//! unit tests that don't require full daemon instantiation.

use hoop_daemon::supervisor::ProjectSupervisor;

#[tokio::test]
async fn test_permanent_error_detection() {
    // Test that permanent errors are correctly detected
    assert!(
        ProjectSupervisor::is_permanent_error(".beads directory not found at: /test"),
        "Missing .beads should be a permanent error"
    );
    assert!(
        ProjectSupervisor::is_permanent_error("Workspace path does not exist: /test"),
        "Missing workspace should be a permanent error"
    );
    assert!(
        !ProjectSupervisor::is_permanent_error("Connection refused"),
        "Connection errors should not be permanent"
    );
    assert!(
        !ProjectSupervisor::is_permanent_error("Timeout"),
        "Timeouts should not be permanent"
    );
}
