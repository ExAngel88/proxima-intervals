set dotenv-load
set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Install/sync the Python development environment.
setup:
    uv sync --all-extras

# Formatting
format:
    cargo fmt --all
    uv run ruff format python

format-check:
    cargo fmt --all -- --check
    uv run ruff format --check python

# Linting and static checks
lint-rust:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

lint-python:
    uv run ruff check python

lint: lint-rust lint-python

# Compiler check. This is the stable, CI-friendly Rust type/compile check.
check-rust:
    cargo check --workspace --all-targets --all-features

# Optional rust-analyzer batch diagnostics. This CLI is a developer/debugging
# interface and may vary by rust-analyzer version; cargo check remains canonical.
rust-analyzer-check:
    rust-analyzer diagnostics .

# Unit tests
test-rust:
    cargo test --workspace --all-features

test-python:
    uv run pytest python/tests

test: test-rust test-python

# Build artifacts
build-rust:
    cargo build --workspace --all-features --release

# Build/install the PyO3 extension into the active uv environment for local work.
# Requires maturin to be available in the uv environment and a suitable maturin
# configuration in pyproject.toml.
develop-python:
    uv run maturin develop --release --manifest-path crates/proxima-python/Cargo.toml

# Build Python sdist/wheels using the project's configured build backend.
build-python:
    uv build

build: build-rust build-python

# Full local CI-style gate (does not publish/deploy).
ci: format-check lint check-rust test

# Publish the standalone Rust core crate to crates.io.
# Review Cargo.toml metadata, version, README/license, and run `just ci` first.
publish-core:
    cargo publish --manifest-path crates/proxima-intervals/Cargo.toml

# Publish the Python distribution to the configured package index.
# Review version/metadata and run `just ci build` first.
publish-python:
    uv publish

# Remove generated build/cache artefacts.
clean:
    cargo clean
    rm -rf .pytest_cache .ruff_cache python/.pytest_cache python/.ruff_cache
