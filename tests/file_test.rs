mod common;

use common::TestServer;
use std::io::Write;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_read_file() {
    let server = TestServer::start().await;
    server.create_file("test.txt", "hello world");

    let res = server
        .client
        .get(server.url("/api/file?path=test.txt"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["content"].as_str().unwrap(), "hello world");
    assert_eq!(body["binary"].as_bool().unwrap(), false);
    assert_eq!(body["path"].as_str().unwrap(), "test.txt");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_read_file_etag_304() {
    let server = TestServer::start().await;
    server.create_file("cached.txt", "cached content");

    // First request — get ETag
    let res = server
        .client
        .get(server.url("/api/file?path=cached.txt"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let etag = res
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Second request with If-None-Match
    let res = server
        .client
        .get(server.url("/api/file?path=cached.txt"))
        .header("If-None-Match", &etag)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 304);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_create_file() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/file"))
        .json(&serde_json::json!({ "path": "new.txt", "content": "new content" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    // Verify file exists
    let content = std::fs::read_to_string(server.dir_path().join("new.txt")).unwrap();
    assert_eq!(content, "new content");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_create_file_conflict() {
    let server = TestServer::start().await;
    server.create_file("exists.txt", "already here");

    let res = server
        .client
        .post(server.url("/api/file"))
        .json(&serde_json::json!({ "path": "exists.txt", "content": "" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_update_file() {
    let server = TestServer::start().await;
    server.create_file("update.txt", "old content");

    let res = server
        .client
        .put(server.url("/api/file?path=update.txt"))
        .header("Content-Type", "text/plain")
        .body("new content")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let content = std::fs::read_to_string(server.dir_path().join("update.txt")).unwrap();
    assert_eq!(content, "new content");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_file() {
    let server = TestServer::start().await;
    server.create_file("delete_me.txt", "bye");

    let res = server
        .client
        .delete(server.url("/api/file?path=delete_me.txt"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    assert!(!server.dir_path().join("delete_me.txt").exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rename_file() {
    let server = TestServer::start().await;
    server.create_file("old_name.txt", "content");

    let res = server
        .client
        .post(server.url("/api/file/rename"))
        .json(&serde_json::json!({ "from": "old_name.txt", "to": "new_name.txt" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    assert!(!server.dir_path().join("old_name.txt").exists());
    let content = std::fs::read_to_string(server.dir_path().join("new_name.txt")).unwrap();
    assert_eq!(content, "content");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_copy_file() {
    let server = TestServer::start().await;
    server.create_file("original.txt", "original content");

    let res = server
        .client
        .post(server.url("/api/file/copy"))
        .json(&serde_json::json!({ "from": "original.txt", "to": "copy.txt" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    assert!(server.dir_path().join("original.txt").exists());
    let content = std::fs::read_to_string(server.dir_path().join("copy.txt")).unwrap();
    assert_eq!(content, "original content");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_read_file_not_found() {
    let server = TestServer::start().await;

    let res = server
        .client
        .get(server.url("/api/file?path=nonexistent.txt"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_path_traversal_blocked() {
    let server = TestServer::start().await;

    let res = server
        .client
        .get(server.url("/api/file?path=../../../etc/passwd"))
        .send()
        .await
        .unwrap();
    // Should be 403 Forbidden or 404
    assert!(
        res.status() == 403 || res.status() == 404,
        "expected 403 or 404, got {}",
        res.status()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_create_file_in_nested_dir() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/file"))
        .json(&serde_json::json!({ "path": "deep/nested/file.txt", "content": "nested" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let content = std::fs::read_to_string(server.dir_path().join("deep/nested/file.txt")).unwrap();
    assert_eq!(content, "nested");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_read_binary_file() {
    let server = TestServer::start().await;

    let path = server.dir_path().join("image.bin");
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x00, 0x00, 0x00, 0x00])
        .unwrap();

    let res = server
        .client
        .get(server.url("/api/file?path=image.bin"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["binary"].as_bool(), Some(true));
    assert!(body["content"].is_null());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_file_not_found() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/file?path=nonexistent.txt"))
        .header("Content-Type", "text/plain")
        .body("content")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_put_file_path_traversal() {
    let server = TestServer::start().await;

    let res = server
        .client
        .put(server.url("/api/file?path=../../etc/shadow"))
        .header("Content-Type", "text/plain")
        .body("hacked")
        .send()
        .await
        .unwrap();
    assert!(
        res.status() == 403 || res.status() == 404,
        "expected 403 or 404, got {}",
        res.status()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_file_path_traversal() {
    let server = TestServer::start().await;

    let res = server
        .client
        .delete(server.url("/api/file?path=../../etc/passwd"))
        .send()
        .await
        .unwrap();
    assert!(
        res.status() == 403 || res.status() == 404,
        "expected 403 or 404, got {}",
        res.status()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rename_file_destination_conflict() {
    let server = TestServer::start().await;
    server.create_file("source.txt", "content");
    server.create_file("exists.txt", "already here");

    let res = server
        .client
        .post(server.url("/api/file/rename"))
        .json(&serde_json::json!({ "from": "source.txt", "to": "exists.txt" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);

    // source should still exist
    assert!(server.dir_path().join("source.txt").exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rename_file_source_not_found() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/file/rename"))
        .json(&serde_json::json!({ "from": "nope.txt", "to": "new.txt" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_copy_file_destination_conflict() {
    let server = TestServer::start().await;
    server.create_file("orig.txt", "original");
    server.create_file("taken.txt", "taken");

    let res = server
        .client
        .post(server.url("/api/file/copy"))
        .json(&serde_json::json!({ "from": "orig.txt", "to": "taken.txt" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_copy_file_source_not_found() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/file/copy"))
        .json(&serde_json::json!({ "from": "missing.txt", "to": "dest.txt" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_create_file_path_traversal() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/file"))
        .json(&serde_json::json!({ "path": "../../etc/hacked", "content": "" }))
        .send()
        .await
        .unwrap();
    assert!(
        res.status() == 403 || res.status() == 404,
        "expected 403 or 404, got {}",
        res.status()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_create_file_empty_content() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/file"))
        .json(&serde_json::json!({ "path": "empty.txt" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let content = std::fs::read_to_string(server.dir_path().join("empty.txt")).unwrap();
    assert_eq!(content, "");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rename_file_into_new_subdirectory() {
    let server = TestServer::start().await;
    server.create_file("moveme.txt", "content");

    let res = server
        .client
        .post(server.url("/api/file/rename"))
        .json(&serde_json::json!({ "from": "moveme.txt", "to": "newdir/moved.txt" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    assert!(!server.dir_path().join("moveme.txt").exists());
    assert!(server.dir_path().join("newdir/moved.txt").is_file());
}
