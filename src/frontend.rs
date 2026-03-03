use axum::http::{header, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "frontend/dist"]
struct FrontendAssets;

/// Serve embedded frontend assets, with SPA fallback to index.html.
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try exact match first
    if let Some(content) = FrontendAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            content.data.into_owned(),
        )
            .into_response()
    } else {
        // SPA fallback: serve index.html for any non-API route
        match FrontendAssets::get("index.html") {
            Some(content) => Html(
                String::from_utf8_lossy(&content.data).into_owned(),
            )
            .into_response(),
            None => (StatusCode::NOT_FOUND, "frontend not built").into_response(),
        }
    }
}
