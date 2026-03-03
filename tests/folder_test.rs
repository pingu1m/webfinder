mod common;

use common::TestServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_create_folder() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/folder"))
        .json(&serde_json::json!({ "path": "new-folder" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    assert!(server.dir_path().join("new-folder").is_dir());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_create_nested_folder() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/folder"))
        .json(&serde_json::json!({ "path": "a/b/c" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    assert!(server.dir_path().join("a/b/c").is_dir());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_create_folder_conflict() {
    let server = TestServer::start().await;
    server.create_dir("existing");

    let res = server
        .client
        .post(server.url("/api/folder"))
        .json(&serde_json::json!({ "path": "existing" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_folder() {
    let server = TestServer::start().await;
    server.create_dir("to-delete");
    server.create_file("to-delete/file.txt", "content");

    let res = server
        .client
        .delete(server.url("/api/folder?path=to-delete"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    assert!(!server.dir_path().join("to-delete").exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_folder_not_found() {
    let server = TestServer::start().await;

    let res = server
        .client
        .delete(server.url("/api/folder?path=nonexistent"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rename_folder() {
    let server = TestServer::start().await;
    server.create_dir("old-dir");
    server.create_file("old-dir/file.txt", "content");

    let res = server
        .client
        .post(server.url("/api/folder/rename"))
        .json(&serde_json::json!({ "from": "old-dir", "to": "new-dir" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    assert!(!server.dir_path().join("old-dir").exists());
    assert!(server.dir_path().join("new-dir").is_dir());
    let content = std::fs::read_to_string(
        server.dir_path().join("new-dir/file.txt"),
    )
    .unwrap();
    assert_eq!(content, "content");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delete_root_folder_blocked() {
    let server = TestServer::start().await;

    // Attempt to delete the root directory (empty path resolves to root)
    let res = server
        .client
        .delete(server.url("/api/folder?path=."))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rename_folder_to_existing() {
    let server = TestServer::start().await;
    server.create_dir("dir-a");
    server.create_dir("dir-b");

    let res = server
        .client
        .post(server.url("/api/folder/rename"))
        .json(&serde_json::json!({ "from": "dir-a", "to": "dir-b" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rename_root_folder_blocked() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/folder/rename"))
        .json(&serde_json::json!({ "from": ".", "to": "newroot" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_rename_folder_source_not_found() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/folder/rename"))
        .json(&serde_json::json!({ "from": "nonexistent", "to": "whatever" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_create_folder_path_traversal() {
    let server = TestServer::start().await;

    let res = server
        .client
        .post(server.url("/api/folder"))
        .json(&serde_json::json!({ "path": "../../evil" }))
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
async fn test_delete_folder_with_nested_contents() {
    let server = TestServer::start().await;
    server.create_dir("parent/child/grandchild");
    server.create_file("parent/child/grandchild/deep.txt", "deep content");
    server.create_file("parent/child/sibling.txt", "sibling");

    let res = server
        .client
        .delete(server.url("/api/folder?path=parent"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    assert!(!server.dir_path().join("parent").exists());
}
