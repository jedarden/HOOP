//! Integration tests for mutation handler pattern (arch-patterns: reject = error + rebroadcast)
//!
//! These tests verify the acceptance criteria:
//! - Reject scenarios in tests: field validation fail, auth fail, contention
//! - All produce error + full rebroadcast
//! - Client's reducer unchanged for reject vs accept
//!
//! Per the architecture patterns convention (docs/notes/architecture-patterns.md):
//! "Reject paths are the least-tested code. Reusing the happy path (receive
//! authoritative state) keeps them honest."

use hoop_daemon::mutation_handler::{ErrorKind, MutationHandler, MutationReject, MutationResult, WithRejectError};
use hoop_daemon::ws::{BeadData, ConfigErrorData, DraftUpdateData};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock authoritative state for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct MockDraftState {
    id: String,
    title: String,
    status: String,
    version: i64,
    /// Error field only present on rejection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ConfigErrorData>,
}

impl WithRejectError for MockDraftState {
    fn with_error(mut self, error: ConfigErrorData) -> Self {
        self.error = Some(error);
        self
    }
}

impl MockDraftState {
    fn new(id: &str, title: &str, status: &str, version: i64) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            status: status.to_string(),
            version,
            error: None,
        }
    }

    /// Add error details to the state (what the client sees on reject)
    fn with_error_reject(mut self, reject: &MutationReject) -> Self {
        self.error = Some(reject.to_config_error());
        self
    }
}

/// Mock mutation service that uses the mutation handler pattern
struct MockDraftService {
    /// Current authoritative state
    state: Arc<RwLock<MockDraftState>>,
    /// Broadcast sender for WS events
    tx: tokio::sync::broadcast::Sender<MockDraftState>,
    /// Actor for mutations
    actor: String,
}

impl MockDraftService {
    fn new(actor: &str) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(16);
        Self {
            state: Arc::new(RwLock::new(MockDraftState::new("", "", "pending", 1))),
            tx,
            actor: actor.to_string(),
        }
    }

    /// Get current state (what client would have)
    async fn current_state(&self) -> MockDraftState {
        self.state.read().await.clone()
    }

    /// Subscribe to WS events (simulates client connection)
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<MockDraftState> {
        self.tx.subscribe()
    }

    /// Mutation: approve draft (happy path)
    async fn approve_draft(&self, draft_id: &str, title: &str) -> MutationResult<MockDraftState> {
        let handler = MutationHandler::new(&self.tx, "draft", self.actor.clone());

        // Get current state
        let current = self.state.read().await.clone();

        // Validation: title must not be empty
        if title.trim().is_empty() {
            let reject = MutationReject::validation(
                "title",
                "Title cannot be empty",
                Some("non-empty string"),
                Some("empty or whitespace"),
            )
            .with_entity_id(draft_id);

            // REJECT: emit error + authoritative state (no "reject" event)
            handler.reject(reject.clone(), current).await;
            return Err(reject);
        }

        // Auth: check if actor has permission
        if self.actor == "unauthorized_user" {
            let reject = MutationReject::auth(
                draft_id,
                "insufficient_permissions",
                "You don't have permission to approve drafts",
            );

            // REJECT: emit error + authoritative state
            handler.reject(reject.clone(), current).await;
            return Err(reject);
        }

        // Contention: check if already approved
        if current.status == "approved" {
            let reject = MutationReject::contention(
                draft_id,
                "already_approved",
                "Draft was already approved by another user",
            );

            // REJECT: emit error + authoritative state
            handler.reject(reject.clone(), current).await;
            return Err(reject);
        }

        // SUCCESS: update state and broadcast
        let updated = MockDraftState::new(draft_id, title, "approved", current.version + 1);
        *self.state.write().await = updated.clone();

        // ACCEPT: emit authoritative state (same data path as reject)
        handler.accept(updated.clone()).await;

        Ok(updated)
    }
}

