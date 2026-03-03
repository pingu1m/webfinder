use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::error::AppError;
use crate::state::AppState;

const VALID_WORD_WRAPS: &[&str] = &["on", "off", "wordWrapColumn", "bounded"];
const VALID_THEMES: &[&str] = &["light", "vs-dark", "hc-black"];

#[derive(Deserialize)]
pub struct UpdateSettingsBody {
    pub auto_save: Option<bool>,
    pub font_size: Option<u32>,
    pub tab_size: Option<u32>,
    pub word_wrap: Option<String>,
    pub theme: Option<String>,
}

/// PUT /api/settings — update editor settings at runtime.
pub async fn put_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingsBody>,
) -> Result<StatusCode, AppError> {
    if let Some(v) = body.font_size {
        if !(8..=72).contains(&v) {
            return Err(AppError::BadRequest(format!(
                "font_size must be 8–72, got {v}"
            )));
        }
    }
    if let Some(v) = body.tab_size {
        if !(1..=16).contains(&v) {
            return Err(AppError::BadRequest(format!(
                "tab_size must be 1–16, got {v}"
            )));
        }
    }
    if let Some(ref v) = body.word_wrap {
        if !VALID_WORD_WRAPS.contains(&v.as_str()) {
            return Err(AppError::BadRequest(format!(
                "word_wrap must be one of {:?}, got \"{v}\"",
                VALID_WORD_WRAPS
            )));
        }
    }
    if let Some(ref v) = body.theme {
        if !VALID_THEMES.contains(&v.as_str()) {
            return Err(AppError::BadRequest(format!(
                "theme must be one of {:?}, got \"{v}\"",
                VALID_THEMES
            )));
        }
    }

    let mut config = state.config.write().await;

    if let Some(v) = body.auto_save {
        config.editor.auto_save = v;
    }
    if let Some(v) = body.font_size {
        config.editor.font_size = v;
    }
    if let Some(v) = body.tab_size {
        config.editor.tab_size = v;
    }
    if let Some(v) = body.word_wrap {
        config.editor.word_wrap = v;
    }
    if let Some(v) = body.theme {
        config.editor.theme = v;
    }

    Ok(StatusCode::NO_CONTENT)
}
