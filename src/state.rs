use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{broadcast, RwLock, Mutex};

use crate::config::Config;
use crate::fs::walk::FileNode;
use crate::runner::RunHandle;

/// Filesystem event sent over the watch WebSocket.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FsEvent {
    pub kind: String,
    pub path: String,
}

/// Shared application state, wrapped in `Arc` for axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub config: Arc<Config>,
    pub tree_cache: Arc<RwLock<Vec<FileNode>>>,
    pub watch_tx: broadcast::Sender<FsEvent>,
    pub run_registry: Arc<Mutex<HashMap<String, RunHandle>>>,
}

impl AppState {
    pub fn new(root: PathBuf, config: Config) -> Self {
        let (watch_tx, _) = broadcast::channel(512);
        Self {
            root,
            config: Arc::new(config),
            tree_cache: Arc::new(RwLock::new(Vec::new())),
            watch_tx,
            run_registry: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
