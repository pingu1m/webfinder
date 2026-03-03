use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path as AxumPath, State, WebSocketUpgrade};
use axum::response::Response;

use crate::state::AppState;

/// WS /api/watch — stream filesystem events.
pub async fn watch_ws(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_watch(socket, state))
}

async fn handle_watch(mut socket: WebSocket, state: AppState) {
    let mut rx = state.watch_tx.subscribe();

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        let json = serde_json::to_string(&ev).unwrap_or_default();
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("watch subscriber lagged by {n} events");
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

/// WS /api/run/:id/stream — stream run output.
pub async fn run_stream_ws(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_run_stream(socket, state, id))
}

async fn handle_run_stream(mut socket: WebSocket, state: AppState, id: String) {
    // Get a receiver for this run's output
    let rx = {
        let registry = state.run_registry.lock().await;
        match registry.get(&id) {
            Some(handle) => handle.output_tx.subscribe(),
            None => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({"error": "run not found"})
                            .to_string()
                            .into(),
                    ))
                    .await;
                return;
            }
        }
    };

    let mut rx = rx;

    loop {
        tokio::select! {
            line = rx.recv() => {
                match line {
                    Ok(output) => {
                        let json = serde_json::to_string(&output).unwrap_or_default();
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("run stream subscriber lagged by {n} lines");
                    }
                    Err(_) => {
                        // Channel closed — clean up registry entry
                        state.run_registry.lock().await.remove(&id);
                        break;
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
