#!/usr/bin/env bash
# Run the same Rust checks as GitHub Actions CI (fmt, clippy, test).
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

echo "== cargo fmt --check =="
cargo fmt --all -- --check

echo "== cargo clippy =="
cargo clippy --workspace --all-targets -- -D warnings

echo "== cargo test =="
cargo test --workspace

echo "All Rust CI checks passed."
