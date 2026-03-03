use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::fs::walk::{FileNode, NodeType};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_mode() -> String {
    "filename".into()
}

#[derive(Serialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// GET /api/search?q=...&mode=filename|content
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    if query.q.is_empty() {
        return Ok(Json(Vec::new()));
    }

    match query.mode.as_str() {
        "content" => search_content(&state, &query.q).await,
        _ => search_filename(&state, &query.q).await,
    }
}

async fn search_filename(
    state: &AppState,
    needle: &str,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let tree = state.tree_cache.read().await;
    let lower = needle.to_lowercase();
    let mut results = Vec::new();
    collect_filename_matches(&tree, &lower, &mut results);
    Ok(Json(results))
}

const MAX_RESULTS: usize = 500;

fn collect_filename_matches(nodes: &[FileNode], needle: &str, results: &mut Vec<SearchResult>) {
    for node in nodes {
        if results.len() >= MAX_RESULTS {
            return;
        }
        if node.name.to_lowercase().contains(needle) {
            results.push(SearchResult {
                path: node.path.clone(),
                name: node.name.clone(),
                node_type: match node.node_type {
                    NodeType::File => "file".into(),
                    NodeType::Dir => "dir".into(),
                },
                line: None,
                snippet: None,
            });
        }
        if let Some(ref children) = node.children {
            collect_filename_matches(children, needle, results);
        }
    }
}

async fn search_content(
    state: &AppState,
    needle: &str,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let root = state.root.clone();
    let config = state.config.read().await.filesystem.clone();
    let needle = needle.to_string();

    // Run content search on blocking thread pool
    let results = tokio::task::spawn_blocking(move || content_grep(&root, &config, &needle))
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("search task failed: {e}")))?;

    Ok(Json(results))
}

fn content_grep(
    root: &std::path::Path,
    config: &crate::config::FilesystemConfig,
    needle: &str,
) -> Vec<SearchResult> {
    use ignore::WalkBuilder;

    let lower = needle.to_lowercase();
    let mut results = Vec::new();

    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(!config.show_hidden)
        .git_ignore(true)
        .max_depth(None);

    for entry in builder.build().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Skip if any ancestor matches an exclude pattern
        if let Ok(relative) = path.strip_prefix(root) {
            let excluded = relative.components().any(|c| {
                let name = c.as_os_str().to_str().unwrap_or("");
                config.exclude_patterns.iter().any(|p| p == name)
            });
            if excluded {
                continue;
            }
        }

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Quick size check
        if let Ok(meta) = path.metadata() {
            if meta.len() > config.max_file_size_bytes {
                continue;
            }
        }

        // Read and search
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if content_inspector::inspect(&data[..data.len().min(8192)]).is_binary() {
            continue;
        }

        let text = match std::str::from_utf8(&data) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let relative = match path.strip_prefix(root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };

        for (line_num, line) in text.lines().enumerate() {
            if line.to_lowercase().contains(&lower) {
                results.push(SearchResult {
                    path: relative.clone(),
                    name: name.to_string(),
                    node_type: "file".into(),
                    line: Some((line_num + 1) as u64),
                    snippet: Some(line.chars().take(200).collect()),
                });
                if results.len() >= 500 {
                    return results;
                }
            }
        }
    }

    results
}
