#!/usr/bin/env bash
# Prepare themes-assets for GitHub CDN (run from repo root).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
python3 scripts/generate-theme-overhaul.py
python3 scripts/validate-theme-assets.py
echo "themes-assets ready. Commit themes-assets/ and ui/static/themes/, then push to GitHub for jsDelivr."
