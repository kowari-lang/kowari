#!/usr/bin/env bash
# Mirrors what CI runs (PLANNING.md §6.3).
set -euo pipefail
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release