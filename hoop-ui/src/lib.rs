//! Embedded static assets for the HOOP web UI
//!
//! Phase 1 UI will ship as embedded static assets served by the daemon.

use axum::{
    body::Body,
    extract::Path as AxumPath,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;

/// Embedded static assets
#[derive(RustEmbed)]
#[folder = "static/"]
#[prefix = "/"]
struct Assets;

/// Assets handler module
pub struct AssetsHandler;

/// Serve an embedded static file
async fn serve_asset(AxumPath(path): AxumPath<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    let asset = Assets::get(path).or_else(|| Assets::get("index.html"));

    match asset {
        Some(content) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();

            let mut response = Response::new(Body::from(content.data.to_vec()));
            response.headers_mut().insert(
                "content-type",
                mime.parse().unwrap_or_else(|_| axum::http::HeaderValue::from_static("application/octet-stream")),
            );
            response
        }
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

impl AssetsHandler {
    /// Create the assets router
    pub fn router() -> Router {
        Router::new().route("/*path", get(serve_asset))
    }
}
