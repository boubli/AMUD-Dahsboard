# Mirror .github/workflows/ci.yml rust job (run before push).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "==> cargo fmt --check"
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> cargo test"
cargo test --workspace --lib
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "CI checks passed."
Write-Host "Note: Windows skips #[cfg(unix)] agent code; unix-only paths are validated on GitHub CI (ubuntu-latest)."
