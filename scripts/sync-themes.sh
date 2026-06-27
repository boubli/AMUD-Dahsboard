#!/usr/bin/env bash
# Sync theme assets from ui/ to docs/ (run after adding or editing bundled themes)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cp "$ROOT/ui/static/themes/"*.css "$ROOT/docs/static/themes/"
mkdir -p "$ROOT/docs/static/themes/wallpapers" "$ROOT/docs/static/themes/previews"
cp "$ROOT/ui/static/themes/wallpapers/"*.webp "$ROOT/docs/static/themes/wallpapers/" 2>/dev/null || true
cp "$ROOT/ui/static/themes/previews/"*.webp "$ROOT/docs/static/themes/previews/" 2>/dev/null || true
cp "$ROOT/ui/static/themes/manifest.json" "$ROOT/docs/static/themes/"
mkdir -p "$ROOT/docs/static/theme-layouts"
cp "$ROOT/ui/static/theme-layouts/"*.css "$ROOT/docs/static/theme-layouts/" 2>/dev/null || true
echo "Synced themes: ui/static/themes -> docs/static/themes"
