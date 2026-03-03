use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::frontend::static_handler;
use crate::handlers;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        // Tree
        .route("/api/tree", get(handlers::tree::get_tree))
        // Files
        .route("/api/file", get(handlers::file::get_file))
        .route("/api/file", put(handlers::file::put_file))
        .route("/api/file", post(handlers::file::create_file))
        .route("/api/file", delete(handlers::file::delete_file))
        .route("/api/file/rename", post(handlers::file::rename_file))
        .route("/api/file/copy", post(handlers::file::copy_file))
        // Folders
        .route("/api/folder", post(handlers::folder::create_folder))
        .route("/api/folder", delete(handlers::folder::delete_folder))
        .route("/api/folder/rename", post(handlers::folder::rename_folder))
        // Search
        .route("/api/search", get(handlers::search::search))
        // Info
        .route("/api/info", get(handlers::info::get_info))
        // Runner
        .route("/api/run", post(handlers::run::start_run))
        .route("/api/run/{id}", delete(handlers::run::stop_run))
        .route("/api/run/{id}", get(handlers::run::get_run_status))
        // WebSocket
        .route("/api/watch", get(handlers::watch::watch_ws))
        .route("/api/run/{id}/stream", get(handlers::watch::run_stream_ws))
        .with_state(state);

    Router::new()
        .merge(api)
        .fallback(static_handler)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
