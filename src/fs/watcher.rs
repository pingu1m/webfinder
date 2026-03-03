use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEvent, DebouncedEventKind};
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::config::FilesystemConfig;
use crate::fs::walk::{self, FileNode};
use crate::state::FsEvent;

/// Spawn a filesystem watcher that:
/// 1. Debounces notify events at 50ms
/// 2. Patches the cached tree in-place (sequentially, preserving order)
/// 3. Broadcasts events to WebSocket subscribers
pub fn spawn_watcher(
    root: PathBuf,
    config: FilesystemConfig,
    tree_cache: Arc<RwLock<Vec<FileNode>>>,
    watch_tx: broadcast::Sender<FsEvent>,
) -> anyhow::Result<()> {
    let root_canon = dunce::canonicalize(&root)?;
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(50), notify_tx)?;
    debouncer
        .watcher()
        .watch(&root, notify::RecursiveMode::Recursive)?;

    // Channel to send batches from the blocking thread to a single async task.
    // This guarantees sequential processing — no out-of-order mutations.
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Vec<DebouncedEvent>>();

    // Single async task that processes all batches in order
    tokio::spawn(async move {
        while let Some(events) = event_rx.recv().await {
            let mut tree = tree_cache.write().await;

            for event in events {
                let path = &event.path;

                // Check if any ancestor (or the file itself) is in exclude list
                if is_excluded(path, &root_canon, &config.exclude_patterns) {
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
                            let already_in_tree = walk::node_exists(&tree, &relative);
                            walk::insert_node(&mut tree, &relative, is_dir);
                            if already_in_tree { "modify" } else { "create" }
                        } else {
                            walk::remove_node(&mut tree, &relative);
                            "remove"
                        }
                    }
                    _ => continue,
                };

                let _ = watch_tx.send(FsEvent {
                    kind: kind.to_string(),
                    path: relative,
                });
            }
        }
    });

    // Blocking thread that reads from the debouncer and forwards to the async task
    tokio::task::spawn_blocking(move || {
        let _debouncer = debouncer;

        for events in notify_rx {
            let events = match events {
                Ok(evts) => evts,
                Err(err) => {
                    tracing::warn!("watcher error: {err}");
                    continue;
                }
            };

            if event_tx.send(events).is_err() {
                break;
            }
        }
    });

    Ok(())
}

/// Check if any component of `path` relative to `root` matches an exclude pattern.
fn is_excluded(path: &std::path::Path, root: &std::path::Path, patterns: &[String]) -> bool {
    if let Ok(relative) = path.strip_prefix(root) {
        for component in relative.components() {
            let name = component.as_os_str().to_str().unwrap_or("");
            if patterns.iter().any(|p| p == name) {
                return true;
            }
        }
    }
    false
}
