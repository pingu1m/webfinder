use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Resolve a user-supplied relative path against the jail root.
/// Returns the canonical absolute path, or an error if it escapes the jail.
pub fn resolve_path(root: &Path, relative: &str) -> Result<PathBuf, AppError> {
    if relative.is_empty() {
        return Ok(root.to_path_buf());
    }

    let cleaned = relative.trim_start_matches('/').trim_start_matches('\\');

    let candidate = root.join(cleaned);

    let canonical = if candidate.exists() {
        dunce::canonicalize(&candidate)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("canonicalize failed: {e}")))?
    } else {
        // For new files that don't exist yet, canonicalize the parent and
        // append the filename.
        let parent = candidate
            .parent()
            .ok_or_else(|| AppError::BadRequest("invalid path".into()))?;
        let parent_canon = if parent.exists() {
            dunce::canonicalize(parent).map_err(|e| {
                AppError::Internal(anyhow::anyhow!("canonicalize parent failed: {e}"))
            })?
        } else {
            // Parent doesn't exist either — still check it's under root
            let mut p = root.to_path_buf();
            for component in Path::new(cleaned)
                .parent()
                .unwrap_or(Path::new(""))
                .components()
            {
                match component {
                    std::path::Component::Normal(c) => p.push(c),
                    std::path::Component::ParentDir => {
                        return Err(AppError::Forbidden("path traversal blocked".into()));
                    }
                    _ => {}
                }
            }
            p
        };
        let filename = candidate
            .file_name()
            .ok_or_else(|| AppError::BadRequest("no filename".into()))?;
        parent_canon.join(filename)
    };

    let root_canon = dunce::canonicalize(root)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("canonicalize root failed: {e}")))?;

    if !canonical.starts_with(&root_canon) {
        return Err(AppError::Forbidden("path traversal blocked".into()));
    }

    Ok(canonical)
}

/// Check if file content appears to be binary.
pub fn is_binary(data: &[u8]) -> bool {
    let check_len = data.len().min(8192);
    content_inspector::inspect(&data[..check_len]).is_binary()
}

/// Detect language from file extension for API responses.
pub fn detect_language(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "rs" => "rust",
        "ts" => "typescript",
        "tsx" => "typescriptreact",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "javascriptreact",
        "py" => "python",
        "rb" => "ruby",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "cs" => "csharp",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "xml" | "svg" => "xml",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "less" => "less",
        "md" | "markdown" => "markdown",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "shell",
        "dockerfile" => "dockerfile",
        "makefile" => "makefile",
        _ => "plaintext",
    }
}
