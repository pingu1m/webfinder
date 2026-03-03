use std::time::Duration;

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
        .read()
        .await
        .find_runner_for_extension(ext)
        .map(|(_, r)| r.clone())
        .ok_or_else(|| {
            AppError::BadRequest(format!("no runner configured for .{ext} files"))
        })?;

    let handle = spawn_runner(&runner_config, &resolved, &state.root)
        .map_err(|e| AppError::Internal(e))?;

    let id = uuid::Uuid::new_v4().to_string();

    // Subscribe to the output channel for auto-cleanup after exit
    let mut cleanup_rx = handle.output_tx.subscribe();
    let cleanup_registry = state.run_registry.clone();
    let cleanup_id = id.clone();
    tokio::spawn(async move {
        while let Ok(line) = cleanup_rx.recv().await {
            if line.stream == "exit" {
                // Give clients 60s to read the final status before removing
                tokio::time::sleep(Duration::from_secs(60)).await;
                cleanup_registry.lock().await.remove(&cleanup_id);
                break;
            }
        }
    });

    state.run_registry.lock().await.insert(id.clone(), handle);

    Ok((StatusCode::CREATED, Json(RunResponse { id })))
}

/// DELETE /api/run/:id — kill a running process.
pub async fn stop_run(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Result<StatusCode, AppError> {
    let mut registry = state.run_registry.lock().await;

    if let Some(mut handle) = registry.remove(&id) {
        // Send kill signal; the spawned task will terminate the child process.
        if let Some(kill_tx) = handle.kill_tx.take() {
            let _ = kill_tx.send(());
        }
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
    // Clone the Arc so we can drop the registry lock before awaiting exit_code.
    // This prevents blocking all registry access while reading exit_code.
    let exit_code_handle = {
        let registry = state.run_registry.lock().await;
        match registry.get(&id) {
            Some(handle) => handle.exit_code.clone(),
            None => return Err(AppError::NotFound(format!("run {id} not found"))),
        }
    };

    let ec = exit_code_handle.lock().await;
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
}
