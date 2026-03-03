use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::error::AppError;
use crate::fs::guard;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct FolderQuery {
    pub path: String,
}

#[derive(Deserialize)]
pub struct CreateFolderBody {
    pub path: String,
}

#[derive(Deserialize)]
pub struct RenameFolderBody {
    pub from: String,
    pub to: String,
}

/// POST /api/folder — create folder.
pub async fn create_folder(
    State(state): State<AppState>,
    Json(body): Json<CreateFolderBody>,
) -> Result<StatusCode, AppError> {
    let resolved = guard::resolve_path(&state.root, &body.path)?;

    if resolved.exists() {
        return Err(AppError::Conflict(format!("{} already exists", body.path)));
    }

    tokio::fs::create_dir_all(&resolved).await?;
    Ok(StatusCode::CREATED)
}

/// DELETE /api/folder?path=... — delete folder recursively.
pub async fn delete_folder(
    State(state): State<AppState>,
    Query(query): Query<FolderQuery>,
) -> Result<StatusCode, AppError> {
    let resolved = guard::resolve_path(&state.root, &query.path)?;

    if !resolved.is_dir() {
        return Err(AppError::NotFound(format!(
            "{} is not a directory",
            query.path
        )));
    }

    // Prevent deleting the root
    let root_canon = dunce::canonicalize(&state.root)
        .map_err(|e| AppError::Internal(e.into()))?;
    if resolved == root_canon {
        return Err(AppError::Forbidden("cannot delete root directory".into()));
    }

    tokio::fs::remove_dir_all(&resolved).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/folder/rename — rename/move folder.
pub async fn rename_folder(
    State(state): State<AppState>,
    Json(body): Json<RenameFolderBody>,
) -> Result<StatusCode, AppError> {
    let from = guard::resolve_path(&state.root, &body.from)?;
    let to = guard::resolve_path(&state.root, &body.to)?;

    if !from.is_dir() {
        return Err(AppError::NotFound(format!(
            "{} is not a directory",
            body.from
        )));
    }
    if to.exists() {
        return Err(AppError::Conflict(format!("{} already exists", body.to)));
    }

    if let Some(parent) = to.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::rename(&from, &to).await?;
    Ok(StatusCode::NO_CONTENT)
}
