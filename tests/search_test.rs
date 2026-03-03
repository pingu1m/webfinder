mod common;

use common::TestServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_search_filename() {
    let server = TestServer::start_with_setup(|dir| {
        std::fs::write(dir.join("readme.md"), "# Hello").unwrap();
        std::fs::write(dir.join("main.rs"), "fn main() {}").unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/lib.rs"), "pub fn lib() {}").unwrap();
    })
    .await;

    let res = server
        .client
        .get(server.url("/api/search?q=main&mode=filename"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(!body.is_empty(), "expected results for 'main'");
    assert!(body.iter().any(|r| r["name"].as_str() == Some("main.rs")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_search_content() {
    let server = TestServer::start_with_setup(|dir| {
        std::fs::write(dir.join("hello.txt"), "hello world").unwrap();
        std::fs::write(dir.join("other.txt"), "goodbye world").unwrap();
    })
    .await;

    let res = server
        .client
        .get(server.url("/api/search?q=hello&mode=content"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(!body.is_empty());
    assert!(body.iter().any(|r| r["path"].as_str() == Some("hello.txt")));
    assert!(body.iter().all(|r| r["path"].as_str() != Some("other.txt")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_search_no_results() {
    let server = TestServer::start_with_setup(|dir| {
        std::fs::write(dir.join("test.txt"), "nothing here").unwrap();
    })
    .await;

    let res = server
        .client
        .get(server.url("/api/search?q=zzzznoexist&mode=filename"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_search_empty_query() {
    let server = TestServer::start().await;

    let res = server
        .client
        .get(server.url("/api/search?q=&mode=filename"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_search_content_with_line_number() {
    let server = TestServer::start_with_setup(|dir| {
        std::fs::write(dir.join("multi.txt"), "line one\nline two\nfind me\nline four").unwrap();
    })
    .await;

    let res = server
        .client
        .get(server.url("/api/search?q=find+me&mode=content"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(!body.is_empty());
    let result = &body[0];
    assert_eq!(result["line"].as_u64(), Some(3));
}
