# Run the same Rust checks as GitHub Actions CI (fmt, clippy, test).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root

Write-Host "== cargo fmt --check =="
cargo fmt --all -- --check

Write-Host "== cargo clippy =="
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "== cargo test =="
cargo test --workspace

Write-Host "All Rust CI checks passed."
