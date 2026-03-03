use std::net::SocketAddr;
use std::path::Path;

use tempfile::TempDir;
use tokio::net::TcpListener;

use webfinder::config::Config;
use webfinder::fs::walk;
use webfinder::server::build_router;
use webfinder::state::AppState;

pub struct TestServer {
    pub addr: SocketAddr,
    pub dir: TempDir,
    pub client: reqwest::Client,
    pub watch_tx: tokio::sync::broadcast::Sender<webfinder::state::FsEvent>,
}

impl TestServer {
    pub async fn start() -> Self {
        Self::start_with_setup(|_| {}).await
    }

    pub async fn start_with_setup(setup: impl FnOnce(&Path)) -> Self {
        let dir = TempDir::new().unwrap();
        setup(dir.path());

        let root = dunce::canonicalize(dir.path()).unwrap();
        let config = Config::default();
        let state = AppState::new(root.clone(), config.clone());

        // Walk tree
        let fs_config = config.filesystem.clone();
        let root_clone = root.clone();
        let tree = tokio::task::spawn_blocking(move || walk::walk_tree(&root_clone, &fs_config))
            .await
            .unwrap();
        {
            let mut cache = state.tree_cache.write().await;
            *cache = tree;
        }

        // Watcher is skipped in tests for speed and to avoid blocking threads.
        // FS events are not needed for API E2E tests.

        let watch_tx = state.watch_tx.clone();
        let app = build_router(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give the server a moment to start accepting connections
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        Self {
            addr,
            dir,
            client,
            watch_tx,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }

    pub fn dir_path(&self) -> &Path {
        self.dir.path()
    }

    pub fn create_file(&self, relative: &str, content: &str) {
        let path = self.dir.path().join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    pub fn create_dir(&self, relative: &str) {
        std::fs::create_dir_all(self.dir.path().join(relative)).unwrap();
    }
}
