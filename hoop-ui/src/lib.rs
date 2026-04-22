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

/// Get MIME type for a file path
fn mime_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "html" => "text/html",
        "js" => "application/javascript",
        "mjs" => "application/javascript",
        "css" => "text/css",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

/// Serve an embedded static file
///
/// For SPA routing: if the path doesn't match an asset file, serve index.html.
/// This allows client-side routing to work properly.
async fn serve_asset(AxumPath(path): AxumPath<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    // Try to get the exact asset first
    let asset = Assets::get(path);

    match asset {
        Some(content) => {
            let mime = mime_type(path);
            let mut response = Response::new(Body::from(content.data.to_vec()));
            response.headers_mut().insert(
                "content-type",
                mime.parse().unwrap(),
            );
            response
        }
        None => {
            // For SPA routing, serve index.html for non-asset paths
            let index = Assets::get("index.html");
            match index {
                Some(content) => {
                    let mut response = Response::new(Body::from(content.data.to_vec()));
                    response.headers_mut().insert(
                        "content-type",
                        "text/html".parse().unwrap(),
                    );
                    response
                }
                None => axum::http::StatusCode::NOT_FOUND.into_response(),
            }
        }
    }
}

impl AssetsHandler {
    /// Create the assets router
    pub fn router() -> Router {
        Router::new().route("/*path", get(serve_asset))
    }
}
