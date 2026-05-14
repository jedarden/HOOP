//! OpenAPI 3.1 spec generation and serving
//!
//! This module generates and serves the OpenAPI specification for all HOOP REST API endpoints.
//! The spec is available at:
//! - `/api/openapi.json` — JSON spec
//! - `/api/openapi.yaml` — YAML spec
//! - `/api/docs` — Swagger UI
//! - `/api/docs/redoc` — ReDoc
//! - `/api/docs/rapidoc` — RapiDoc
//!
//! ## Adding new endpoints
//!
//! 1. Add `#[utoipa::path]` annotation to the handler function
//! 2. Add `#[derive(ToSchema)]` to request/response types
//! 3. Add the handler to the `paths()` list below
//! 4. Add schemas to the `components(schemas())` list below

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_rapidoc::RapiDoc;

/// OpenAPI documentation structure
///
/// NOTE: The paths() section is temporarily commented out because handlers need
/// #[utoipa::path] annotations which are not yet implemented. The schemas are still
/// available for OpenAPI documentation.
#[cfg_attr(feature = "openapi", derive(OpenApi))]
#[openapi(
    info(
        title = "HOOP REST API",
        version = "1.0.0",
        description = "
# HOOP REST API

HOOP is an opinionated project management system built on beads, stitches, and
semantic deduplication. This API provides endpoints for:

- **Agent sessions**: Control Claude/OpenAI agent lifecycle
- **Beads**: Create, list, and manage work items
- **Stitches**: Multi-bead work items with decomposition
- **Drafts**: Queue-based stitch approval workflow
- **Audit**: Query audit log with hash chain verification
- **Transcription**: Whisper audio transcription jobs
- **Uploads**: Chunked resumable file upload
- **Attachments**: Serve bead/stitch attachments
- **Dictated notes**: Voice note capture with transcription
- **Metrics**: Prometheus metrics endpoint
- **Morning brief**: Daily brief generation

## Breaking Changes

Per §20, all spec changes require a CHANGELOG entry.
        ",
        contact(
            name = "HOOP Project",
            url = "https://github.com/jedarden/hoop"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    // Paths section temporarily disabled - handlers need #[utoipa::path] annotations
    // paths(
    //     // Agent API
    //     crate::api_agent::get_status,
    //     crate::api_agent::spawn_session,
    //     crate::api_agent::disable_agent,
    //     crate::api_agent::switch_adapter,
    //     crate::api_agent::send_turn,
    //     crate::api_agent::list_sessions,
    //
    //     // Backup API
    //     crate::api_backup::trigger_backup,
    //
    //     // Beads API
    //     crate::api_beads::list_open_beads,
    //     crate::api_beads::create_bead,
    //     crate::api_beads::check_dedup,
    //     crate::api_beads::dismiss_dedup,
    //     crate::api_beads::query_vector_index,
    //     crate::api_beads::get_vector_index_stats,
    //
    //     // Bead Blockers API
    //     crate::api_bead_blockers::get_bead_blockers,
    //
    //     // Audit API
    //     crate::api_audit::list_audit_rows,
    //     crate::api_audit::verify_hash_chain,
    //     crate::api_audit::query_redaction_audit,
    //
    //     // Attachments API
    //     crate::api_attachments::serve_attachment,
    //
    //     // Config API
    //     crate::api_config::get_config,
    //     crate::api_config::get_secrets_patterns,
    //
    //     // Content Blocks API
    //     crate::api_content_blocks::list_content_blocks,
    //     crate::api_content_blocks::create_content_block,
    //     crate::api_content_blocks::update_content_block_endpoint,
    //     crate::api_content_blocks::delete_content_block_endpoint,
    //     crate::api_content_blocks::reorder_content_blocks_endpoint,
    //
    //     // Conversations API
    //     crate::api_conversations::list_conversations,
    //
    //     // Cost Per Stitch API
    //     crate::api_cost_per_stitch::get_stitch_trends,
    //     crate::api_cost_per_stitch::get_stitch_cost,
    //
    //     // Dictated Notes API
    //     crate::api_dictated_notes::create_note,
    //     crate::api_dictated_notes::list_notes,
    //     crate::api_dictated_notes::get_note,
    //     crate::api_dictated_notes::update_note,
    //     crate::api_dictated_notes::redact_words,
    //     crate::api_dictated_notes::get_audio,
    //     crate::api_dictated_notes::synthesize_draft,
    //     crate::api_dictated_notes::create_draft_from_note,
    //
    //     // Draft Queue API
    //     crate::api_draft_queue::list_all_drafts,
    //     crate::api_draft_queue::list_project_drafts,
    //     crate::api_draft_queue::get_draft,
    //     crate::api_draft_queue::create_draft,
    //     crate::api_draft_queue::approve_draft,
    //     crate::api_draft_queue::edit_draft,
    //     crate::api_draft_queue::reject_draft,
    //     crate::api_draft_queue::open_draft,
    //     crate::api_draft_queue::autosave_draft,
    //     crate::api_draft_queue::abandon_draft,
    //     crate::api_draft_queue::get_dedup_stats,
    //     crate::api_draft_queue::report_false_positive,
    //
    //     // Files API
    //     crate::api_files::list_directory,
    //     crate::api_files::get_file_content,
    //     crate::api_files::search_files,
    //
    //     // Fix Patterns API
    //     crate::api_fix_patterns::create_pattern,
    //     crate::api_fix_patterns::list_patterns,
    //     crate::api_fix_patterns::get_pattern,
    //     crate::api_fix_patterns::update_pattern,
    //     crate::api_fix_patterns::delete_pattern,
    //     crate::api_fix_patterns::match_patterns,
    //     crate::api_fix_patterns::search_patterns,
    //     crate::api_fix_patterns::export_patterns,
    //     crate::api_fix_patterns::import_patterns,
    //
    //     // Metrics API
    //     crate::api_metrics::get_metrics,
    //     crate::api_metrics::debug_state,
    //     crate::api_metrics::get_unknown_events,
    //     crate::api_metrics::get_unknown_event_samples,
    //
    //     // Morning Brief API
    //     crate::api_morning_brief::get_latest,
    //     crate::api_morning_brief::list_briefs,
    //     crate::api_morning_brief::trigger_brief,
    //     crate::api_morning_brief::get_status,
    //
    //     // Onboarding API
    //     crate::api_onboarding::list_onboarding_prompts,
    //     crate::api_onboarding::dismiss_onboarding_prompt,
    //     crate::api_onboarding::set_onboarding_enabled,
    //     crate::api_onboarding::record_feature_usage,
    //     crate::api_onboarding::acknowledge_version,
    //
    //     // Orphans API
    //     crate::api_orphans::list_orphans,
    //     crate::api_orphans::attach_orphan,
    //
    //     // Patterns API
    //     crate::api_patterns::list_patterns,
    //     crate::api_patterns::get_pattern,
    //
    //     // Presence API
    //     crate::api_presence::get_presence,
    //     crate::api_presence::update_presence,
    //     crate::api_presence::remove_presence,
    //
    //     // Preview API
    //     crate::api_preview::preview_bead,
    //
    //     // Prompts API
    //     crate::api_prompts::list_prompts,
    //     crate::api_prompts::get_prompt,
    //     crate::api_prompts::substitute_prompt,
    //
    //     // Reflection Ledger API
    //     crate::api_reflection_ledger::list_proposals,
    //     crate::api_reflection_ledger::list_reflections,
    //     crate::api_reflection_ledger::approve_proposal,
    //     crate::api_reflection_ledger::reject_proposal,
    //
    //     // Screen Capture API
    //     crate::api_screen_capture::create_screen_capture,
    //     crate::api_screen_capture::list_screen_captures,
    //     crate::api_screen_capture::get_metadata,
    //     crate::api_screen_capture::get_video,
    //
    //     // Scripts API
    //     crate::api_scripts::list_scripts,
    //     crate::api_scripts::get_script,
    //     crate::api_scripts::run_script,
    //
    //     // Stitch Decompose API
    //     crate::api_stitch_decompose::preview_decompose,
    //     crate::api_stitch_decompose::submit_stitch,
    //
    //     // Stitch Links API
    //     crate::api_stitch_links::create_link,
    //     crate::api_stitch_links::delete_link,
    //     crate::api_stitch_links::search_stitches,
    //
    //     // Stitch Read API
    //     crate::api_stitch_read::read_stitch,
    //
    //     // Stitch Replay API
    //     crate::api_stitch_replay::get_replay_options,
    //     crate::api_stitch_replay::resume_as_new_bead,
    //     crate::api_stitch_replay::restore_workspace_state,
    //
    //     // Stitch Traversal API
    //     crate::api_stitch_traversal::get_parents,
    //     crate::api_stitch_traversal::get_children,
    //     crate::api_stitch_traversal::get_referenced_by,
    //     crate::api_stitch_traversal::get_closure,
    //
    //     // Timeline API
    //     crate::api_timeline::get_worker_timeline,
    //
    //     // Tour Project API
    //     crate::api_tour_project::enable_tour_project,
    //     crate::api_tour_project::disable_tour_project,
    //     crate::api_tour_project::get_tour_status,
    //
    //     // Transcription API
    //     crate::api_transcription::get_job,
    //     crate::api_transcription::list_jobs,
    //
    //     // UI State API
    //     crate::api_ui_state::get_ui_state,
    //     crate::api_ui_state::put_ui_state,
    //     crate::api_ui_state::put_ui_state_batch,
    //     crate::api_ui_state::delete_ui_state,
    //
    //     // Unassigned API
    //     crate::api_unassigned::list_unassigned,
    //     crate::api_unassigned::assign_session,
    //     crate::api_unassigned::ignore_session,
    //
    //     // Uploads API
    //     crate::api_uploads::init_upload,
    //     crate::api_uploads::upload_chunk,
    //     crate::api_uploads::get_progress,
    //     crate::api_uploads::complete_upload,
    //     crate::api_uploads::cancel_upload,
    //
    //     // Diff API
    //     crate::api_diff::get_project_diff,
    //     crate::api_diff::get_merge_base,
    //
    //     // Blame API
    //     crate::api_blame::get_file_blame,
    // ),
    components(
        schemas(
            // Agent API types
            crate::agent_session::AgentSessionStatus,
            crate::api_agent::SwitchRequest,
            crate::api_agent::TurnRequest,
            crate::api_agent::TurnAttachment,
            crate::fleet::AgentSessionRow,

            // Backup API types
            crate::api_backup::TriggerResponse,

            // Beads API types
            crate::api_beads::BeadSummary,
            crate::api_beads::CreateBeadRequest,
            crate::api_beads::CreateBeadResponse,
            crate::api_beads::DedupCheckRequest,
            crate::api_beads::DedupCheckResponse,
            crate::api_beads::DedupMatchRef,
            crate::api_beads::VectorIndexStats,

            // Bead Blockers API types
            crate::api_bead_blockers::CrossWorkspaceBlocker,
            crate::api_bead_blockers::BeadBlockersResponse,

            // Audit API types
            crate::api_audit::AuditQuery,
            crate::api_audit::AuditResponse,
            crate::api_audit::AuditRow,
            crate::api_audit::HashChainVerifyResponse,
            crate::api_audit::RedactionAuditQuery,
            crate::api_audit::RedactionAuditResponse,
            crate::api_audit::RedactionAuditRow,
            crate::fleet::ActionKind,
            crate::fleet::ActionResult,

            // Config API types
            crate::api_config::ConfigResponse,
            crate::api_config::SecretsPatternsResponse,
            crate::api_config::RunningConfig,
            crate::config_resolver::SecretPattern,

            // Content Blocks API types
            crate::content_blocks::ContentBlock,
            crate::content_blocks::ContentBlockCreate,
            crate::content_blocks::ContentBlockUpdate,

            // Conversations API types
            crate::api_conversations::ConversationsQuery,
            crate::api_conversations::ConversationsResponse,
            crate::api_conversations::ConversationSummary,
            crate::api_conversations::WorkerMetadata,

            // Cost Per Stitch API types
            crate::api_cost_per_stitch::CostTrendsResponse,
            crate::api_cost_per_stitch::StitchCostResponse,
            crate::api_cost_per_stitch::CostTrendPoint,
            crate::api_cost_per_stitch::AdapterCostTrend,
            crate::api_cost_per_stitch::ProjectCostTrend,

            // Dictated Notes API types
            crate::dictated_notes::TranscriptionStatus,
            crate::dictated_notes::DictatedNote,
            crate::dictated_notes::TranscriptWord,
            crate::dictated_notes::RedactedWord,
            crate::dictated_notes::CreateNoteRequest,
            crate::dictated_notes::CreateNoteResponse,
            crate::api_dictated_notes::UpdateNoteRequest,
            crate::api_dictated_notes::RedactWordsRequest,
            crate::api_dictated_notes::SynthesizeRequest,
            crate::api_dictated_notes::SynthesizeResponse,

            // Draft Queue API types
            crate::api_draft_queue::DraftResponse,
            crate::api_draft_queue::DraftListResponse,
            crate::api_draft_queue::CreateDraftRequest,
            crate::api_draft_queue::CreateDraftResponse,
            crate::api_draft_queue::ApproveRequest,
            crate::api_draft_queue::ApproveResponse,
            crate::api_draft_queue::EditDraftRequest,
            crate::api_draft_queue::EditResponse,
            crate::api_draft_queue::RejectRequest,
            crate::api_draft_queue::RejectResponse,
            crate::api_draft_queue::ReportFalsePositiveRequest,
            crate::api_draft_queue::ReportFalsePositiveResponse,
            crate::api_draft_queue::DedupStatsResponse,
            crate::api_draft_queue::OpenDraftRequest,
            crate::api_draft_queue::OpenDraftResponse,
            crate::api_draft_queue::AutosaveDraftRequest,
            crate::api_draft_queue::AutosaveDraftResponse,
            crate::api_draft_queue::AbandonDraftResponse,
            crate::fleet::DraftRow,
            crate::fleet::BeadSource,

            // Files API types
            crate::files::FileEntry,
            crate::files::FileSearchResult,
            crate::files::GrepMatch,

            // Fix Patterns API types
            crate::api_fix_patterns::PatternListResponse,
            crate::api_fix_patterns::PatternDetail,
            crate::api_fix_patterns::PatternCreatedResponse,
            crate::api_fix_patterns::PatternMatchResponse,
            crate::api_fix_patterns::PatternMatchDetail,
            crate::api_fix_patterns::MatchRequest,
            crate::api_fix_patterns::PatternsExportResponse,
            crate::api_fix_patterns::PatternExport,
            crate::api_fix_patterns::PatternsImportRequest,
            crate::api_fix_patterns::PatternsImportResponse,

            // Metrics API types
            crate::api_metrics::DebugStateResponse,
            crate::api_metrics::UnknownEventsResponse,
            crate::api_metrics::UnknownEventSamplesResponse,

            // Morning Brief API types
            crate::api_morning_brief::TriggerResponse,
            crate::api_morning_brief::StatusResponse,
            crate::fleet::MorningBriefRow,

            // Onboarding API types
            crate::api_onboarding::OnboardingPrompt,
            crate::api_onboarding::OnboardingPromptsResponse,
            crate::api_onboarding::DismissPromptRequest,
            crate::api_onboarding::RecordFeatureUsageRequest,

            // Orphans API types
            crate::orphan_beads::OrphansResponse,
            crate::orphan_beads::OrphanBead,
            crate::api_orphans::AttachOrphanRequest,
            crate::api_orphans::AttachOrphanResponse,

            // Patterns API types
            crate::api_patterns::PatternListResponse,
            crate::api_patterns::PatternDetailResponse,
            crate::api_patterns::PatternListItem,

            // Presence API types
            crate::api_presence::PresenceResponse,
            crate::api_presence::PresenceListResponse,
            crate::api_presence::UpdatePresenceRequest,
            crate::api_presence::UpdatePresenceResponse,

            // Preview API types
            crate::api_preview::PreviewRequest,
            crate::api_preview::StitchPreview,
            crate::api_preview::PredictionData,
            crate::api_preview::PercentileEstimate,
            crate::api_preview::DateRange,
            crate::api_preview::RiskPatternMatch,
            crate::api_preview::RiskPatternInfo,
            crate::api_preview::FileConflict,
            crate::api_preview::SimilarStitchRef,

            // Prompts API types
            crate::api_prompts::Prompt,
            crate::api_prompts::PromptLibrary,
            crate::api_prompts::SubstitutionRequest,
            crate::api_prompts::SubstitutionResponse,

            // Propagation API types
            crate::api_propagation::DetectRequest,
            crate::api_propagation::DetectResponse,
            crate::cross_project_propagation::PropagationResult,
            crate::cross_project_propagation::SiblingProject,
            crate::cross_project_propagation::SiblingStitch,
            crate::cross_project_propagation::SiblingEvidence,
            crate::cross_project_propagation::SourceStitchInfo,

            // Reflection Ledger API types
            crate::api_reflection_ledger::ProposalsResponse,
            crate::api_reflection_ledger::ReflectionsResponse,
            crate::api_reflection_ledger::ApproveProposalRequest,
            crate::api_reflection_ledger::ApproveProposalResponse,
            crate::api_reflection_ledger::RejectProposalRequest,
            crate::api_reflection_ledger::RejectProposalResponse,
            crate::fleet::ReflectionLedgerEntry,


            // Scripts API types
            crate::api_scripts::ScriptEntry,
            crate::api_scripts::ScriptRunRequest,
            crate::api_scripts::ScriptRunResponse,

            // Stitch Decompose API types
            crate::api_stitch_decompose::DecomposePreviewRequest,
            crate::api_stitch_decompose::DecomposePreviewResponse,
            crate::api_stitch_decompose::StitchPreviewData,
            crate::api_stitch_decompose::PredictionData,
            crate::api_stitch_decompose::RiskPatternMatch,
            crate::api_stitch_decompose::RiskPatternInfo,
            crate::api_stitch_decompose::SimilarStitchRef,
            crate::api_stitch_decompose::DedupMatchRef,
            crate::api_stitch_decompose::StitchSubmitRequest,
            crate::api_stitch_decompose::StitchSubmitResponse,
            crate::api_stitch_decompose::CreatedBead,
            crate::api_stitch_decompose::SubmitResult,
            crate::stitch_decompose::BeadGraph,

            // Stitch Links API types
            crate::api_stitch_links::CreateLinkRequest,
            crate::api_stitch_links::CreateLinkResponse,
            crate::api_stitch_links::SearchStitchesResponse,
            crate::api_stitch_links::StitchSearchResult,

            // Stitch Replay API types
            crate::api_stitch_replay::ReplayOptionsResponse,
            crate::api_stitch_replay::ResumeAsNewResponse,

            // Stitch Traversal API types
            crate::api_stitch_traversal::ParentsResponse,
            crate::api_stitch_traversal::ChildrenResponse,
            crate::api_stitch_traversal::ReferencedByResponse,
            crate::api_stitch_traversal::ClosureResponse,
            crate::api_stitch_traversal::StitchLinkInfo,
            crate::api_stitch_traversal::ClosureNodeInfo,

            // Timeline API types
            crate::api_timeline::TimelineResponse,
            crate::api_timeline::TimelineSegment,
            crate::api_timeline::WorkerTimeline,

            // Tour Project API types
            crate::api_tour_project::TourProjectResponse,
            crate::api_tour_project::TourStitchInfo,
            crate::api_tour_project::EnableTourRequest,

            // Transcription API types
            crate::api_transcription::ListJobsQuery,
            crate::transcription::JobStatus,
            crate::transcription::TranscriptionJob,

            // UI State API types
            crate::api_ui_state::UiStateResponse,
            crate::api_ui_state::UiStateUpdate,
            crate::api_ui_state::UiStateBatchUpdate,

            // Unassigned API types
            crate::api_unassigned::UnassignedSessionsResponse,
            crate::api_unassigned::UnassignedSession,
            crate::api_unassigned::AssignRequest,
            crate::api_unassigned::SuccessResponse,

            // Uploads API types
            crate::api_uploads::InitUploadRequest,
            crate::uploads::InitUploadResponse,
            crate::uploads::UploadProgressResponse,
            crate::uploads::UploadMetadata,

            // Diff API types
            crate::api_diff::DiffResponse,
            crate::api_diff::FileDiff,
            crate::api_diff::MergeBaseResponse,

            // Blame API types
            crate::api_blame::BlameLine,
        )
    ),
    tags(
        (name = "agent", description = "Agent session lifecycle control"),
        (name = "attachments", description = "File attachment serving"),
        (name = "audit", description = "Audit log queries and hash chain verification"),
        (name = "backup", description = "Fleet.db backup management"),
        (name = "bead_blockers", description = "Cross-workspace bead blocker resolution"),
        (name = "beads", description = "Bead creation, listing, and deduplication"),
        (name = "blame", description = "Git blame queries"),
        (name = "config", description = "Configuration access"),
        (name = "content_blocks", description = "Content block management"),
        (name = "conversations", description = "Fleet conversation listing"),
        (name = "cost", description = "Cost analysis and trends"),
        (name = "dictated_notes", description = "Voice dictation notes"),
        (name = "diff", description = "Git diff queries"),
        (name = "draft_queue", description = "Draft bead queue"),
        (name = "files", description = "Project file browsing and content"),
        (name = "fix_patterns", description = "Fix pattern suggestions"),
        (name = "metrics", description = "Metrics and monitoring"),
        (name = "morning_brief", description = "Morning brief generation"),
        (name = "onboarding", description = "Onboarding prompts and feature usage"),
        (name = "orphans", description = "Orphaned file detection"),
        (name = "patterns", description = "Pattern (multi-stitch) management"),
        (name = "presence", description = "Operator presence tracking"),
        (name = "preview", description = "Stitch prediction and preview"),
        (name = "prompts", description = "Prompt library"),
        (name = "reflections", description = "Reflection ledger and approval queue"),
        (name = "screen_capture", description = "Screen capture management"),
        (name = "scripts", description = "Scheduled script management"),
        (name = "stitch_decompose", description = "Stitch decomposition"),
        (name = "stitch_links", description = "Stitch relationship links"),
        (name = "stitch_read", description = "Stitch reading"),
        (name = "stitch_replay", description = "Stitch replay"),
        (name = "stitch_traversal", description = "Stitch graph traversal"),
        (name = "timeline", description = "Worker timeline queries"),
        (name = "tour_project", description = "Project tour mode"),
        (name = "transcription", description = "Audio transcription"),
        (name = "ui_state", description = "UI state persistence"),
        (name = "unassigned", description = "Unassigned session management"),
        (name = "uploads", description = "File upload handling"),
    )
)]
pub struct ApiDoc;

/// Build the OpenAPI router with all documentation endpoints
pub fn router() -> axum::Router<crate::DaemonState> {
    let openapi_json = ApiDoc::openapi();

    axum::Router::new()
        .route("/api/openapi.json", axum::routing::get(|| async { axum::Json(openapi_json) }))
        .route("/api/openapi.yaml", axum::routing::get(openapi_yaml_handler))
        .merge(SwaggerUi::new("/api/docs/swagger-ui").url("/api/openapi.json", openapi_json.clone()))
        .merge(Redoc::with_url("/api/docs/redoc", openapi_json.clone()))
        .merge(RapiDoc::with_openapi(openapi_json))
}

/// GET /api/openapi.yaml - Return the OpenAPI spec as YAML
async fn openapi_yaml_handler() -> impl axum::response::IntoResponse {
    use axum::http::header;
    let openapi = ApiDoc::openapi();
    let json = serde_json::to_value(&openapi).unwrap();
    let yaml = serde_yaml::to_string(&json).unwrap();

    (
        [(header::CONTENT_TYPE, "application/vnd.oai.openapi;version=3.0")],
        yaml,
    )
}
