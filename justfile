# WebFinder — Development Commands

# Show available commands
default:
    @just --list

# ─── Build ───────────────────────────────────────────────────────────────────

# Install frontend dependencies
frontend-install:
    cd frontend && npm install

# Build the frontend for production
frontend-build:
    cd frontend && npm run build

# Build the Rust binary (debug)
build: frontend-build
    cargo build

# Build the Rust binary (release, optimized)
build-release: frontend-build
    cargo build --release

# Install webfinder to ~/.cargo/bin
install: frontend-build
    cargo install --path .

# ─── Dev ─────────────────────────────────────────────────────────────────────

# Start the frontend dev server (Vite HMR)
dev-frontend:
    cd frontend && npm run dev

# Start the Rust backend pointing at a directory (default: current dir)
dev-backend dir=".":
    cargo run -- --no-open --port 3000 {{dir}}

# Start both frontend (Vite) and backend in dev mode
dev dir=".":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting Vite dev server..."
    cd frontend && npm run dev &
    VITE_PID=$!
    sleep 2
    echo "Starting Rust backend on port 3000..."
    cargo run -- --no-open --port 3000 {{dir}} &
    RUST_PID=$!
    trap "kill $VITE_PID $RUST_PID 2>/dev/null" EXIT
    wait

# Run the production binary against a directory
run dir=".":
    cargo run --release -- {{dir}}

# ─── Test ────────────────────────────────────────────────────────────────────

# Run all Rust API E2E tests
test-api:
    cargo test

# Run a specific Rust test file (e.g. just test-api-file file_test)
test-api-file name:
    cargo test --test {{name}}

# Run all Playwright E2E tests
test-e2e: frontend-build build
    cd frontend && npx playwright test

# Run a specific Playwright spec (e.g. just test-e2e-file editor)
test-e2e-file name:
    cd frontend && npx playwright test e2e/{{name}}.spec.ts

# Run all tests (API + E2E)
test: test-api test-e2e

# ─── Lint / Check ───────────────────────────────────────────────────────────

# Type-check the frontend
check-frontend:
    cd frontend && npx tsc -b

# Cargo check (fast compile check, no codegen)
check-backend:
    cargo check

# Check everything
check: check-backend check-frontend

# Cargo clippy
lint:
    cargo clippy -- -W clippy::all

# ─── Clean ───────────────────────────────────────────────────────────────────

# Remove Rust build artifacts
clean-rust:
    cargo clean

# Remove frontend build artifacts
clean-frontend:
    rm -rf frontend/dist frontend/node_modules

# Remove Playwright test results
clean-test-results:
    rm -rf frontend/test-results frontend/playwright-report

# Remove everything
clean: clean-rust clean-frontend clean-test-results
