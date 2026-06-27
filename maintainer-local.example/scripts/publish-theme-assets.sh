#!/usr/bin/env bash
# Deprecated — v1.6.2+ bundles assets in ui/static/themes/. Regenerate with:
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
python3 maintainer-local/scripts/generate-theme-overhaul.py
python3 maintainer-local/scripts/validate-theme-assets.py
echo "Done. Commit ui/static/themes/ only (no CDN themes-assets/)."
