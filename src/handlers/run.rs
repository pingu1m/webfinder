use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::fs::guard;
use crate::runner::process::spawn_runner;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RunBody {
    pub path: String,
}

#[derive(Serialize)]
pub struct RunResponse {
    pub id: String,
}

#[derive(Serialize)]
pub struct RunStatusResponse {
    pub id: String,
    pub running: bool,
    pub exit_code: Option<i32>,
}

/// POST /api/run — start running a file.
pub async fn start_run(
    State(state): State<AppState>,
    Json(body): Json<RunBody>,
) -> Result<(StatusCode, Json<RunResponse>), AppError> {
    let resolved = guard::resolve_path(&state.root, &body.path)?;

    if !resolved.is_file() {
        return Err(AppError::NotFound(format!("{} is not a file", body.path)));
    }

    let ext = resolved
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let runner_config = state
        .config
        .find_runner_for_extension(ext)
        .map(|(_, r)| r.clone())
        .ok_or_else(|| {
            AppError::BadRequest(format!("no runner configured for .{ext} files"))
        })?;

    let handle = spawn_runner(&runner_config, &resolved, &state.root)
        .map_err(|e| AppError::Internal(e))?;

    let id = uuid::Uuid::new_v4().to_string();
    state.run_registry.lock().await.insert(id.clone(), handle);

    Ok((StatusCode::CREATED, Json(RunResponse { id })))
}

/// DELETE /api/run/:id — kill a running process.
pub async fn stop_run(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Result<StatusCode, AppError> {
    let mut registry = state.run_registry.lock().await;

    if registry.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("run {id} not found")))
    }
}

/// GET /api/run/:id — check status of a run.
pub async fn get_run_status(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<RunStatusResponse>, AppError> {
    let registry = state.run_registry.lock().await;

    if let Some(handle) = registry.get(&id) {
        let ec = handle.exit_code.lock().await;
        if let Some(code) = *ec {
            Ok(Json(RunStatusResponse {
                id,
                running: false,
                exit_code: Some(code),
            }))
        } else {
            Ok(Json(RunStatusResponse {
                id,
                running: true,
                exit_code: None,
            }))
        }
    } else {
        Err(AppError::NotFound(format!("run {id} not found")))
    }
}
