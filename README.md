<p align="center">
  <h1 align="center">WebFinder</h1>
  <p align="center">A fast, web-based file explorer launched from the command line.</p>
</p>

<p align="center">
  <a href="#installation">Installation</a> &bull;
  <a href="#usage">Usage</a> &bull;
  <a href="#features">Features</a> &bull;
  <a href="#configuration">Configuration</a> &bull;
  <a href="#development">Development</a> &bull;
  <a href="#license">License</a>
</p>

---

WebFinder gives you a full-featured file browser and code editor in your browser, served from a single binary with zero external dependencies. Point it at any directory and start editing, searching, and running scripts instantly.

## Features

- **File Tree** — Browse directories with a virtualized, collapsible tree. Create, rename, copy, and delete files and folders via context menu.
- **Code Editor** — Monaco-powered editor with syntax highlighting, configurable themes (`vs-dark`, `light`, etc.), font size, tab size, and word wrap.
- **Search** — Full-text content search (powered by `grep`) and filename filtering across the entire directory tree.
- **Script Runner** — Run Python, Node.js, TypeScript, Rust, and shell scripts directly from the editor. Streaming output via WebSocket.
- **Live Reload** — Filesystem watcher detects external changes and pushes updates to the browser in real time.
- **Settings Panel** — Adjust editor preferences and runner configuration on the fly; changes persist via the API.
- **Single Binary** — The React frontend is embedded at compile time using `rust-embed`. No separate web server or static file hosting needed.

## Installation

### From GitHub Releases

Download a prebuilt binary for your platform from the [Releases](../../releases) page.

| Platform | Archive |
|---|---|
| Linux x86_64 | `webfinder-v*-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `webfinder-v*-aarch64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `webfinder-v*-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `webfinder-v*-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `webfinder-v*-x86_64-pc-windows-msvc.zip` |

Extract and place the binary somewhere on your `PATH`:

```bash
tar xzf webfinder-v*-x86_64-unknown-linux-gnu.tar.gz
sudo mv webfinder-*/webfinder /usr/local/bin/
```

### From Source

Requires [Rust](https://rustup.rs/) (stable) and [Node.js](https://nodejs.org/) >= 22.

```bash
# Clone the repository
git clone https://github.com/your-org/webfinder.git
cd webfinder

# Build frontend and install the binary
cd frontend && npm ci && npm run build && cd ..
cargo install --path .
```

Or, if you have [just](https://github.com/casey/just) installed:

```bash
just frontend-install
just install
```

## Usage

```
webfinder [OPTIONS] [PATH]
```

| Argument / Flag | Description | Default |
|---|---|---|
| `PATH` | Directory to explore | `.` (current directory) |
| `-p`, `--port PORT` | Port to listen on (`0` = auto) | `0` |
| `--host HOST` | Host/IP to bind (`0.0.0.0` for all interfaces) | `127.0.0.1` |
| `--no-open` | Don't open browser automatically | opens browser |
| `-c`, `--config PATH` | Path to config file | auto-detected |

### Examples

```bash
# Explore the current directory (auto-opens browser)
webfinder

# Explore a specific project on port 8080
webfinder ~/projects/my-app --port 8080

# Bind to all interfaces (e.g. for remote access)
webfinder /srv/data --host 0.0.0.0 --port 3000

# Headless mode (don't open browser)
webfinder . --no-open
```

## Configuration

WebFinder looks for config files in this order:

1. `--config <path>` (CLI flag, highest priority)
2. `./webfinder.toml` (current directory)
3. `$XDG_CONFIG_HOME/webfinder/config.toml`
4. `~/.config/webfinder/config.toml`

If no config is found, sensible defaults are used. See [`webfinder.example.toml`](webfinder.example.toml) for all options:

```toml
[server]
host = "127.0.0.1"
port = 0                          # 0 = auto-select a free port
open_browser = true

[editor]
font_size = 14
tab_size = 2
word_wrap = "on"
theme = "vs-dark"

[filesystem]
show_hidden = false
max_file_size_bytes = 10485760    # 10 MB
exclude_patterns = ["node_modules", ".git", "target", "__pycache__"]

# Runners — define commands to execute files by extension.
# {file} is replaced with the absolute path to the file.
[runners.python]
command = "python3"
args = ["{file}"]
extensions = ["py"]

[runners.node]
command = "node"
args = ["{file}"]
extensions = ["js", "mjs"]
```

## Architecture

WebFinder is a Rust + React application compiled into a single binary.

```
┌──────────────────────────────────────────────┐
│                   Browser                     │
│  React 19 · Monaco Editor · Tailwind CSS v4  │
│  Zustand · TanStack Query · Radix UI         │
└──────────────┬──────────────┬────────────────┘
               │ REST API     │ WebSocket
┌──────────────┴──────────────┴────────────────┐
│              Rust Backend (Axum)              │
│  File I/O · FS Watcher · Script Runner       │
│  Config · rust-embed (serves frontend)       │
└──────────────────────────────────────────────┘
```

### API Endpoints

| Method | Route | Description |
|---|---|---|
| `GET` | `/api/tree` | File tree |
| `GET` | `/api/file?path=` | Read file |
| `PUT` | `/api/file` | Write file |
| `POST` | `/api/file` | Create file |
| `DELETE` | `/api/file?path=` | Delete file |
| `POST` | `/api/file/rename` | Rename file |
| `POST` | `/api/file/copy` | Copy file |
| `POST` | `/api/folder` | Create folder |
| `DELETE` | `/api/folder?path=` | Delete folder |
| `POST` | `/api/folder/rename` | Rename folder |
| `GET` | `/api/search?q=` | Search files |
| `GET` | `/api/info` | Server info & config |
| `PUT` | `/api/settings` | Update settings |
| `POST` | `/api/run` | Start script runner |
| `DELETE` | `/api/run/:id` | Stop runner |
| `GET` | `/api/run/:id` | Runner status |
| `GET` | `/api/watch` | WebSocket — file change events |
| `GET` | `/api/run/:id/stream` | WebSocket — runner output |

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) >= 22
- [just](https://github.com/casey/just) (optional, recommended)

### Quick Start

```bash
# Install frontend dependencies
just frontend-install

# Start frontend (Vite HMR) + backend in dev mode
just dev
```

The Vite dev server proxies `/api` requests to the Rust backend on port 3000.

### Available Commands

| Command | Description |
|---|---|
| `just build` | Build frontend + Rust binary (debug) |
| `just build-release` | Build frontend + Rust binary (release, optimized) |
| `just install` | Build and install to `~/.cargo/bin` |
| `just dev` | Start Vite + Rust backend in dev mode |
| `just dev-frontend` | Vite dev server only |
| `just dev-backend` | Rust backend only (port 3000) |
| `just test-api` | Run Rust API tests |
| `just test-e2e` | Run Playwright E2E tests |
| `just test` | Run all tests |
| `just check` | Cargo check + TypeScript type-check |
| `just lint` | Cargo clippy |
| `just clean` | Remove all build artifacts |

### Running Tests

```bash
# API tests (Rust)
just test-api

# E2E tests (Playwright — requires a built binary)
just test-e2e

# Everything
just test
```

## Tech Stack

| Layer | Technology |
|---|---|
| Backend | Rust, Axum, Tokio, rust-embed |
| Frontend | React 19, TypeScript, Vite |
| Editor | Monaco Editor |
| UI | Radix UI, Tailwind CSS v4, Lucide Icons |
| State | Zustand, TanStack Query |
| Testing | Cargo test, Playwright |

## License

[MIT](LICENSE) &copy; Felipe Gusmao
