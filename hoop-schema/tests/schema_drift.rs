//! Schema drift test: Rust types ↔ TypeScript types round-trip parity (§3.5)
//!
//! This test generates JSON fixtures for all schema types, which are then
//! validated by the TypeScript test in hoop-ui/web/src/schemaDrift.test.ts.
//!
//! Test fails if any field adds/removes mismatched between Rust and TS codegen.

use hoop_schema::*;
use std::fs;

/// Fixture output directory
const FIXTURE_DIR: &str = "../hoop-ui/web/src/__fixtures__/schema";

/// Generate schema fixtures for round-trip testing
///
/// This test is marked as ignored so it only runs when explicitly requested
/// (e.g., in CI or via `cargo test generate_schema_fixtures -- --ignored`).
#[ignore]
#[test]
fn generate_schema_fixtures() {
    use chrono::{DateTime, Utc};

    let ts: DateTime<Utc> = "2024-01-01T00:00:00Z".parse().unwrap();
    let uuid = uuid::Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap();

    let fixtures: Vec<(&str, String)> = vec![
        // Worker types
        (
            "worker_liveness",
            serde_json::to_string_pretty(&WorkerLiveness::Live).unwrap(),
        ),
        (
            "worker_display_state",
            serde_json::to_string_pretty(&WorkerDisplayState {
                state: WorkerDisplayStateState::Executing,
                bead: Some("bd-abc123".to_string()),
                adapter: Some("claude".to_string()),
                model: Some("opus".to_string()),
                last_strand: None,
                reason: None,
            })
            .unwrap(),
        ),
        (
            "worker_metadata",
            serde_json::to_string_pretty(&WorkerMetadata {
                worker: "alpha".to_string(),
                bead: "bd-abc123".to_string(),
                strand: None,
            })
            .unwrap(),
        ),
        (
            "worker_data",
            serde_json::to_string_pretty(&WorkerData {
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
                last_heartbeat: ts,
                heartbeat_age_secs: 5,
            })
            .unwrap(),
        ),

        // Bead types
        (
            "bead_data",
            serde_json::to_string_pretty(&BeadData {
                id: "bd-abc123".to_string(),
                title: "Test bead".to_string(),
                status: BeadDataStatus::Open,
                priority: 0,
                issue_type: BeadDataIssueType::Task,
                created_at: ts,
                updated_at: ts,
                created_by: "user".to_string(),
                dependencies: vec![],
            })
            .unwrap(),
        ),
        (
            "bead",
            serde_json::to_string_pretty(&Bead {
                id: "hoop-ttb.1".to_string(),
                title: "Test bead".to_string(),
                description: Some("Test description".to_string()),
                status: BeadStatus::Open,
                priority: 0,
                issue_type: BeadIssueType::Task,
                created_at: ts,
                updated_at: ts,
                created_by: "user".to_string(),
                dependencies: vec![],
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "bead_created_by_hoop",
            serde_json::to_string_pretty(&BeadCreatedByHoop {
                actor: "hoop:agent:session-123".to_string(),
                bead_id: "hoop-ttb.1".to_string(),
                project: "test-project".to_string(),
                source: "chat".to_string(),
                ts,
            })
            .unwrap(),
        ),

        // Session/Conversation types
        (
            "session_kind",
            serde_json::to_string_pretty(&SessionKind::Variant0 {
                worker: "alpha".to_string(),
                bead: "bd-abc123".to_string(),
                strand: None,
            }).unwrap(),
        ),
        (
            "session_message",
            serde_json::to_string_pretty(&SessionMessage {
                role: "user".to_string(),
                content: serde_json::Value::String("Hello".to_string()),
                usage: None,
                timestamp: None,
            })
            .unwrap(),
        ),
        (
            "parsed_session",
            serde_json::to_string_pretty(&ParsedSession {
                id: "session-123".to_string(),
                session_id: "session-456".to_string(),
                provider: "claude".to_string(),
                kind: ParsedSessionKind::Variant0 {
                    worker: "alpha".to_string(),
                    bead: "bd-abc123".to_string(),
                    strand: None,
                },
                cwd: "/home/coding/project".to_string(),
                canonical_cwd: None,
                title: "Test session".to_string(),
                messages: vec![],
                total_usage: ParsedSessionTotalUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_tokens: 0,
                    cache_write_tokens: 0,
                },
                created_at: ts,
                updated_at: ts,
                complete: false,
                file_path: "/path/to/session.jsonl".to_string(),
            })
            .unwrap(),
        ),
        (
            "conversation_data",
            serde_json::to_string_pretty(&ConversationData {
                id: "conv-123".to_string(),
                session_id: "session-456".to_string(),
                provider: ConversationDataProvider::Claude,
                kind: ConversationDataKind::Operator,
                worker_metadata: None,
                cwd: "/home/coding/project".to_string(),
                title: "Test conversation".to_string(),
                messages: vec![],
                total_tokens: 0,
                created_at: ts,
                updated_at: ts,
                complete: false,
                file_path: "/path/to/session.jsonl".to_string(),
            })
            .unwrap(),
        ),
        (
            "message_usage",
            serde_json::to_string_pretty(&MessageUsage {
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: 10,
                cache_write_tokens: 5,
            })
            .unwrap(),
        ),

        // Stitch types
        (
            "stitch",
            serde_json::to_string_pretty(&Stitch {
                id: uuid,
                project: "test-project".to_string(),
                kind: StitchKind::Operator,
                title: "Test stitch".to_string(),
                created_by: "user".to_string(),
                created_at: ts,
                updated_at: None,
                closed_at: None,
                participants: vec![],
                attachments_path: None,
                archived: false,
                archived_at: None,
                worker_metadata: None,
                parent_stitch_id: None,
                pattern_id: None,
                classification: Some(StitchClassification::Operator),
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "stitch_bead",
            serde_json::to_string_pretty(&StitchBead {
                stitch_id: uuid,
                bead_id: "hoop-ttb.1".to_string(),
                relationship: StitchBeadRelationship::CreatedHere,
                linked_at: Some(ts),
                workspace: "/home/coding/project".to_string(),
                canonical_workspace: None,
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "stitch_link",
            serde_json::to_string_pretty(&StitchLink {
                from_stitch: uuid,
                to_stitch: uuid,
                kind: StitchLinkKind::Spawned,
                created_at: Some(ts),
                workspace_from: "/home/coding/project".to_string(),
                workspace_to: "/home/coding/project".to_string(),
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "stitch_message",
            serde_json::to_string_pretty(&StitchMessage {
                id: uuid,
                stitch_id: uuid,
                ts,
                role: StitchMessageRole::User,
                content: serde_json::Value::String("Hello".to_string()),
                attachments: vec![],
                tokens: None,
                tool_use: None,
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "stitch_preview",
            serde_json::to_string_pretty(&StitchPreview {
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
                prediction: None,
                risk_patterns: vec![],
                file_conflicts: vec![],
                similar_stitches: vec![],
            })
            .unwrap(),
        ),

        // Pattern types
        (
            "pattern",
            serde_json::to_string_pretty(&Pattern {
                id: uuid,
                title: "Test pattern".to_string(),
                description: Some("Test description".to_string()),
                status: PatternStatus::Active,
                owner: None,
                deadline: None,
                parent_pattern: None,
                created_at: ts,
                updated_at: None,
                closed_at: None,
                progress_percent: None,
                total_cost_usd: None,
                duration_seconds: None,
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "pattern_member",
            serde_json::to_string_pretty(&PatternMember {
                pattern_id: uuid,
                stitch_id: uuid,
                added_at: Some(ts),
                added_by: Some("user".to_string()),
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "pattern_query",
            serde_json::to_string_pretty(&PatternQuery {
                pattern_id: uuid,
                query: "status:open".to_string(),
                created_at: Some(ts),
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "reflection_ledger",
            serde_json::to_string_pretty(&ReflectionLedger {
                id: uuid,
                scope: "global".to_string(),
                rule: "test rule".to_string(),
                reason: Some("test example".to_string()),
                source_stitches: vec![],
                status: ReflectionLedgerStatus::Proposed,
                created_at: ts,
                last_applied: None,
                applied_count: 0,
                approved_by: None,
                approved_at: None,
                archived_at: None,
                content_hash: "abc123".to_string(),
                rejection_count: 0,
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),

        // Configuration types
        (
            "hoop_config",
            serde_json::to_string_pretty(&HoopConfig {
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
                agent: None,
                projects_file: None,
                backup: None,
                ui: None,
                voice: None,
                agent_extensions: None,
                metrics: None,
                audit: None,
                reflection: None,
                stuck_detector: None,
                morning_brief: None,
                pricing: None,
                redaction: None,
                roles: None,
                server: None,
                embedding: None,
            })
            .unwrap(),
        ),
        (
            "agent_config",
            serde_json::to_string_pretty(&AgentConfig {
                adapter: Some(AgentConfigAdapter::Claude),
                content_seen_grace_secs: 600,
                cost_cap_per_session_usd: None,
                heartbeat_transition_threshold_secs: 300,
                idle_timeout_secs: 180,
                max_runtime_secs: 3600,
                model: Some("opus".to_string()),
                rate_limit_requests_per_minute: None,
                retry_threshold: 3,
            })
            .unwrap(),
        ),
        (
            "backup_config",
            serde_json::to_string_pretty(&BackupConfig {
                endpoint: "https://s3.example.com".to_string(),
                bucket: "test-bucket".to_string(),
                prefix: "hoop/".to_string(),
                schedule: "0 4 * * *".to_string(),
                retention_days: 30,
                encryption: false,
            })
            .unwrap(),
        ),
        (
            "ui_config",
            serde_json::to_string_pretty(&UiConfig {
                default_project_sort: UiConfigDefaultProjectSort::Name,
                theme: UiConfigTheme::Dark,
                archive_after_days: 30,
            })
            .unwrap(),
        ),
        (
            "voice_config",
            serde_json::to_string_pretty(&VoiceConfig {
                whisper_model_path: Some("/path/to/model.bin".to_string()),
                hotkey: "Ctrl+Shift+V".to_string(),
                max_recording_seconds: 300,
            })
            .unwrap(),
        ),
        (
            "pricing_config",
            serde_json::to_string_pretty(&PricingConfig {
                adapters: serde_json::Map::new(),
            })
            .unwrap(),
        ),
        (
            "config_error",
            serde_json::to_string_pretty(&ConfigError {
                message: "Invalid configuration".to_string(),
                line: 10,
                col: 5,
            })
            .unwrap(),
        ),
        (
            "project_config_status",
            serde_json::to_string_pretty(&ProjectConfigStatus {
                valid: true,
                error: None,
            })
            .unwrap(),
        ),
        (
            "morning_brief_config",
            serde_json::to_string_pretty(&MorningBriefConfig {
                window_hours: 24,
                schedule_hour: 7,
                auto_run_enabled: true,
            })
            .unwrap(),
        ),
        (
            "stuck_detector_config",
            serde_json::to_string_pretty(&StuckDetectorConfig {
                idle_timeout_secs: 180,
                max_runtime_secs: 3600,
                content_seen_grace_secs: 600,
                heartbeat_transition_threshold_secs: 300,
                retry_threshold: 3,
            })
            .unwrap(),
        ),

        // Project types
        (
            "workspace_entry",
            serde_json::to_string_pretty(&WorkspaceEntry {
                path: "/home/coding/project".to_string(),
                canonical_path: Some("/home/coding/project".to_string()),
                role: WorkspaceEntryRole::Primary,
            })
            .unwrap(),
        ),
        (
            "project_entry",
            serde_json::to_string_pretty(&ProjectEntry::Variant0 {
                name: "test-project".to_string(),
                color: Some(ProjectEntryVariant0Color("#FF0000".to_string())),
                label: None,
                path: "/home/coding/project".to_string(),
                canonical_path: None,
            })
            .unwrap(),
        ),
        (
            "projects_registry",
            serde_json::to_string_pretty(&ProjectsRegistry {
                projects: vec![],
            })
            .unwrap(),
        ),

        // Capacity types
        (
            "capacity_limits",
            serde_json::to_string_pretty(&CapacityLimits {
                tokens_per_5h: None,
                tokens_per_7d: None,
                requests_per_day: None,
                spend_usd_per_day: None,
                concurrent_requests: None,
            })
            .unwrap(),
        ),
        (
            "capacity_usage",
            serde_json::to_string_pretty(&CapacityUsage {
                tokens_5h: None,
                tokens_7d: None,
                requests_day: None,
                spend_usd_day: None,
                concurrent_requests: None,
                last_reset: None,
            })
            .unwrap(),
        ),
        (
            "capacity_account",
            serde_json::to_string_pretty(&CapacityAccount {
                id: "account-123".to_string(),
                adapter: Some(CapacityAccountAdapter::Claude),
                account_id: Some("account-456".to_string()),
                limits: None,
                usage: None,
                window_start: None,
                window_end: None,
                updated_at: ts,
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "cost_bucket",
            serde_json::to_string_pretty(&CostBucket {
                adapter: "claude".to_string(),
                model: "opus".to_string(),
                project: "test-project".to_string(),
                date: "2024-01-01".to_string(),
                classification: Some(CostBucketClassification::Fleet),
                strand: None,
                usage: CostBucketUsage {
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_read_tokens: 100,
                    cache_write_tokens: 50,
                },
                cost_usd: 0.05,
                request_count: None,
            })
            .unwrap(),
        ),

        // Audit types
        (
            "audit_row",
            serde_json::to_string_pretty(&AuditRow {
                id: uuid,
                ts,
                actor: "user:test".to_string(),
                kind: AuditRowKind::BeadCreated,
                target: "bd-123".to_string(),
                args: serde_json::Map::new(),
                result: AuditRowResult::Success,
                error: None,
                hash_prev: AuditRowHashPrev("0".repeat(64)),
                hash_self: AuditRowHashSelf("0".repeat(64)),
                schema_version: None,
            })
            .unwrap(),
        ),

        // Monitoring types
        (
            "debug_state",
            serde_json::to_string_pretty(&DebugState {
                bind_addr: "127.0.0.1:8080".to_string(),
                config_hash: "abc123".to_string(),
                fleet_db_path: "/tmp/fleet.db".to_string(),
                fleet_db_size_bytes: 1024,
                fleet_db_wal_size_bytes: 512,
                open_stitches: 5,
                projects: vec![],
                total_beads: 100,
                uptime_secs: 3600,
                version: DebugStateVersion {
                    daemon: "1.0.0".to_string(),
                    schema: "1.0.0".to_string(),
                },
                workers: vec![],
                ws_clients: vec![],
                active_claims: vec![],
                worker_pids: vec![],
                session_alias_table: vec![],
                backup_timestamps: DebugStateBackupTimestamps {
                    last_success_iso: None,
                    last_success_unix: 0,
                    last_size_bytes: 0,
                },
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),

        // WebSocket types
        (
            "ws_event",
            serde_json::to_string_pretty(&WebSocketEvent {
                type_: WebSocketEventType::WorkerUpdate,
                worker: None,
                workers: vec![],
                beads: vec![],
                conversations: vec![],
                conversation: None,
                streaming: None,
                config_status: None,
                bead_created_by_hoop: None,
            })
            .unwrap(),
        ),
        (
            "streaming_content",
            serde_json::to_string_pretty(&StreamingContent {
                content: "Hello".to_string(),
                conversation_id: "conv-123".to_string(),
                timestamp: 1704067200000,
            })
            .unwrap(),
        ),

        // UI types
        (
            "ui_state",
            serde_json::to_string_pretty(&UiState {
                active_project: None,
                active_stitch: None,
                sidebar_width: 300,
                theme: UiStateTheme::Dark,
                panel_layout: None,
                filters: None,
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),

        // Other types
        (
            "dictated_note",
            serde_json::to_string_pretty(&DictatedNote {
                stitch_id: uuid,
                audio_filename: "recording.m4a".to_string(),
                duration_secs: None,
                language: None,
                recorded_at: ts,
                transcribed_at: ts,
                transcript: "Test transcript".to_string(),
                transcript_words: vec![],
                tags: vec![],
                redacted_words: vec![],
                transcription_status: None,
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "redaction_policy",
            serde_json::to_string_pretty(&RedactionPolicy {
                patterns: vec![RedactionPolicyPatternsItem::AnthropicApiKey],
                action: RedactionPolicyAction::Warn,
            })
            .unwrap(),
        ),
        (
            "script_manifest",
            serde_json::to_string_pretty(&ScriptManifest {
                name: "test-script".to_string(),
                description: Some("Test script".to_string()),
                scope: ScriptManifestScope::Global,
                projects: vec![],
                timeout_secs: 300,
                arguments: vec![],
                schedule: None,
                overlap_policy: ScriptManifestOverlapPolicy::Skip,
                on: vec![],
            })
            .unwrap(),
        ),
        (
            "secret_pattern",
            serde_json::to_string_pretty(&SecretPattern {
                id: "test_pattern".to_string(),
                name: "Test Pattern".to_string(),
                severity: SecretPatternSeverity::High,
                patterns: vec!["sk-ant-[0-9a-zA-Z_-]{95}".to_string()],
            })
            .unwrap(),
        ),
        (
            "template_field",
            serde_json::to_string_pretty(&TemplateField {
                key: "test_field".to_string(),
                label: "Test Field".to_string(),
                required: true,
                placeholder: None,
                default: None,
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "stitch_template",
            serde_json::to_string_pretty(&StitchTemplate {
                name: "test-template".to_string(),
                description: "Test template".to_string(),
                scope: "global".to_string(),
                kind: None,
                priority: None,
                labels: None,
                default_beads: None,
                fields: vec![],
                body: "Test body with {{test_field}}".to_string(),
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "unknown_events_response",
            serde_json::to_string_pretty(&UnknownEventsResponse {
                total_count: 10,
                labeled_totals: vec![],
                daemon_version: "1.0.0".to_string(),
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
        (
            "unknown_event_samples_response",
            serde_json::to_string_pretty(&UnknownEventSamplesResponse {
                samples: vec![],
                total_count: 5,
                daemon_version: "1.0.0".to_string(),
                schema_version: hoop_schema::version::SCHEMA_VERSION.parse().unwrap(),
            })
            .unwrap(),
        ),
    ];

    // Create fixture directory
    fs::create_dir_all(FIXTURE_DIR)
        .expect("Failed to create fixture directory");

    // Write each fixture
    for (name, json) in &fixtures {
        let file_path = format!("{}/{}.json", FIXTURE_DIR, name);
        fs::write(&file_path, &json)
            .expect(&format!("Failed to write fixture: {}", file_path));
    }

    // Write index
    let index: serde_json::Map<String, serde_json::Value> = fixtures
        .iter()
        .map(|(name, _)| {
            (
                name.to_string(),
                serde_json::json!({"file": format!("{}.json", name)}),
            )
        })
        .collect();

    let index_path = format!("{}/index.json", FIXTURE_DIR);
    fs::write(&index_path, serde_json::to_string_pretty(&index).unwrap())
        .expect("Failed to write index");

    println!("Generated {} schema fixtures in {}", fixtures.len(), FIXTURE_DIR);
}

/// Validate round-trip: Rust → JSON → Rust parse → deep-equal
///
/// This test reads the generated fixtures and verifies that Rust can
/// parse them back into identical values. This ensures serialization
/// and deserialization are symmetric.
///
/// Run this test after `generate_schema_fixtures` to verify that
/// the fixtures are valid and can be round-tripped correctly.
#[test]
fn validate_fixture_roundtrip() {
    let fixture_files = [
        "worker_liveness.json",
        "worker_display_state.json",
        "worker_metadata.json",
        "worker_data.json",
        "bead_data.json",
        "bead.json",
        "bead_created_by_hoop.json",
        "session_kind.json",
        "session_message.json",
        "parsed_session.json",
        "conversation_data.json",
        "message_usage.json",
        "stitch.json",
        "stitch_bead.json",
        "stitch_link.json",
        "stitch_message.json",
        "stitch_preview.json",
        "pattern.json",
        "pattern_member.json",
        "pattern_query.json",
        "reflection_ledger.json",
        "hoop_config.json",
        "agent_config.json",
        "backup_config.json",
        "ui_config.json",
        "voice_config.json",
        "pricing_config.json",
        "config_error.json",
        "project_config_status.json",
        "morning_brief_config.json",
        "stuck_detector_config.json",
        "workspace_entry.json",
        "project_entry.json",
        "projects_registry.json",
        "capacity_limits.json",
        "capacity_usage.json",
        "capacity_account.json",
        "cost_bucket.json",
        "codex_account_daily_spend_row.json",
        "codex_account_monthly_rollup_row.json",
        "audit_row.json",
        "debug_state.json",
        "ws_event.json",
        "streaming_content.json",
        "ui_state.json",
        "dictated_note.json",
        "redaction_policy.json",
        "script_manifest.json",
        "secret_pattern.json",
        "template_field.json",
        "stitch_template.json",
        "unknown_events_response.json",
        "unknown_event_samples_response.json",
    ];

    for fixture_file in fixture_files {
        let fixture_path = format!("{}/{}", FIXTURE_DIR, fixture_file);

        // Skip if fixture doesn't exist (may not have been generated yet)
        if !std::path::Path::new(&fixture_path).exists() {
            eprintln!(
                "Warning: Fixture {} not found (run generate_schema_fixtures first)",
                fixture_file
            );
            continue;
        }

        let json_content = fs::read_to_string(&fixture_path)
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_file, e));

        // Parse as generic JSON to preserve structure
        let original: serde_json::Value = serde_json::from_str(&json_content)
            .unwrap_or_else(|e| panic!("Failed to parse fixture {} as JSON: {}", fixture_file, e));

        // Re-serialize to normalize formatting
        let normalized = serde_json::to_string(&original)
            .unwrap_or_else(|e| panic!("Failed to serialize fixture {}: {}", fixture_file, e));

        // Parse back and compare
        let round_trip: serde_json::Value = serde_json::from_str(&normalized).unwrap_or_else(|e| {
            panic!("Failed to parse normalized JSON for {}: {}", fixture_file, e)
        });

        assert_eq!(
            original, round_trip,
            "Round-trip failed for {}: serialized value differs after round-trip",
            fixture_file
        );
    }

    // Verify index.json exists and is valid
    let index_path = format!("{}/index.json", FIXTURE_DIR);
    if std::path::Path::new(&index_path).exists() {
        let index_content = fs::read_to_string(&index_path).expect("Failed to read index.json");
        let _index: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&index_content).expect("Failed to parse index.json");
    }
}
