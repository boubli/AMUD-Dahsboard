#!/usr/bin/env python3
"""Validate theme manifest v5 and local bundled assets."""

from __future__ import annotations

import json
import sys
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


def local_path(url_or_path: str) -> Path | None:
    if not url_or_path or not url_or_path.startswith("/static/"):
        return None
    rel = url_or_path.removeprefix("/static/")
    return ROOT / "ui" / "static" / rel


def main() -> int:
    errors: list[str] = []
    with open(MANIFEST, encoding="utf-8") as f:
        manifest = json.load(f)

    if manifest.get("version") != 5:
        errors.append("manifest version must be 5")

    base = manifest.get("assetBase") or ""
    if base != "/static/themes":
        errors.append(f"assetBase should be /static/themes (got {base!r})")

    for theme in manifest.get("themes", []):
        tid = theme["id"]
        if tid in FROZEN:
            continue
        css_file = theme.get("file")
        if css_file:
            path = ROOT / "ui" / "static" / "themes" / css_file
            if not path.is_file():
                errors.append(f"{tid}: missing CSS {css_file}")

        pack_path = local_path(theme.get("iconPack", ""))
        if not pack_path or not pack_path.is_file():
            errors.append(f"{tid}: missing icon pack {theme.get('iconPack')}")
        else:
            pack = json.loads(pack_path.read_text(encoding="utf-8"))
            pack_dir = pack_path.parent
            for name in ICON_NAMES:
                if name not in pack.get("icons", {}):
                    errors.append(f"{tid}: pack missing icon {name}")
                else:
                    svg = pack_dir / pack["icons"][name]
                    if not svg.is_file():
                        errors.append(f"{tid}: missing SVG {name}")

        for key in ("preview", "wallpaper"):
            val = theme.get(key)
            if not val:
                continue
            p = local_path(val)
            if p and not p.is_file():
                errors.append(f"{tid}: missing {key} file {val}")

        layout = theme.get("layoutCss")
        profile = theme.get("uiProfile")
        if layout and profile:
            layout_file = ROOT / "ui" / "static" / "theme-layouts" / f"{profile}.css"
            if not layout_file.is_file():
                errors.append(f"{tid}: missing layout CSS for profile {profile}")

    if errors:
        print("Theme validation issues:")
        for e in errors:
            print(" -", e)
        return 1

    print("Theme assets validation passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
