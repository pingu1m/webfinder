mod common;

use common::TestServer;
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_watch_websocket_connect() {
    let server = TestServer::start().await;

    let url = format!("ws://{}/api/watch", server.addr);
    let (ws, _) = connect_async(&url).await.expect("ws connect failed");
    let (_write, _read) = ws.split();
    // Connection established successfully
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_watch_receives_broadcast_event() {
    let server = TestServer::start().await;

    let url = format!("ws://{}/api/watch", server.addr);
    let (ws, _) = connect_async(&url).await.expect("ws connect failed");
    let (_write, mut read) = ws.split();

    // Small delay to ensure the WS subscription is active
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Manually broadcast an event (simulating what the watcher would do)
    let _ = server.watch_tx.send(webfinder::state::FsEvent {
        kind: "create".to_string(),
        path: "test-file.txt".to_string(),
    });

    let event = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while let Some(msg) = read.next().await {
            if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = msg {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                if parsed["path"].as_str() == Some("test-file.txt") {
                    return Some(parsed);
                }
            }
        }
        None
    })
    .await;

    assert!(event.is_ok(), "should receive event within timeout");
    let event = event.unwrap();
    assert!(event.is_some(), "should match the test-file.txt event");
    let event = event.unwrap();
    assert_eq!(event["kind"].as_str(), Some("create"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_watch_receives_multiple_events() {
    let server = TestServer::start().await;

    let url = format!("ws://{}/api/watch", server.addr);
    let (ws, _) = connect_async(&url).await.expect("ws connect failed");
    let (_write, mut read) = ws.split();

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Send multiple events
    let _ = server.watch_tx.send(webfinder::state::FsEvent {
        kind: "create".to_string(),
        path: "a.txt".to_string(),
    });
    let _ = server.watch_tx.send(webfinder::state::FsEvent {
        kind: "modify".to_string(),
        path: "b.txt".to_string(),
    });
    let _ = server.watch_tx.send(webfinder::state::FsEvent {
        kind: "remove".to_string(),
        path: "c.txt".to_string(),
    });

    let mut received = Vec::new();
    let _ = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while let Some(msg) = read.next().await {
            if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = msg {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                received.push(parsed);
                if received.len() >= 3 {
                    return;
                }
            }
        }
    })
    .await;

    assert_eq!(received.len(), 3, "should receive all 3 events");
    assert_eq!(received[0]["kind"].as_str(), Some("create"));
    assert_eq!(received[1]["kind"].as_str(), Some("modify"));
    assert_eq!(received[2]["kind"].as_str(), Some("remove"));
}
