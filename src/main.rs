mod config;
mod error;
mod frontend;
mod fs;
mod handlers;
mod runner;
mod server;
mod state;

use std::path::PathBuf;

use clap::Parser;
use tokio::net::TcpListener;

use crate::config::load_config;
use crate::fs::{walk, watcher};
use crate::state::AppState;

#[derive(Parser)]
#[command(name = "webfinder", about = "A fast, web-based file explorer")]
struct Cli {
    /// Directory to explore (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Port to listen on (0 = auto-select free port)
    #[arg(short, long)]
    port: Option<u16>,

    /// Host/IP to bind to (e.g. 0.0.0.0 for all interfaces)
    #[arg(long)]
    host: Option<String>,

    /// Don't open browser automatically
    #[arg(long)]
    no_open: bool,

    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "webfinder=info,tower_http=info".into()),
        )
        .init();

    let cli = Cli::parse();

    let mut config = load_config(cli.config.as_deref())?;

    // CLI overrides
    if let Some(port) = cli.port {
        config.server.port = port;
    }
    if let Some(host) = cli.host {
        config.server.host = host;
    }
    if cli.no_open {
        config.server.open_browser = false;
    }

    let root = dunce::canonicalize(&cli.path)?;
    tracing::info!(root = %root.display(), "starting webfinder");

    let state = AppState::new(root.clone(), config.clone());

    // Initial tree walk (async, but we await it before serving)
    let tree = {
        let root = root.clone();
        let fs_config = config.filesystem.clone();
        tokio::task::spawn_blocking(move || walk::walk_tree(&root, &fs_config)).await?
    };
    {
        let mut cache = state.tree_cache.write().await;
        *cache = tree;
    }
    tracing::info!("tree cache populated");

    // Spawn filesystem watcher
    watcher::spawn_watcher(
        root.clone(),
        config.filesystem.clone(),
        state.tree_cache.clone(),
        state.watch_tx.clone(),
    )?;
    tracing::info!("filesystem watcher started");

    let app = server::build_router(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    let local_addr = listener.local_addr()?;
    let url = format!("http://{local_addr}");

    tracing::info!(%url, "webfinder ready");
    eprintln!("\n  webfinder serving {} at {}\n", root.display(), url);

    if config.server.open_browser {
        let _ = open::that(&url);
    }

    axum::serve(listener, app).await?;

    Ok(())
}
