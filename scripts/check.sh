#!/usr/bin/env bash
# Mirrors what CI runs (Planning.md §6.3) / Ci shell command.
set -euo pipefail
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace