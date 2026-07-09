# Enable repo git hooks (run once after clone).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

git config core.hooksPath .githooks
Write-Host "Git hooks enabled: core.hooksPath=.githooks"
Write-Host "  pre-commit  — cargo fmt --all (auto-stage *.rs)"
Write-Host "  pre-push    — scripts/ci-check.ps1 (fmt + clippy + test, mirrors CI)"
Write-Host ""
Write-Host "Windows cannot compile #[cfg(unix)] agent paths; unix-only code is validated on GitHub CI (ubuntu-latest)."
