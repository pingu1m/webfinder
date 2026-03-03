pub mod process;

use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, Mutex};

/// Handle to a running process, stored in the run registry.
pub struct RunHandle {
    pub output_tx: broadcast::Sender<OutputLine>,
    pub exit_code: Arc<Mutex<Option<i32>>>,
    /// Send on this channel to request the child process be killed.
    pub kill_tx: Option<oneshot::Sender<()>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OutputLine {
    pub stream: String, // "stdout" | "stderr" | "exit"
    pub data: String,
}
