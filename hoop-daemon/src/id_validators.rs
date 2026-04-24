//! ID validators — re-exported from `hoop_schema::id_validators`.
//!
//! This module adds axum-specific helpers (`rejection`) on top of the canonical
//! validators that live in the shared `hoop-schema` crate.

pub use hoop_schema::id_validators::*;

use axum::http::StatusCode;

/// Convert an `IdValidationError` into an axum HTTP error response.
pub fn rejection(err: IdValidationError) -> (StatusCode, String) {
    (
        StatusCode::BAD_REQUEST,
        format!("Invalid {} parameter", err.kind),
    )
}
