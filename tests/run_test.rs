mod common;

use common::TestServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_start_run() {
    let server = TestServer::start().await;
    server.create_file("hello.sh", "#!/bin/bash\necho hello");

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = server.dir_path().join("hello.sh");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let res = server
        .client
        .post(server.url("/api/run"))
        .json(&serde_json::json!({ "path": "hello.sh" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["id"].as_str().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_run_no_runner() {
    let server = TestServer::start().await;
    server.create_file("test.xyz", "no runner for this");

    let res = server
        .client
        .post(server.url("/api/run"))
        .json(&serde_json::json!({ "path": "test.xyz" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_run_file_not_found() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/run"))
        .json(&serde_json::json!({ "path": "nonexistent.sh" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stop_run() {
    let server = TestServer::start().await;
    server.create_file("sleep.sh", "#!/bin/bash\nsleep 60");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = server.dir_path().join("sleep.sh");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let res = server
        .client
        .post(server.url("/api/run"))
        .json(&serde_json::json!({ "path": "sleep.sh" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: serde_json::Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    // Stop it
    let res = server
        .client
        .delete(server.url(&format!("/api/run/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_stop_nonexistent_run() {
    let server = TestServer::start().await;

    let res = server
        .client
        .delete(server.url("/api/run/nonexistent-id"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_run_python_script() {
    let server = TestServer::start().await;
    server.create_file("hello.py", "print('hello from python')");

    let res = server
        .client
        .post(server.url("/api/run"))
        .json(&serde_json::json!({ "path": "hello.py" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let body: serde_json::Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    // Wait a bit for execution
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Check status
    let res = server
        .client
        .get(server.url(&format!("/api/run/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let status: serde_json::Value = res.json().await.unwrap();
    assert_eq!(status["running"].as_bool(), Some(false));
    assert_eq!(status["exit_code"].as_i64(), Some(0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_run_status_not_found() {
    let server = TestServer::start().await;

    let res = server
        .client
        .get(server.url("/api/run/nonexistent-id"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_run_status_while_running() {
    let server = TestServer::start().await;
    server.create_file("slow.sh", "#!/bin/bash\nsleep 30");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = server.dir_path().join("slow.sh");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let res = server
        .client
        .post(server.url("/api/run"))
        .json(&serde_json::json!({ "path": "slow.sh" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: serde_json::Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    // Check status immediately — should be running
    let res = server
        .client
        .get(server.url(&format!("/api/run/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let status: serde_json::Value = res.json().await.unwrap();
    assert_eq!(status["running"].as_bool(), Some(true));
    assert!(status["exit_code"].is_null());

    // Clean up: stop the run
    let _ = server
        .client
        .delete(server.url(&format!("/api/run/{id}")))
        .send()
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_run_nonzero_exit_code() {
    let server = TestServer::start().await;
    server.create_file("fail.sh", "#!/bin/bash\nexit 42");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = server.dir_path().join("fail.sh");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let res = server
        .client
        .post(server.url("/api/run"))
        .json(&serde_json::json!({ "path": "fail.sh" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: serde_json::Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let res = server
        .client
        .get(server.url(&format!("/api/run/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let status: serde_json::Value = res.json().await.unwrap();
    assert_eq!(status["running"].as_bool(), Some(false));
    assert_eq!(status["exit_code"].as_i64(), Some(42));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_run_path_traversal_blocked() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/run"))
        .json(&serde_json::json!({ "path": "../../etc/passwd" }))
        .send()
        .await
        .unwrap();
    assert!(
        res.status() == 403 || res.status() == 404,
        "expected 403 or 404, got {}",
        res.status()
    );
}
