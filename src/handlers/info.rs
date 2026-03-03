use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct InfoResponse {
    pub root: String,
    pub name: String,
    pub version: String,
    pub config: ConfigSummary,
}

#[derive(Serialize)]
pub struct ConfigSummary {
    pub runners: Vec<String>,
    pub editor_theme: String,
    pub auto_save: bool,
    pub font_size: u32,
    pub tab_size: u32,
    pub word_wrap: String,
    pub show_hidden: bool,
}

/// GET /api/info — server and project information.
pub async fn get_info(State(state): State<AppState>) -> Json<InfoResponse> {
    let name = state
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let config = state.config.read().await;
    let runners: Vec<String> = config.runners.keys().cloned().collect();

    Json(InfoResponse {
        root: state.root.to_string_lossy().into_owned(),
        name,
        version: env!("CARGO_PKG_VERSION").to_string(),
        config: ConfigSummary {
            runners,
            editor_theme: config.editor.theme.clone(),
            auto_save: config.editor.auto_save,
            font_size: config.editor.font_size,
            tab_size: config.editor.tab_size,
            word_wrap: config.editor.word_wrap.clone(),
            show_hidden: config.filesystem.show_hidden,
        },
    })
}
