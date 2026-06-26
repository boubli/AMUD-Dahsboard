#!/usr/bin/env bash
# Sync theme CSS from ui/ to docs/ (run after adding or editing bundled themes)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cp "$ROOT/ui/static/themes/"*.css "$ROOT/docs/static/themes/"
mkdir -p "$ROOT/docs/static/themes/wallpapers"
cp "$ROOT/ui/static/themes/wallpapers/"*.jpg "$ROOT/docs/static/themes/wallpapers/" 2>/dev/null || true
echo "Synced theme CSS and wallpapers: ui/static/themes -> docs/static/themes"
