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
#[derive(OpenApi)]
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
    paths(
        // Agent API
        crate::api_agent::get_status,
        crate::api_agent::spawn_session,
        crate::api_agent::disable_agent,
        crate::api_agent::switch_adapter,
        crate::api_agent::send_turn,
        crate::api_agent::list_sessions,

        // Beads API
        crate::api_beads::list_open_beads,
        crate::api_beads::create_bead,
        crate::api_beads::check_dedup,
        crate::api_beads::dismiss_dedup,
        crate::api_beads::query_vector_index,
        crate::api_beads::get_vector_index_stats,

        // Audit API
        crate::api_audit::list_audit_rows,
        crate::api_audit::verify_hash_chain,
        crate::api_audit::query_redaction_audit,

        // Attachments API
        crate::api_attachments::serve_attachment,

        // Config API
        crate::api_config::get_config,
        crate::api_config::get_secrets_patterns,

        // Dictated Notes API
        crate::api_dictated_notes::create_note,
        crate::api_dictated_notes::list_notes,
        crate::api_dictated_notes::get_note,
        crate::api_dictated_notes::update_note,
        crate::api_dictated_notes::redact_words,
        crate::api_dictated_notes::get_audio,
        crate::api_dictated_notes::synthesize_draft,
        crate::api_dictated_notes::create_draft_from_note,

        // Draft Queue API
        crate::api_draft_queue::list_all_drafts,
        crate::api_draft_queue::list_project_drafts,
        crate::api_draft_queue::get_draft,
        crate::api_draft_queue::create_draft,
        crate::api_draft_queue::approve_draft,
        crate::api_draft_queue::edit_draft,
        crate::api_draft_queue::reject_draft,
        crate::api_draft_queue::open_draft,
        crate::api_draft_queue::autosave_draft,
        crate::api_draft_queue::abandon_draft,
        crate::api_draft_queue::get_dedup_stats,
        crate::api_draft_queue::report_false_positive,

        // Morning Brief API
        crate::api_morning_brief::get_latest,
        crate::api_morning_brief::list_briefs,
        crate::api_morning_brief::trigger_brief,
        crate::api_morning_brief::get_status,

        // Preview API
        crate::api_preview::preview_bead,

        // Stitch Decompose API
        crate::api_stitch_decompose::preview_decompose,
        crate::api_stitch_decompose::submit_stitch,

        // Transcription API
        crate::api_transcription::get_job,
        crate::api_transcription::list_jobs,

        // Uploads API
        crate::api_uploads::init_upload,
        crate::api_uploads::upload_chunk,
        crate::api_uploads::get_progress,
        crate::api_uploads::complete_upload,
        crate::api_uploads::cancel_upload,
    ),
    components(
        schemas(
            // Agent API types
            crate::agent_session::AgentSessionStatus,
            crate::api_agent::SpawnSessionResponse,
            crate::api_agent::DisableAgentResponse,
            crate::api_agent::SwitchAdapterResponse,
            crate::api_agent::SendTurnResponse,
            crate::api_agent::SwitchRequest,
            crate::api_agent::TurnRequest,
            crate::api_agent::TurnAttachment,
            crate::fleet::AgentSessionRow,

            // Beads API types
            crate::api_beads::BeadSummary,
            crate::api_beads::CreateBeadRequest,
            crate::api_beads::CreateBeadResponse,
            crate::api_beads::DedupCheckRequest,
            crate::api_beads::DedupCheckResponse,
            crate::api_beads::DedupMatchRef,
            crate::api_beads::VectorIndexStats,

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

            // Morning Brief API types
            crate::api_morning_brief::TriggerResponse,
            crate::api_morning_brief::StatusResponse,
            crate::fleet::MorningBriefRow,

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

            // Transcription API types
            crate::api_transcription::ListJobsQuery,
            crate::transcription::JobStatus,
            crate::transcription::TranscriptionJob,

            // Uploads API types
            crate::api_uploads::InitUploadRequest,
            crate::uploads::InitUploadResponse,
            crate::uploads::UploadProgressResponse,
            crate::uploads::UploadMetadata,
        )
    ),
    tags(
        (name = "agent", description = "Agent session lifecycle control"),
        (name = "beads", description = "Bead creation, listing, and deduplication"),
        (name = "audit", description = "Audit log queries and hash chain verification"),
        (name = "attachments", description = "File attachment serving"),
        (name = "config", description = "Configuration access"),
        (name = "dictated_notes", description = "Voice dictation notes"),
        (name = "draft_queue", description = "Draft bead queue"),
        (name = "metrics", description = "Metrics and monitoring"),
        (name = "morning_brief", description = "Morning brief generation"),
        (name = "patterns", description = "Pattern (multi-stitch) management"),
        (name = "preview", description = "Stitch prediction and preview"),
        (name = "scripts", description = "Scheduled script management"),
        (name = "stitch_decompose", description = "Stitch decomposition"),
        (name = "stitch_links", description = "Stitch relationship links"),
        (name = "stitch_read", description = "Stitch reading"),
        (name = "stitch_replay", description = "Stitch replay"),
        (name = "transcription", description = "Audio transcription"),
        (name = "uploads", description = "File upload handling"),
        (name = "backup", description = "Fleet.db backup management"),
        (name = "diff", description = "Git diff queries"),
        (name = "blame", description = "Git blame queries"),
        (name = "orphans", description = "Orphaned file detection"),
        (name = "fix_patterns", description = "Fix pattern suggestions"),
        (name = "timeline", description = "Worker timeline queries"),
        (name = "screen_capture", description = "Screen capture"),
        (name = "net_diff", description = "Network diff queries"),
        (name = "templates", description = "Stitch template library"),
        (name = "prompts", description = "Prompt library"),
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
        .merge(RapiDoc::new("/api/docs/rapidoc").url("/api/openapi.json", openapi_json))
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
