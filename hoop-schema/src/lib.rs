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

pub mod id_validators;

pub mod version {
    /// Current schema version following SemVer (X.Y.Z)
    pub const SCHEMA_VERSION: &str = "1.0.0";

    // Minimum pinned br version, generated from br-compat.toml by build.rs
    include!(concat!(env!("OUT_DIR"), "/br_compat.rs"));
}

// Include generated types at crate root
include!(concat!(env!("OUT_DIR"), "/types.rs"));

/// A unified view of a workspace, abstracting over both `ProjectEntry` variants.
#[derive(Debug, Clone)]
pub struct WorkspaceView {
    /// Raw workspace path (display-only)
    pub path: std::path::PathBuf,
    /// Realpath-resolved absolute path (for joins/dedup). None if not yet resolved.
    pub canonical_path: Option<std::path::PathBuf>,
    pub role: WorkspaceViewRole,
}

/// Workspace role, mirroring the JSON schema enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceViewRole {
    Primary,
    Manifests,
    Source,
    Secrets,
    Docs,
}

impl std::fmt::Display for WorkspaceViewRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primary => write!(f, "primary"),
            Self::Manifests => write!(f, "manifests"),
            Self::Source => write!(f, "source"),
            Self::Secrets => write!(f, "secrets"),
            Self::Docs => write!(f, "docs"),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ProjectsRegistry {
    fn default() -> Self {
        Self { projects: vec![] }
    }
}

/// Helper methods shared between `ProjectEntry` and `ProjectsRegistryProjectsItem`.
/// Both enums have the same two-variant shape (shorthand path vs multi-workspace).
macro_rules! impl_project_helpers {
    ($ty:ty, $ws_item:ty, $role_ty:ty) => {
        impl $ty {
            /// Returns the project name regardless of which variant this is.
            pub fn name(&self) -> &str {
                match self {
                    Self::Variant0 { name, .. } => name,
                    Self::Variant1 { name, .. } => name,
                }
            }

            /// Returns a unified workspace view for all workspaces in this project.
            pub fn workspace_views(&self) -> Vec<WorkspaceView> {
                match self {
                    Self::Variant0 { path, .. } => vec![WorkspaceView {
                        path: std::path::PathBuf::from(path),
                        role: WorkspaceViewRole::Primary,
                    }],
                    Self::Variant1 { workspaces, .. } => workspaces
                        .iter()
                        .map(|w| WorkspaceView {
                            path: std::path::PathBuf::from(&w.path),
                            role: match w.role {
                                <$role_ty>::Primary => WorkspaceViewRole::Primary,
                                <$role_ty>::Manifests => WorkspaceViewRole::Manifests,
                                <$role_ty>::Source => WorkspaceViewRole::Source,
                                <$role_ty>::Secrets => WorkspaceViewRole::Secrets,
                                <$role_ty>::Docs => WorkspaceViewRole::Docs,
                            },
                        })
                        .collect(),
                }
            }

            /// Returns an iterator over all workspace paths in this project.
            pub fn all_paths(&self) -> impl Iterator<Item = std::path::PathBuf> + '_ {
                self.workspace_views().into_iter().map(|w| w.path)
            }

            /// Returns the optional display label.
            pub fn label(&self) -> Option<&str> {
                match self {
                    Self::Variant0 { label, .. } => label.as_deref(),
                    Self::Variant1 { label, .. } => label.as_deref(),
                }
            }

            /// Returns the optional color hex code.
            pub fn color(&self) -> Option<&str> {
                match self {
                    Self::Variant0 { color, .. } => color.as_ref().map(|c| c.as_str()),
                    Self::Variant1 { color, .. } => color.as_ref().map(|c| c.as_str()),
                }
            }
        }
    };
}

impl_project_helpers!(ProjectEntry, ProjectEntryVariant1WorkspacesItem, ProjectEntryVariant1WorkspacesItemRole);
impl_project_helpers!(ProjectsRegistryProjectsItem, ProjectsRegistryProjectsItemVariant1WorkspacesItem, ProjectsRegistryProjectsItemVariant1WorkspacesItemRole);

/// Base trait for all schema records
pub trait SchemaRecord {
    /// Returns the schema version for this record
    fn schema_version(&self) -> &'static str {
        version::SCHEMA_VERSION
    }
}

/// Health check response (liveness)
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

/// Readiness check response — 200 when all project runtimes are healthy,
/// 503 with degraded project names otherwise.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadinessResponse {
    pub status: String,
    pub version: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub degraded: Vec<DegradedProject>,
}

