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

    let runners: Vec<String> = state.config.runners.keys().cloned().collect();

    Json(InfoResponse {
        root: state.root.to_string_lossy().into_owned(),
        name,
        version: env!("CARGO_PKG_VERSION").to_string(),
        config: ConfigSummary {
            runners,
            editor_theme: state.config.editor.theme.clone(),
            show_hidden: state.config.filesystem.show_hidden,
        },
    })
}
