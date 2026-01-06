#!/usr/bin/env bash
set -euo pipefail

# Formatting
if [[ "${CI:-}" == "true" ]]; then
  cargo fmt --all -- --check
else
  cargo fmt --all
fi

# Lints
cargo clippy --all-targets --all-features -- -D warnings

# Tests (dev profile)
cargo test --all-targets --all-features

# Release build
cargo build --release --all-targets --all-features
