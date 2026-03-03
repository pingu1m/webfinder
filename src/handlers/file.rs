use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::error::AppError;
use crate::fs::guard;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct FileQuery {
    pub path: String,
}

#[derive(Serialize)]
pub struct FileResponse {
    pub path: String,
    pub content: Option<String>,
    pub language: String,
    pub size: u64,
    pub modified: String,
    pub binary: bool,
}

#[derive(Deserialize)]
pub struct CreateFileBody {
    pub path: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Deserialize)]
pub struct RenameBody {
    pub from: String,
    pub to: String,
}

/// GET /api/file?path=... — read file with ETag support.
pub async fn get_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<FileQuery>,
) -> Result<Response, AppError> {
    let resolved = guard::resolve_path(&state.root, &query.path)?;

    if !resolved.is_file() {
        return Err(AppError::NotFound(format!("{} is not a file", query.path)));
    }

    let metadata = tokio::fs::metadata(&resolved).await?;
    let size = metadata.len();

    let max_file_size = state.config.read().await.filesystem.max_file_size_bytes;
    if size > max_file_size {
        return Err(AppError::PayloadTooLarge(format!(
            "file is {} bytes, max is {}",
            size, max_file_size
        )));
    }

    let modified = metadata
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let modified_str = humanize_time(modified);

    // Compute ETag from mtime + size
    let etag = compute_etag(modified, size);

    // Check If-None-Match
    if let Some(inm) = headers.get("if-none-match") {
        if let Ok(inm_str) = inm.to_str() {
            if inm_str.trim_matches('"') == etag {
                return Ok(StatusCode::NOT_MODIFIED.into_response());
            }
        }
    }

    let data = tokio::fs::read(&resolved).await?;
    let binary = guard::is_binary(&data);
    let language = guard::detect_language(&resolved).to_string();

    let content = if binary {
        None
    } else {
        Some(String::from_utf8_lossy(&data).into_owned())
    };

    let body = FileResponse {
        path: query.path,
        content,
        language,
        size,
        modified: modified_str,
        binary,
    };

    let mut response = Json(body).into_response();
    response
        .headers_mut()
        .insert("etag", format!("\"{etag}\"").parse().unwrap());
    Ok(response)
}

/// PUT /api/file?path=... — write raw bytes to disk. No JSON wrapping.
pub async fn put_file(
    State(state): State<AppState>,
    Query(query): Query<FileQuery>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let resolved = guard::resolve_path(&state.root, &query.path)?;

    if !resolved.is_file() {
        return Err(AppError::NotFound(format!("{} is not a file", query.path)));
    }

    tokio::fs::write(&resolved, &body).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/file — create new file.
pub async fn create_file(
    State(state): State<AppState>,
    Json(body): Json<CreateFileBody>,
) -> Result<StatusCode, AppError> {
    let resolved = guard::resolve_path(&state.root, &body.path)?;

    if resolved.exists() {
        return Err(AppError::Conflict(format!("{} already exists", body.path)));
    }

    // Ensure parent directory exists
    if let Some(parent) = resolved.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(&resolved, body.content.as_bytes()).await?;
    Ok(StatusCode::CREATED)
}

/// DELETE /api/file?path=... — delete file.
pub async fn delete_file(
    State(state): State<AppState>,
    Query(query): Query<FileQuery>,
) -> Result<StatusCode, AppError> {
    let resolved = guard::resolve_path(&state.root, &query.path)?;

    if !resolved.is_file() {
        return Err(AppError::NotFound(format!("{} is not a file", query.path)));
    }

    tokio::fs::remove_file(&resolved).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/file/rename — rename/move file.
pub async fn rename_file(
    State(state): State<AppState>,
    Json(body): Json<RenameBody>,
) -> Result<StatusCode, AppError> {
    let from = guard::resolve_path(&state.root, &body.from)?;
    let to = guard::resolve_path(&state.root, &body.to)?;

    if !from.is_file() {
        return Err(AppError::NotFound(format!("{} is not a file", body.from)));
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

/// POST /api/file/copy — copy file.
pub async fn copy_file(
    State(state): State<AppState>,
    Json(body): Json<RenameBody>,
) -> Result<StatusCode, AppError> {
    let from = guard::resolve_path(&state.root, &body.from)?;
    let to = guard::resolve_path(&state.root, &body.to)?;

    if !from.is_file() {
        return Err(AppError::NotFound(format!("{} is not a file", body.from)));
    }
    if to.exists() {
        return Err(AppError::Conflict(format!("{} already exists", body.to)));
    }

    if let Some(parent) = to.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::copy(&from, &to).await?;
    Ok(StatusCode::CREATED)
}

fn compute_etag(mtime: std::time::SystemTime, size: u64) -> String {
    let mut hasher = DefaultHasher::new();
    mtime.hash(&mut hasher);
    size.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn humanize_time(t: std::time::SystemTime) -> String {
    let duration = t
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // ISO 8601 approximation
    chrono_from_epoch(secs)
}

fn chrono_from_epoch(secs: u64) -> String {
    // Simple ISO 8601 without chrono dependency
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Approximate date from days since epoch (good enough for display)
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z"
    )
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1;
    for md in &month_days {
        if days < *md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}
