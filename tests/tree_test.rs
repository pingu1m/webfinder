mod common;

use common::TestServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_tree_empty() {
    let server = TestServer::start().await;
    let res = server
        .client
        .get(server.url("/api/tree"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_tree_with_files() {
    let server = TestServer::start_with_setup(|dir| {
        std::fs::write(dir.join("hello.txt"), "hello").unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/main.rs"), "fn main() {}").unwrap();
    })
    .await;

    let res = server
        .client
        .get(server.url("/api/tree"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(body.len() >= 2, "expected at least 2 entries, got: {body:?}");

    // Should contain src dir and hello.txt
    let names: Vec<&str> = body.iter().filter_map(|n| n["name"].as_str()).collect();
    assert!(names.contains(&"src"), "expected src dir in {names:?}");
    assert!(names.contains(&"hello.txt"), "expected hello.txt in {names:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_tree_excludes_gitignored() {
    let server = TestServer::start_with_setup(|dir| {
        // Initialize a git repo so the ignore crate picks up .gitignore
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
        std::fs::write(dir.join(".gitignore"), "ignored/\n").unwrap();
        std::fs::create_dir_all(dir.join("ignored")).unwrap();
        std::fs::write(dir.join("ignored/secret.txt"), "secret").unwrap();
        std::fs::write(dir.join("visible.txt"), "visible").unwrap();
    })
    .await;

    let res = server
        .client
        .get(server.url("/api/tree"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    let names: Vec<&str> = body.iter().filter_map(|n| n["name"].as_str()).collect();
    assert!(!names.contains(&"ignored"), "should exclude gitignored dir");
    assert!(names.contains(&"visible.txt"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_tree_excludes_node_modules() {
    let server = TestServer::start_with_setup(|dir| {
        std::fs::create_dir_all(dir.join("node_modules/foo")).unwrap();
        std::fs::write(dir.join("node_modules/foo/index.js"), "").unwrap();
        std::fs::write(dir.join("app.js"), "").unwrap();
    })
    .await;

    let res = server
        .client
        .get(server.url("/api/tree"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    let names: Vec<&str> = body.iter().filter_map(|n| n["name"].as_str()).collect();
    assert!(!names.contains(&"node_modules"));
    assert!(names.contains(&"app.js"));
}
