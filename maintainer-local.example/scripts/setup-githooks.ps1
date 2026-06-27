# Enable repo git hooks (run once after clone). Matches CI rustfmt on every commit.
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root

git config core.hooksPath maintainer-local/.githooks
Write-Host "Git hooks enabled: core.hooksPath=maintainer-local/.githooks"
Write-Host "Pre-commit runs 'cargo fmt --all' before each commit (same as CI)."
