//! HOOP schema definitions
//!
//! This crate provides the shared data types and schemas used across HOOP.
//! All types are generated from JSON Schema files in the `schemas/` directory
//! using typify. Every record carries `schema_version: "1.0.0"` for compatibility tracking.
//!
//! ## Schema files
//!
//! The source of truth is the JSON Schema files in `schemas/`. To add a new type:
//!
//! 1. Create a JSON Schema file following draft-07
//! 2. Include a `schema_version` property with pattern `^\d+\.\d+\.\d+$`
//! 3. Run `cargo build` to regenerate types
//!
//! ## Code generation
//!
//! - **Rust**: Generated via typify in build.rs → `OUT_DIR/types.rs`
//! - **TypeScript**: Generated via json-schema-to-typescript → `hoop-ui/web/src/types.gen.ts`

pub mod version {
    /// Current schema version following SemVer (X.Y.Z)
    pub const SCHEMA_VERSION: &str = "1.0.0";
}

// Include generated types
include!(concat!(env!("OUT_DIR"), "/types.rs"));

// Re-export commonly used types at the crate root
pub use types::*;

/// Base trait for all schema records
pub trait SchemaRecord {
    /// Returns the schema version for this record
    fn schema_version(&self) -> &'static str {
        version::SCHEMA_VERSION
    }
}

/// Health check response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: &'static str,
}

impl HealthResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            version: version::SCHEMA_VERSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    /// Round-trip test: serialize → deserialize → equal
    macro_rules! round_trip_test {
        ($name:ident, $type:ty, $value:expr) => {
            #[test]
            fn $name() {
                let original = $value;
                let serialized =
                    serde_json::to_string(&original).expect("Failed to serialize");
                let deserialized: $type =
                    serde_json::from_str(&serialized).expect("Failed to deserialize");
                assert_eq!(
                    original, deserialized,
                    "Round-trip failed for {}",
                    stringify!($type)
                );
            }
        };
    }

    // Test round-trip for WorkerData
    round_trip_test!(
        worker_data_round_trip,
        WorkerData,
        WorkerData {
            worker: "alpha".to_string(),
            state: WorkerDisplayState::Executing {
                bead: "bd-abc123".to_string(),
                adapter: "claude".to_string(),
                model: Some("opus".to_string()),
            },
            liveness: WorkerLiveness::Alive,
            last_heartbeat: "2024-01-01T00:00:00Z".to_string(),
            heartbeat_age_secs: 5,
        }
    );

    // Test round-trip for BeadData
    round_trip_test!(
        bead_data_round_trip,
        BeadData,
        BeadData {
            id: "bd-abc123".to_string(),
            title: "Test bead".to_string(),
            status: "open".to_string(),
            priority: 0,
            issue_type: "task".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            created_by: "user".to_string(),
            dependencies: vec![],
        }
    );

    // Test round-trip for ConversationData
    round_trip_test!(
        conversation_data_round_trip,
        ConversationData,
        ConversationData {
            id: "uuid-123".to_string(),
            session_id: "session-456".to_string(),
            provider: "claude".to_string(),
            kind: "operator".to_string(),
            worker_metadata: None,
            cwd: "/home/coding/project".to_string(),
            title: "Test conversation".to_string(),
            messages: vec![],
            total_tokens: 0,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            complete: false,
            file_path: "/path/to/session.jsonl".to_string(),
        }
    );

    // Test round-trip for WebSocketEvent
    round_trip_test!(
        ws_event_round_trip,
        WebSocketEvent,
        WebSocketEvent {
            r#type: "worker_update".to_string(),
            worker: None,
            workers: None,
            beads: None,
            conversations: None,
            conversation: None,
            streaming: None,
        }
    );

    // Test round-trip for ProjectsRegistry
    round_trip_test!(
        projects_registry_round_trip,
        ProjectsRegistry,
        ProjectsRegistry {
            projects: vec![],
        }
    );

    // Test round-trip for AuditRow
    round_trip_test!(
        audit_row_round_trip,
        AuditRow,
        AuditRow {
            id: "uuid-audit".to_string(),
            ts: "2024-01-01T00:00:00Z".to_string(),
            actor: "user:test".to_string(),
            kind: "bead_created".to_string(),
            target: "bd-123".to_string(),
            args: None,
            result: "success".to_string(),
            error: None,
            schema_version: "1.0.0".to_string(),
        }
    );

    // Test round-trip for Stitch
    round_trip_test!(
        stitch_round_trip,
        Stitch,
        Stitch {
            id: "uuid-stitch".to_string(),
            project: "test-project".to_string(),
            kind: "operator".to_string(),
            title: "Test stitch".to_string(),
            created_by: "user".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: None,
            closed_at: None,
            participants: vec![],
            attachments_path: None,
            archived: false,
            archived_at: None,
            worker_metadata: None,
            parent_stitch_id: None,
            pattern_id: None,
            schema_version: "1.0.0".to_string(),
        }
    );

    // Test round-trip for Pattern
    round_trip_test!(
        pattern_round_trip,
        Pattern,
        Pattern {
            id: "uuid-pattern".to_string(),
            title: "Test pattern".to_string(),
            description: Some("Test description".to_string()),
            status: "active".to_string(),
            owner: Some("user".to_string()),
            deadline: None,
            parent_pattern: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            closed_at: None,
            progress_percent: 50,
            total_cost_usd: None,
            duration_seconds: None,
            schema_version: "1.0.0".to_string(),
        }
    );

    // Test round-trip for ReflectionLedger
    round_trip_test!(
        reflection_ledger_round_trip,
        ReflectionLedger,
        ReflectionLedger {
            id: "uuid-reflection".to_string(),
            scope: "global".to_string(),
            rule: "Always use snake_case".to_string(),
            reason: "User repeatedly corrected camelCase".to_string(),
            source_stitches: vec![],
            status: "proposed".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_applied: None,
            applied_count: 0,
            approved_by: None,
            approved_at: None,
            archived_at: None,
            schema_version: "1.0.0".to_string(),
        }
    );

    // Test round-trip for CapacityAccount
    round_trip_test!(
        capacity_account_round_trip,
        CapacityAccount,
        CapacityAccount {
            id: "account-1".to_string(),
            adapter: "claude".to_string(),
            account_id: "acc-123".to_string(),
            limits: CapacityLimits {
                max_requests_per_minute: None,
                max_tokens_per_minute: None,
                max_tokens_per_day: Some(1000000),
                max_cost_usd_per_day: None,
            },
            usage: CapacityUsage {
                requests_this_minute: 100,
                tokens_this_minute: 50000,
                tokens_today: 500000,
                cost_usd_today: None,
                window_start: None,
                window_end: None,
            },
            window_start: None,
            window_end: None,
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            schema_version: "1.0.0".to_string(),
        }
    );

    // Test schema version format
    #[test]
    fn test_schema_version_format() {
        assert!(regex::Regex::new(r"^\d+\.\d+\.\d+$")
            .unwrap()
            .is_match(version::SCHEMA_VERSION));
    }
}
