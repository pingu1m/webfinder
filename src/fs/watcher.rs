use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use tokio::sync::{broadcast, RwLock};

use crate::config::FilesystemConfig;
use crate::fs::walk::{self, FileNode};
use crate::state::FsEvent;

/// Spawn a filesystem watcher that:
/// 1. Debounces notify events at 50ms
/// 2. Patches the cached tree in-place
/// 3. Broadcasts events to WebSocket subscribers
pub fn spawn_watcher(
    root: PathBuf,
    config: FilesystemConfig,
    tree_cache: Arc<RwLock<Vec<FileNode>>>,
    watch_tx: broadcast::Sender<FsEvent>,
) -> anyhow::Result<()> {
    let root_canon = dunce::canonicalize(&root)?;
    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(50), tx)?;
    debouncer
        .watcher()
        .watch(&root, notify::RecursiveMode::Recursive)?;

    tokio::task::spawn_blocking(move || {
        // Keep debouncer alive
        let _debouncer = debouncer;

        for events in rx {
            let events = match events {
                Ok(evts) => evts,
                Err(err) => {
                    tracing::warn!("watcher error: {err}");
                    continue;
                }
            };

            let tree_cache = tree_cache.clone();
            let root_canon = root_canon.clone();
            let watch_tx = watch_tx.clone();
            let config = config.clone();

            tokio::spawn(async move {
                let mut tree = tree_cache.write().await;

                for event in events {
                    let path = &event.path;

                    // Skip excluded patterns
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    if config.exclude_patterns.iter().any(|p| name == p) {
                        continue;
                    }

                    let relative = match path.strip_prefix(&root_canon) {
                        Ok(r) => r.to_string_lossy().replace('\\', "/"),
                        Err(_) => continue,
                    };

                    if relative.is_empty() {
                        continue;
                    }

                    let kind = match event.kind {
                        DebouncedEventKind::Any => {
                            if path.exists() {
                                let is_dir = path.is_dir();
                                walk::insert_node(&mut tree, &relative, is_dir);
                                if is_dir { "create" } else { "modify" }
                            } else {
                                walk::remove_node(&mut tree, &relative);
                                "remove"
                            }
                        }
                        DebouncedEventKind::AnyContinuous | _ => continue,
                    };

                    let _ = watch_tx.send(FsEvent {
                        kind: kind.to_string(),
                        path: relative,
                    });
                }
            });
        }
    });

    Ok(())
}
