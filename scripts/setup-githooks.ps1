# Enable repo git hooks (run once after clone).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

git config core.hooksPath .githooks
Write-Host "Git hooks enabled: core.hooksPath=.githooks"
Write-Host "Pre-commit will run 'cargo fmt --all' automatically before each commit."