/// A single degraded project entry in the readiness response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DegradedProject {
    pub project: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ReadinessResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            version: version::SCHEMA_VERSION,
            degraded: Vec::new(),
        }
    }

    pub fn degraded(projects: Vec<DegradedProject>) -> Self {
        Self {
            status: "degraded".to_string(),
            version: version::SCHEMA_VERSION,
            degraded: projects,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

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

    fn parse_utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&Utc)
    }

    // Test round-trip for WorkerData
    round_trip_test!(
        worker_data_round_trip,
        WorkerData,
        WorkerData {
            worker: "alpha".to_string(),
            state: WorkerDataState {
                state: WorkerDataStateState::Executing,
                bead: Some("bd-abc123".to_string()),
                adapter: Some("claude".to_string()),
                model: Some("opus".to_string()),
                last_strand: None,
                reason: None,
            },
            liveness: WorkerDataLiveness::Live,
            last_heartbeat: parse_utc("2024-01-01T00:00:00Z"),
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
            status: BeadDataStatus::Open,
            priority: 0,
            issue_type: BeadDataIssueType::Task,
            created_at: parse_utc("2024-01-01T00:00:00Z"),
            updated_at: parse_utc("2024-01-01T00:00:00Z"),
            created_by: "user".to_string(),
            dependencies: vec![],
        }
    );

    // Test round-trip for ConversationData
    round_trip_test!(
        conversation_data_round_trip,
        ConversationData,
        ConversationData {
            id: "conv-123".to_string(),
            session_id: "session-456".to_string(),
            provider: ConversationDataProvider::Claude,
            kind: ConversationDataKind::Operator,
            worker_metadata: None,
            cwd: "/home/coding/project".to_string(),
            title: "Test conversation".to_string(),
            messages: vec![],
            total_tokens: 0,
            created_at: parse_utc("2024-01-01T00:00:00Z"),
            updated_at: parse_utc("2024-01-01T00:00:00Z"),
            complete: false,
            file_path: "/path/to/session.jsonl".to_string(),
        }
    );

    // Test round-trip for WebSocketEvent
    round_trip_test!(
        ws_event_round_trip,
        WebSocketEvent,
        WebSocketEvent {
            type_: WebSocketEventType::WorkerUpdate,
            worker: None,
            workers: vec![],
            beads: vec![],
            conversations: vec![],
            conversation: None,
            streaming: None,
            config_status: None,
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
            id: Uuid::new_v4(),
            ts: parse_utc("2024-01-01T00:00:00Z"),
            actor: "user:test".to_string(),
            kind: AuditRowKind::BeadCreated,
            target: "bd-123".to_string(),
            args: serde_json::Map::new(),
            result: AuditRowResult::Success,
            error: None,
            schema_version: AuditRowSchemaVersion("1.0.0".to_string()),
        }
    );

    // Test round-trip for Stitch
    round_trip_test!(
        stitch_round_trip,
        Stitch,
        Stitch {
            id: Uuid::new_v4(),
            project: "test-project".to_string(),
            kind: StitchKind::Operator,
            title: "Test stitch".to_string(),
            created_by: "user".to_string(),
            created_at: parse_utc("2024-01-01T00:00:00Z"),
            updated_at: None,
            closed_at: None,
            participants: vec![],
            attachments_path: None,
            archived: false,
            archived_at: None,
            worker_metadata: None,
            parent_stitch_id: None,
            pattern_id: None,
            schema_version: StitchSchemaVersion("1.0.0".to_string()),
        }
    );

    // Test round-trip for Pattern
    round_trip_test!(
        pattern_round_trip,
        Pattern,
        Pattern {
            id: Uuid::new_v4(),
            title: "Test pattern".to_string(),
            description: Some("Test description".to_string()),
            status: PatternStatus::Active,
            owner: None,
            deadline: None,
            parent_pattern: None,
            created_at: parse_utc("2024-01-01T00:00:00Z"),
            updated_at: None,
            closed_at: None,
            progress_percent: None,
            total_cost_usd: None,
            duration_seconds: None,
            schema_version: PatternSchemaVersion("1.0.0".to_string()),
        }
    );

    // Test round-trip for ReflectionLedger
    round_trip_test!(
        reflection_ledger_round_trip,
        ReflectionLedger,
        ReflectionLedger {
            id: Uuid::new_v4(),
            scope: "global".to_string(),
            rule: "Always use snake_case".to_string(),
            reason: Some("User repeatedly corrected camelCase".to_string()),
            source_stitches: vec![],
            status: ReflectionLedgerStatus::Proposed,
            created_at: parse_utc("2024-01-01T00:00:00Z"),
            last_applied: None,
            applied_count: 0,
            approved_by: None,
            approved_at: None,
            archived_at: None,
            schema_version: ReflectionLedgerSchemaVersion("1.0.0".to_string()),
        }
    );

    // Test round-trip for CapacityAccount
    round_trip_test!(
        capacity_account_round_trip,
        CapacityAccount,
        CapacityAccount {
            id: "account-1".to_string(),
            adapter: CapacityAccountAdapter::Claude,
            account_id: "acc-123".to_string(),
            limits: CapacityAccountLimits {
                concurrent_requests: None,
                requests_per_day: None,
                spend_usd_per_day: None,
                tokens_per_5h: None,
                tokens_per_7d: Some(1000000),
            },
            usage: CapacityAccountUsage {
                active_requests: None,
                requests_today: 100,
                spend_usd_today: 50.0,
                tokens_5h: 50000,
                tokens_7d: 500000,
            },
            window_start: None,
            window_end: None,
            updated_at: parse_utc("2024-01-01T00:00:00Z"),
            schema_version: CapacityAccountSchemaVersion("1.0.0".to_string()),
        }
    );

    // Test schema version format
    #[test]
    fn test_schema_version_format() {
        let re = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
        assert!(re.is_match(version::SCHEMA_VERSION));
    }

    /// Round-trip test for HoopConfig covering all §17.3 sections.
    /// Constructs a config with every section populated, serializes to JSON,
    /// deserializes back, and asserts equality — proving Rust ↔ JSON ↔ TS fidelity.
    ///
    /// Fields with JSON Schema defaults are generated as non-Option by typify.
    #[test]
    fn hoop_config_round_trip() {
        let sv = || HoopConfigSchemaVersion("1.0.0".to_string());

        let original = HoopConfig {
            // §17.3 §1: agent
            agent: Some(HoopConfigAgent {
                adapter: Some(HoopConfigAgentAdapter::Claude),
                model: Some("opus".to_string()),
                rate_limit_requests_per_minute: Some(60),
                cost_cap_per_session_usd: Some(10.0),
            }),
            // §17.3 §2: projects_file
            projects_file: Some("~/.hoop/projects.yaml".to_string()),
            // §17.3 §3: backup
            backup: Some(HoopConfigBackup {
                endpoint: "https://s3.example.com".to_string(),
                bucket: "hoop-backups".to_string(),
                prefix: "hoop/".to_string(),
                schedule: "0 4 * * *".to_string(),
                retention_days: 30,
                encryption: false,
            }),
            // §17.3 §4: ui
            ui: Some(HoopConfigUi {
                theme: HoopConfigUiTheme::Auto,
                default_project_sort: HoopConfigUiDefaultProjectSort::LastActivity,
                archive_after_days: 30,
            }),
            // §17.3 §5: voice
            voice: Some(HoopConfigVoice {
                whisper_model_path: Some("/path/to/model.bin".to_string()),
                hotkey: "Ctrl+Shift+V".to_string(),
                max_recording_seconds: 300,
            }),
            // §17.3 §6: agent_extensions
            agent_extensions: Some(HoopConfigAgentExtensions {
                skills: Some("~/.hoop/skills".to_string()),
                scripts: Some("~/.hoop/scripts".to_string()),
                notes: Some("~/.hoop/notes".to_string()),
                prompts: Some("~/.hoop/prompts".to_string()),
            }),
            // §17.3 §7: metrics
            metrics: Some(HoopConfigMetrics {
                enabled: true,
                port: 9091,
            }),
            // §17.3 §8: audit
            audit: Some(HoopConfigAudit {
                retention_days: 90,
                hash_chain: true,
            }),
            // §17.3 §9: reflection
            reflection: Some(HoopConfigReflection {
                enabled: true,
                detection_threshold: Some(0.8),
                auto_archive_after_days: 30,
            }),
            // pricing (beyond §17.3, but part of config)
            pricing: None,
            // schema_version (required)
            schema_version: sv(),
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&original).expect("serialize HoopConfig");
        // Deserialize back
        let round_tripped: HoopConfig =
            serde_json::from_str(&json).expect("deserialize HoopConfig");

        assert_eq!(original, round_tripped, "HoopConfig round-trip mismatch");

        // Verify schema_version is present in the JSON output
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed["schema_version"], "1.0.0",
            "schema_version must appear in serialized JSON"
        );
    }

    /// Minimal HoopConfig round-trip: only required field (schema_version).
    #[test]
    fn hoop_config_minimal_round_trip() {
        let original = HoopConfig {
            schema_version: HoopConfigSchemaVersion("1.0.0".to_string()),
            agent: None,
            projects_file: None,
            backup: None,
            ui: None,
            voice: None,
            agent_extensions: None,
            metrics: None,
            audit: None,
            reflection: None,
            pricing: None,
        };

        let json = serde_json::to_string(&original).expect("serialize");
        let round_tripped: HoopConfig =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, round_tripped);
    }
}
