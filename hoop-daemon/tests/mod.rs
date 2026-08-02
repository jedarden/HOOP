//! Acceptance tests for HOOP scenarios S1-S6
//!
//! These tests verify the acceptance criteria defined in the plan (§1.8).
//! Each scenario represents a real-world usage pattern that must work
//! correctly for HOOP to be considered complete for its phase.
//!
//! Scenarios:
//! - S1: Morning review dashboard (Phase 2)
//! - S2: Transcript archaeology / visual debug panel (Phase 2)
//! - S3: Bead creation from chat (Phase 2)
//! - S4: Daemon restart with no fleet disruption (Phase 1)
//! - S5: Degraded mode - workspace deleted at runtime (Phase 2)
//! - S6: Machine mode / non-interactive (Phase 1) - covered by existing tests

mod integration_harness;

mod s1_morning_review;
mod s2_transcript_archaeology;
mod s3_bead_creation_from_chat;
mod s4_daemon_restart;
mod s5_workspace_deleted;
