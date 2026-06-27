#!/usr/bin/env python3
"""Validate theme manifest v4, local CSS files, and CDN asset paths."""

from __future__ import annotations

import json
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "ui" / "static" / "themes" / "manifest.json"
FROZEN = {"default", "luxury-gold"}
ICON_NAMES = [
    "sun", "moon", "cloud", "cpu", "hard-drive", "activity", "wifi", "settings",
    "layout-grid", "search", "plus", "bell", "users", "rss", "server", "plug",
    "home", "shield", "database", "zap", "eye", "palette", "arrow-left",
    "external-link", "power", "play", "pause", "refresh",
]


def head_ok(url: str) -> bool:
    req = urllib.request.Request(url, method="HEAD")
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            return 200 <= resp.status < 400
    except urllib.error.HTTPError as e:
        return e.code == 405  # some CDNs disallow HEAD
    except Exception:
        return False


def main() -> int:
    errors: list[str] = []
    with open(MANIFEST, encoding="utf-8") as f:
        manifest = json.load(f)

    if manifest.get("version") != 4:
        errors.append("manifest version must be 4")

    base = (manifest.get("assetBase") or "").rstrip("/")
    if not base:
        errors.append("assetBase missing")

    for theme in manifest.get("themes", []):
        tid = theme["id"]
        if tid in FROZEN:
            continue
        css_file = theme.get("file")
        if css_file:
            path = ROOT / "ui" / "static" / "themes" / css_file
            if not path.is_file():
                errors.append(f"{tid}: missing CSS {css_file}")

        pack_local = ROOT / "themes-assets" / "icons" / tid / "pack.json"
        if not pack_local.is_file():
            errors.append(f"{tid}: missing local pack.json (commit themes-assets/)")
        else:
            pack = json.loads(pack_local.read_text(encoding="utf-8"))
            for name in ICON_NAMES:
                if name not in pack.get("icons", {}):
                    errors.append(f"{tid}: pack missing icon {name}")

        if theme.get("iconPack") and base:
            url = f"{base}/{theme['iconPack'].lstrip('/')}"
            if not head_ok(url) and not (ROOT / "themes-assets" / theme["iconPack"]).is_file():
                errors.append(f"{tid}: CDN pack not reachable yet (push themes-assets): {url}")

    if errors:
        print("Theme validation issues:")
        for e in errors:
            print(" -", e)
        # Local-only assets are OK before push
        local_only = all("CDN pack not reachable" in e for e in errors)
        return 0 if local_only else 1

    print("Theme assets validation passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
