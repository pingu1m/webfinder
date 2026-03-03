use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub user: UserConfig,
    pub git: GitConfig,
    #[serde(default)]
    pub runners: HashMap<String, RunnerConfig>,
    pub editor: EditorConfig,
    pub filesystem: FilesystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub open_browser: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UserConfig {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GitConfig {
    pub repos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub font_size: u32,
    pub tab_size: u32,
    pub word_wrap: String,
    pub theme: String,
    pub auto_save: bool,
    pub save_debounce_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FilesystemConfig {
    pub show_hidden: bool,
    pub max_file_size_bytes: u64,
    pub exclude_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut runners = HashMap::new();
        runners.insert(
            "python".into(),
            RunnerConfig {
                command: "python3".into(),
                args: vec!["{file}".into()],
                extensions: vec!["py".into()],
            },
        );
        runners.insert(
            "node".into(),
            RunnerConfig {
                command: "node".into(),
                args: vec!["{file}".into()],
                extensions: vec!["js".into(), "mjs".into()],
            },
        );
        runners.insert(
            "typescript".into(),
            RunnerConfig {
                command: "npx".into(),
                args: vec!["tsx".into(), "{file}".into()],
                extensions: vec!["ts".into(), "tsx".into()],
            },
        );
        runners.insert(
            "shell".into(),
            RunnerConfig {
                command: "bash".into(),
                args: vec!["{file}".into()],
                extensions: vec!["sh".into(), "bash".into()],
            },
        );
        Self {
            server: ServerConfig::default(),
            user: UserConfig::default(),
            git: GitConfig::default(),
            runners,
            editor: EditorConfig::default(),
            filesystem: FilesystemConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 0,
            open_browser: true,
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_size: 14,
            tab_size: 2,
            word_wrap: "on".into(),
            theme: "light".into(),
            auto_save: false,
            save_debounce_ms: 150,
        }
    }
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            max_file_size_bytes: 10 * 1024 * 1024,
            exclude_patterns: vec![
                "node_modules".into(),
                ".git".into(),
                "target".into(),
                "__pycache__".into(),
            ],
        }
    }
}

impl Config {
    pub fn find_runner_for_extension(&self, ext: &str) -> Option<(&str, &RunnerConfig)> {
        self.runners
            .iter()
            .find(|(_, r)| r.extensions.iter().any(|e| e == ext))
            .map(|(name, r)| (name.as_str(), r))
    }
}

/// Search for config in standard locations, in priority order:
/// 1. Explicit --config path (highest priority)
/// 2. ./webfinder.toml (cwd)
/// 3. $XDG_CONFIG_HOME/webfinder/config.toml
/// 4. ~/.config/webfinder/config.toml
pub fn load_config(explicit_path: Option<&Path>) -> Result<Config> {
    let candidates = config_candidates(explicit_path);

    for path in &candidates {
        if path.is_file() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("reading config from {}", path.display()))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("parsing config from {}", path.display()))?;
            tracing::info!(path = %path.display(), "loaded config");
            return Ok(config);
        }
    }

    tracing::info!("no config file found, using defaults");
    Ok(Config::default())
}

fn config_candidates(explicit: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Explicit --config flag takes highest priority
    if let Some(p) = explicit {
        paths.push(p.to_path_buf());
    }

    // CWD
    paths.push(PathBuf::from("webfinder.toml"));

    // XDG / platform
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("webfinder").join("config.toml"));
    }

    // Fallback ~/.config
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".config").join("webfinder").join("config.toml"));
    }

    paths
}
