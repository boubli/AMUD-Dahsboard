# Sync theme CSS from ui/ to docs/ (run after adding or editing bundled themes)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$uiThemes = Join-Path $root "ui\static\themes"
$docsThemes = Join-Path $root "docs\static\themes"
$uiWallpapers = Join-Path $uiThemes "wallpapers"
$docsWallpapers = Join-Path $docsThemes "wallpapers"

Copy-Item (Join-Path $uiThemes "*.css") $docsThemes -Force
if (Test-Path $uiWallpapers) {
    New-Item -ItemType Directory -Force -Path $docsWallpapers | Out-Null
    Copy-Item (Join-Path $uiWallpapers "*.jpg") $docsWallpapers -Force -ErrorAction SilentlyContinue
}

Write-Host "Synced theme CSS and wallpapers: ui/static/themes -> docs/static/themes"