// ---------------------------------------------------------------------------
// Test: field validation failure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_reject_field_validation_emits_error_and_authoritative_state() {
    let service = MockDraftService::new("test-user");
    let mut rx = service.subscribe();

    // Initialize state
    *service.state.write().await = MockDraftState::new("draft-123", "Valid Title", "pending", 1);

    // Attempt to approve with empty title (validation failure)
    let result = service.approve_draft("draft-123", "   ").await;

    // Verify rejection
    assert!(result.is_err());
    let reject = result.unwrap_err();
    assert_eq!(reject.kind, ErrorKind::Validation);
    assert_eq!(reject.field, Some("title".to_string()));
    assert!(reject.message.contains("empty"));

    // Verify WS broadcast: error + authoritative state
    let broadcast_state = rx.recv().await.unwrap();
    assert_eq!(broadcast_state.id, "draft-123");
    assert_eq!(broadcast_state.status, "pending"); // State unchanged
    assert_eq!(broadcast_state.version, 1); // Version unchanged
    assert!(broadcast_state.error.is_some(), "Should include error in broadcast state");

    let error = broadcast_state.error.unwrap();
    assert_eq!(error.field, Some("title".to_string()));
    assert!(error.message.contains("empty"));

    // Verify server state unchanged
    let server_state = service.current_state().await;
    assert_eq!(server_state.status, "pending");
    assert_eq!(server_state.version, 1);
}

// ---------------------------------------------------------------------------
// Test: auth failure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_reject_auth_failure_emits_error_and_authoritative_state() {
    let service = MockDraftService::new("unauthorized_user");
    let mut rx = service.subscribe();

    // Initialize state
    *service.state.write().await = MockDraftState::new("draft-456", "Valid Title", "pending", 1);

    // Attempt to approve without permission
    let result = service.approve_draft("draft-456", "Valid Title").await;

    // Verify rejection
    assert!(result.is_err());
    let reject = result.unwrap_err();
    assert_eq!(reject.kind, ErrorKind::Auth);
    assert!(reject.field.unwrap().starts_with("auth:"));
    assert!(reject.message.contains("permission"));

    // Verify WS broadcast: error + authoritative state
    let broadcast_state = rx.recv().await.unwrap();
    assert_eq!(broadcast_state.id, "draft-456");
    assert_eq!(broadcast_state.status, "pending"); // State unchanged
    assert!(broadcast_state.error.is_some());

    let error = broadcast_state.error.unwrap();
    assert!(error.field.unwrap().starts_with("auth:"));
    assert!(error.message.contains("permission"));
}

// ---------------------------------------------------------------------------
// Test: contention (already approved by another user)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_reject_contention_emits_error_and_authoritative_state() {
    let service = MockDraftService::new("test-user");
    let mut rx = service.subscribe();

    // Initialize state as already approved
    *service.state.write().await = MockDraftState::new("draft-789", "Some Title", "approved", 2);

    // Attempt to approve again (contention)
    let result = service.approve_draft("draft-789", "Updated Title").await;

    // Verify rejection
    assert!(result.is_err());
    let reject = result.unwrap_err();
    assert_eq!(reject.kind, ErrorKind::Contention);
    assert!(reject.field.unwrap().starts_with("contention:"));
    assert!(reject.message.contains("already approved"));

    // Verify WS broadcast: error + authoritative state
    let broadcast_state = rx.recv().await.unwrap();
    assert_eq!(broadcast_state.id, "draft-789");
    assert_eq!(broadcast_state.status, "approved"); // Current state unchanged
    assert_eq!(broadcast_state.version, 2); // Version unchanged
    assert!(broadcast_state.error.is_some());

    let error = broadcast_state.error.unwrap();
    assert!(error.field.unwrap().starts_with("contention:"));
    assert!(error.message.contains("already approved"));
}

// ---------------------------------------------------------------------------
// Test: client reducer unchanged for reject vs accept (single rendering path)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_client_reducer_single_rendering_path_accept_vs_reject() {
    /// Simulates the client-side reducer
    ///
    /// Per arch-patterns: "Client's state update path is the same whether the
    /// update was accepted or rejected."
    fn client_reducer(current: MockDraftState, incoming: MockDraftState) -> MockDraftState {
        // The reducer simply applies the incoming state - no special logic for reject
        incoming
    }

    let service = MockDraftService::new("test-user");
    let mut rx = service.subscribe();

    // Test accept path
    *service.state.write().await = MockDraftState::new("draft-1", "Title A", "pending", 1);
    let client_state_before = service.current_state().await;

    service.approve_draft("draft-1", "Title A").await.unwrap();
    let accept_event = rx.recv().await.unwrap();

    let client_state_after_accept = client_reducer(client_state_before, accept_event);
    assert_eq!(client_state_after_accept.status, "approved");
    assert!(client_state_after_accept.error.is_none());

    // Test reject path (same reducer logic)
    *service.state.write().await = MockDraftState::new("draft-2", "Title B", "pending", 1);
    let client_state_before = service.current_state().await;

    let result = service.approve_draft("draft-2", "").await;
    assert!(result.is_err(), "Should reject empty title");
    let reject_event = rx.recv().await.unwrap();

    let client_state_after_reject = client_reducer(client_state_before, reject_event);
    assert_eq!(client_state_after_reject.status, "pending"); // State unchanged
    assert!(client_state_after_reject.error.is_some(), "Error present for UI to display");

    // Verify: same reducer, different outcome based on incoming state
    // No conditional logic needed for "reject" vs "accept"
}

