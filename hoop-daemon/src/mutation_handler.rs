//! Mutation handler template — enforces "reject = error + rebroadcast" pattern (arch-patterns)
//!
//! Per the architecture patterns convention (docs/notes/architecture-patterns.md):
//! - Reject paths should never emit a "reject" event
//! - Instead: emit `error` + fresh authoritative state
//! - Single rendering path on the client (accept vs reject handled identically)

use crate::ws::ConfigErrorData;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{info, warn};

/// Trait for state types that can carry rejection error details.
///
/// Per arch-patterns: "Reject paths emit error + authoritative state."
/// States that participate in mutations must implement this trait to allow
/// the mutation handler to embed error details before broadcasting.
pub trait WithRejectError: Clone {
    /// Return a new instance of self with the error field populated.
    fn with_error(self, error: ConfigErrorData) -> Self;
}

/// Result type for mutation handlers
pub type MutationResult<T> = Result<T, MutationReject>;

/// Rejection reason with structured details for WS broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationReject {
    /// Human-readable error message
    pub message: String,
    /// Error kind: "validation", "auth", "contention", "internal"
    pub kind: ErrorKind,
    /// Entity ID being mutated (for audit/logging)
    pub entity_id: String,
    /// Dotted path to the offending field (e.g. "draft.title")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// What was expected (e.g. "string", "field present")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// What was actually found (e.g. "integer", "missing")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub got: Option<String>,
}

/// Error kind for categorizing rejections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Validation,
    Auth,
    Contention,
    Internal,
}

impl MutationReject {
    pub fn validation(
        field: impl Into<String>,
        message: impl Into<String>,
        expected: Option<impl Into<String>>,
        got: Option<impl Into<String>>,
    ) -> Self {
        let field_str = field.into();
        Self {
            message: message.into(),
            kind: ErrorKind::Validation,
            entity_id: String::new(),
            field: Some(field_str.clone()),
            expected: expected.map(|s| s.into()),
            got: got.map(|s| s.into()),
        }
    }

    pub fn auth(
        entity_id: impl Into<String>,
        reason: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            kind: ErrorKind::Auth,
            entity_id: entity_id.into(),
            field: Some(format!("auth:{}", reason.into())),
            expected: Some("authorized".to_string()),
            got: Some("unauthorized".to_string()),
        }
    }

    pub fn contention(
        entity_id: impl Into<String>,
        conflict_type: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            kind: ErrorKind::Contention,
            entity_id: entity_id.into(),
            field: Some(format!("contention:{}", conflict_type.into())),
            expected: Some("no conflict".to_string()),
            got: Some("conflict".to_string()),
        }
    }

    pub fn internal(entity_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ErrorKind::Internal,
            entity_id: entity_id.into(),
            field: None,
            expected: None,
            got: None,
        }
    }

    pub fn with_entity_id(mut self, id: impl Into<String>) -> Self {
        self.entity_id = id.into();
        self
    }

    pub fn to_config_error(&self) -> ConfigErrorData {
        ConfigErrorData {
            message: self.message.clone(),
            line: 0,
            col: 0,
            field: self.field.clone(),
            expected: self.expected.clone(),
            got: self.got.clone(),
        }
    }
}

impl fmt::Display for MutationReject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.kind, self.message)
    }
}

impl std::error::Error for MutationReject {}

impl From<MutationReject> for (axum::http::StatusCode, String) {
    fn from(reject: MutationReject) -> Self {
        let status = match reject.kind {
            ErrorKind::Validation => axum::http::StatusCode::BAD_REQUEST,
            ErrorKind::Auth => axum::http::StatusCode::FORBIDDEN,
            ErrorKind::Contention => axum::http::StatusCode::CONFLICT,
            ErrorKind::Internal => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, reject.message)
    }
}

pub struct MutationHandler<'a, T>
where
    T: Clone + WithRejectError + serde::Serialize + Send + Sync,
{
    tx: &'a tokio::sync::broadcast::Sender<T>,
    entity_type: &'a str,
    actor: String,
}

impl<'a, T> MutationHandler<'a, T>
where
    T: Clone + WithRejectError + serde::Serialize + Send + Sync,
{
    pub fn new(
        tx: &'a tokio::sync::broadcast::Sender<T>,
        entity_type: &'a str,
        actor: String,
    ) -> Self {
        Self {
            tx,
            entity_type,
            actor,
        }
    }

    pub async fn accept(&self, event: T) {
        info!(
            entity_type = %self.entity_type,
            actor = %self.actor,
            "Mutation accepted"
        );
        let _ = self.tx.send(event);
    }

    pub async fn reject(&self, reject: MutationReject, state_snapshot: T) {
        warn!(
            entity_type = %self.entity_type,
            actor = %self.actor,
            error = %reject.message,
            kind = ?reject.kind,
            entity_id = %reject.entity_id,
            "Mutation rejected"
        );
        // Per arch-patterns: emit error + authoritative state (no "reject" event)
        let state_with_error = state_snapshot.with_error(reject.to_config_error());
        let _ = self.tx.send(state_with_error);
    }
}

pub trait MutationHandlerExt<T>
where
    T: Clone + WithRejectError + serde::Serialize + Send + Sync,
{
    fn mutation_handler(&self, entity_type: &str, actor: String) -> MutationHandler<T>;
}

