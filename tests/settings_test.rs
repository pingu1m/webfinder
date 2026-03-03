mod common;

use common::TestServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_info_exposes_auto_save_default_false() {
    let server = TestServer::start().await;

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["auto_save"].as_bool(), Some(false));
    assert_eq!(body["config"]["font_size"].as_u64(), Some(14));
    assert_eq!(body["config"]["tab_size"].as_u64(), Some(2));
    assert_eq!(body["config"]["word_wrap"].as_str(), Some("on"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_toggle_auto_save() {
    let server = TestServer::start().await;

    // Enable auto_save
    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "auto_save": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Verify via info
    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["auto_save"].as_bool(), Some(true));

    // Disable auto_save
    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "auto_save": false }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["auto_save"].as_bool(), Some(false));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_update_font_size() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "font_size": 18 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["font_size"].as_u64(), Some(18));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_update_tab_size() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "tab_size": 4 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["tab_size"].as_u64(), Some(4));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_update_word_wrap() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "word_wrap": "off" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["word_wrap"].as_str(), Some("off"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_update_theme() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "theme": "vs-dark" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["editor_theme"].as_str(), Some("vs-dark"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_partial_update_preserves_others() {
    let server = TestServer::start().await;

    // Update only font_size
    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "font_size": 20 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    // font_size changed
    assert_eq!(body["config"]["font_size"].as_u64(), Some(20));
    // other fields unchanged
    assert_eq!(body["config"]["auto_save"].as_bool(), Some(false));
    assert_eq!(body["config"]["tab_size"].as_u64(), Some(2));
    assert_eq!(body["config"]["word_wrap"].as_str(), Some("on"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_rejects_out_of_range_font_size() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "font_size": 5 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["font_size"].as_u64(), Some(14));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_rejects_invalid_word_wrap() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "word_wrap": "banana" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["word_wrap"].as_str(), Some("on"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_rejects_invalid_theme() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "theme": "solarized-rainbow" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_rejects_out_of_range_tab_size() {
    let server = TestServer::start().await;

    // Below minimum
    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "tab_size": 0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    // Above maximum
    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "tab_size": 99 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    // Verify value unchanged
    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["tab_size"].as_u64(), Some(2));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_empty_body_no_change() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // All defaults preserved
    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["auto_save"].as_bool(), Some(false));
    assert_eq!(body["config"]["font_size"].as_u64(), Some(14));
    assert_eq!(body["config"]["tab_size"].as_u64(), Some(2));
    assert_eq!(body["config"]["word_wrap"].as_str(), Some("on"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_rejects_font_size_above_max() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "font_size": 100 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_boundary_values_accepted() {
    let server = TestServer::start().await;

    // Min boundaries
    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "font_size": 8, "tab_size": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["font_size"].as_u64(), Some(8));
    assert_eq!(body["config"]["tab_size"].as_u64(), Some(1));

    // Max boundaries
    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({ "font_size": 72, "tab_size": 16 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["font_size"].as_u64(), Some(72));
    assert_eq!(body["config"]["tab_size"].as_u64(), Some(16));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_settings_multiple_fields_at_once() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/settings"))
        .json(&serde_json::json!({
            "auto_save": true,
            "font_size": 16,
            "tab_size": 8,
            "word_wrap": "off",
            "theme": "hc-black"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = server
        .client
        .get(server.url("/api/info"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["auto_save"].as_bool(), Some(true));
    assert_eq!(body["config"]["font_size"].as_u64(), Some(16));
    assert_eq!(body["config"]["tab_size"].as_u64(), Some(8));
    assert_eq!(body["config"]["word_wrap"].as_str(), Some("off"));
    assert_eq!(body["config"]["editor_theme"].as_str(), Some("hc-black"));
}