// ---------------------------------------------------------------------------
// Test: multiple rejection scenarios in sequence
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_multiple_rejections_each_emit_error_and_state() {
    let service = MockDraftService::new("test-user");
    let mut rx = service.subscribe();

    // Initialize state
    *service.state.write().await = MockDraftState::new("draft-seq", "Original", "pending", 1);

    // Rejection 1: validation failure (empty title)
    let result = service.approve_draft("draft-seq", "").await;
    assert!(result.is_err());
    let event1 = rx.recv().await.unwrap();
    assert!(event1.error.is_some());
    assert_eq!(event1.status, "pending"); // Unchanged

    // Rejection 2: validation failure (whitespace only)
    let result = service.approve_draft("draft-seq", "   ").await;
    assert!(result.is_err());
    let event2 = rx.recv().await.unwrap();
    assert!(event2.error.is_some());
    assert_eq!(event2.status, "pending"); // Still unchanged

    // Rejection 3: switch to unauthorized user
    service.actor = "unauthorized_user".to_string();
    let result = service.approve_draft("draft-seq", "Valid Title").await;
    assert!(result.is_err());
    let event3 = rx.recv().await.unwrap();
    assert!(event3.error.is_some());
    assert_eq!(event3.status, "pending"); // Still unchanged

    // Verify each error has appropriate kind
    assert_eq!(event1.error.as_ref().unwrap().message, "Title cannot be empty");
    assert_eq!(event2.error.as_ref().unwrap().message, "Title cannot be empty");
    assert!(event3.error.as_ref().unwrap().message.contains("permission"));
}

// ---------------------------------------------------------------------------
// Test: acceptance after rejections
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_acceptance_after_rejections_follows_same_path() {
    let service = MockDraftService::new("authorized_user");
    let mut rx = service.subscribe();

    // Initialize state
    *service.state.write().await = MockDraftState::new("draft-final", "Final Title", "pending", 1);

    // First attempt: validation failure
    let result = service.approve_draft("draft-final", "").await;
    assert!(result.is_err());
    let reject_event = rx.recv().await.unwrap();
    assert!(reject_event.error.is_some());

    // Second attempt: success
    let result = service.approve_draft("draft-final", "Final Title").await;
    assert!(result.is_ok());
    let accept_event = rx.recv().await.unwrap();
    assert!(accept_event.error.is_none());

    // Both events use the same structure
    assert_eq!(reject_event.id, accept_event.id);
    assert_eq!(reject_event.title, accept_event.title);

    // Client reducer applies both the same way
}

// ---------------------------------------------------------------------------
// Test: rejection with all error details populated
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_reject_populates_all_error_details() {
    let service = MockDraftService::new("test-user");
    let mut rx = service.subscribe();

    *service.state.write().await = MockDraftState::new("draft-details", "Title", "pending", 1);

    // Validation rejection with all fields
    let reject = MutationReject::validation(
        "priority",
        "Priority must be positive",
        Some("positive integer"),
        Some("-5"),
    )
    .with_entity_id("draft-details");

    let handler = MutationHandler::new(&service.tx, "draft", "test-user".to_string());
    handler.reject(reject.clone(), service.current_state().await).await;

    let event = rx.recv().await.unwrap();
    let error = event.error.unwrap();

    assert_eq!(error.message, "Priority must be positive");
    assert_eq!(error.field, Some("priority".to_string()));
    assert_eq!(error.expected, Some("positive integer".to_string()));
    assert_eq!(error.got, Some("-5".to_string()));
}

// ---------------------------------------------------------------------------
// Test: internal error rejection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_reject_internal_error() {
    let service = MockDraftService::new("test-user");
    let mut rx = service.subscribe();

    *service.state.write().await = MockDraftState::new("draft-internal", "Title", "pending", 1);

    let reject = MutationReject::internal(
        "draft-internal",
        "Database connection lost during approval",
    );

    let handler = MutationHandler::new(&service.tx, "draft", "test-user".to_string());
    handler.reject(reject, service.current_state().await).await;

    let event = rx.recv().await.unwrap();
    let error = event.error.unwrap();

    assert_eq!(error.kind, ErrorKind::Internal);
    assert!(error.message.contains("Database"));
    assert!(error.field.is_none()); // Internal errors don't have field details
}