impl<T> MutationHandlerExt<T> for tokio::sync::broadcast::Sender<T>
where
    T: Clone + WithRejectError + serde::Serialize + Send + Sync,
{
    fn mutation_handler(&self, entity_type: &str, actor: String) -> MutationHandler<T> {
        MutationHandler::new(self, entity_type, actor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// Mock state for unit tests
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct MockState {
        pub value: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub error: Option<ConfigErrorData>,
    }

    impl WithRejectError for MockState {
        fn with_error(mut self, error: ConfigErrorData) -> Self {
            self.error = Some(error);
            self
        }
    }

    #[test]
    fn test_mutation_reject_validation() {
        let reject = MutationReject::validation(
            "title",
            "Title is required",
            Some("non-empty string"),
            Some("missing"),
        )
        .with_entity_id("draft-123");

        assert_eq!(reject.kind, ErrorKind::Validation);
        assert_eq!(reject.message, "Title is required");
        assert_eq!(reject.field, Some("title".to_string()));
        assert_eq!(reject.expected, Some("non-empty string".to_string()));
        assert_eq!(reject.got, Some("missing".to_string()));
        assert_eq!(reject.entity_id, "draft-123");
    }

    #[test]
    fn test_mutation_reject_auth() {
        let reject = MutationReject::auth(
            "draft-123",
            "insufficient_permissions",
            "You don't have permission to approve this draft",
        );

        assert_eq!(reject.kind, ErrorKind::Auth);
        assert_eq!(reject.entity_id, "draft-123");
        assert_eq!(
            reject.field,
            Some("auth:insufficient_permissions".to_string())
        );
    }

    #[test]
    fn test_mutation_reject_contention() {
        let reject = MutationReject::contention(
            "draft-123",
            "already_approved",
            "Draft was already approved by another user",
        );

        assert_eq!(reject.kind, ErrorKind::Contention);
        assert_eq!(reject.entity_id, "draft-123");
        assert_eq!(
            reject.field,
            Some("contention:already_approved".to_string())
        );
    }

    #[test]
    fn test_mutation_reject_internal() {
        let reject = MutationReject::internal("draft-123", "Database connection failed");

        assert_eq!(reject.kind, ErrorKind::Internal);
        assert_eq!(reject.entity_id, "draft-123");
        assert_eq!(reject.message, "Database connection failed");
        assert!(reject.field.is_none());
    }

    #[test]
    fn test_mutation_reject_to_config_error() {
        let reject = MutationReject::validation(
            "priority",
            "Priority must be positive",
            Some("positive integer"),
            Some("-1"),
        );

        let error = reject.to_config_error();
        assert_eq!(error.message, "Priority must be positive");
        assert_eq!(error.field, Some("priority".to_string()));
        assert_eq!(error.expected, Some("positive integer".to_string()));
        assert_eq!(error.got, Some("-1".to_string()));
    }

    #[test]
    fn test_mutation_reject_to_http_response() {
        let validation =
            MutationReject::validation("title", "Bad title", None::<String>, None::<String>);
        let (status, msg) = <(axum::http::StatusCode, String)>::from(validation.clone());
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(msg, "Bad title");

        let auth = MutationReject::auth("id", "forbidden", "Not allowed");
        let (status, msg) = <(axum::http::StatusCode, String)>::from(auth);
        assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
        assert_eq!(msg, "Not allowed");

        let contention = MutationReject::contention("id", "conflict", "Already modified");
        let (status, msg) = <(axum::http::StatusCode, String)>::from(contention);
        assert_eq!(status, axum::http::StatusCode::CONFLICT);
        assert_eq!(msg, "Already modified");

        let internal = MutationReject::internal("id", "Server error");
        let (status, msg) = <(axum::http::StatusCode, String)>::from(internal);
        assert_eq!(status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(msg, "Server error");
    }

    #[test]
    fn test_mutation_reject_display() {
        let reject =
            MutationReject::validation("title", "Title required", None::<String>, None::<String>);
        assert_eq!(format!("{}", reject), "[Validation] Title required");

        let reject = MutationReject::auth("id", "forbidden", "Not allowed");
        assert_eq!(format!("{}", reject), "[Auth] Not allowed");

        let reject = MutationReject::contention("id", "conflict", "Conflict");
        assert_eq!(format!("{}", reject), "[Contention] Conflict");
    }

    #[tokio::test]
    async fn test_mutation_handler_accept() {
        let (tx, mut rx) = tokio::sync::broadcast::channel::<MockState>(16);
        let handler = MutationHandler::new(&tx, "test_draft", "test-user".to_string());

        let state = MockState {
            value: "success".to_string(),
            error: None,
        };
        handler.accept(state.clone()).await;

        let event = rx.recv().await.unwrap();
        assert_eq!(event.value, "success");
        assert!(event.error.is_none());
    }

    #[tokio::test]
    async fn test_mutation_handler_reject() {
        let (tx, mut rx) = tokio::sync::broadcast::channel::<MockState>(16);
        let handler = MutationHandler::new(&tx, "test_draft", "test-user".to_string());

        let reject = MutationReject::validation(
            "title",
            "Title is required",
            Some("non-empty string"),
            Some("missing"),
        )
        .with_entity_id("draft-123");

        let state = MockState {
            value: "authoritative_state".to_string(),
            error: None,
        };
        handler.reject(reject, state).await;

        let event = rx.recv().await.unwrap();
        assert_eq!(event.value, "authoritative_state");
        assert!(event.error.is_some());
        assert_eq!(event.error.as_ref().unwrap().message, "Title is required");
        assert_eq!(
            event.error.as_ref().unwrap().field,
            Some("title".to_string())
        );
    }

    #[tokio::test]
    async fn test_mutation_handler_ext() {
        let (tx, mut rx) = tokio::sync::broadcast::channel::<MockState>(16);

        let handler = tx.mutation_handler("test_entity", "test-user".to_string());
        assert_eq!(handler.entity_type, "test_entity");
        assert_eq!(handler.actor, "test-user");

        let state = MockState {
            value: "test_event".to_string(),
            error: None,
        };
        handler.accept(state).await;
        let event = rx.recv().await.unwrap();
        assert_eq!(event.value, "test_event");
    }
}
