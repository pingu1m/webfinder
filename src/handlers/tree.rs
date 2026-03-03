use axum::extract::State;
use axum::Json;

use crate::fs::walk::FileNode;
use crate::state::AppState;

/// GET /api/tree — serve the cached tree (no filesystem I/O).
pub async fn get_tree(State(state): State<AppState>) -> Json<Vec<FileNode>> {
    let tree = state.tree_cache.read().await;
    Json(tree.clone())
}
