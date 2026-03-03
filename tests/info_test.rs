mod common;

use common::TestServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_info() {
    let server = TestServer::start().await;

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["root"].as_str().is_some());
    assert!(body["name"].as_str().is_some());
    assert_eq!(body["version"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert!(body["config"]["runners"].as_array().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_info_has_config_summary() {
    let server = TestServer::start().await;

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = res.json().await.unwrap();
    let config = &body["config"];
    assert!(config["editor_theme"].as_str().is_some());
    assert!(config["show_hidden"].is_boolean());
}
