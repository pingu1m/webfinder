use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{broadcast, oneshot, Mutex};

use crate::config::RunnerConfig;
use crate::runner::{OutputLine, RunHandle};

/// Spawn a child process for the given file using the matched runner config.
/// Returns a RunHandle that includes a kill channel for graceful termination.
pub fn spawn_runner(
    runner: &RunnerConfig,
    file_path: &Path,
    working_dir: &Path,
) -> anyhow::Result<RunHandle> {
    let file_str = file_path.to_string_lossy().to_string();

    let args: Vec<String> = runner
        .args
        .iter()
        .map(|a| a.replace("{file}", &file_str))
        .collect();

    let mut cmd = Command::new(&runner.command);
    cmd.args(&args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd.spawn()?;

    let (output_tx, _) = broadcast::channel(1024);
    let exit_code: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));
    let (kill_tx, kill_rx) = oneshot::channel::<()>();

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let tx = output_tx.clone();
    let ec = exit_code.clone();

    tokio::spawn(async move {
        let stdout_task = tokio::spawn({
            let tx = tx.clone();
            async move {
                if let Some(stdout) = stdout {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let _ = tx.send(OutputLine {
                            stream: "stdout".into(),
                            data: line,
                        });
                    }
                }
            }
        });

        let stderr_task = tokio::spawn({
            let tx = tx.clone();
            async move {
                if let Some(stderr) = stderr {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let _ = tx.send(OutputLine {
                            stream: "stderr".into(),
                            data: line,
                        });
                    }
                }
            }
        });

        // Wait for the process to exit naturally, or for a kill request.
        let code = tokio::select! {
            result = child.wait() => {
                result.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1)
            }
            _ = kill_rx => {
                let _ = child.kill().await;
                child.wait().await.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1)
            }
        };

        // Drain remaining output before sending exit event
        let _ = stdout_task.await;
        let _ = stderr_task.await;

        *ec.lock().await = Some(code);

        let _ = tx.send(OutputLine {
            stream: "exit".into(),
            data: code.to_string(),
        });
    });

    Ok(RunHandle {
        output_tx,
        exit_code,
        kill_tx: Some(kill_tx),
    })
}
