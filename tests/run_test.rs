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
