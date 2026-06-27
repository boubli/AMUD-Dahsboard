#!/usr/bin/env python3
"""Validate theme manifest v5 and local bundled assets."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from amud_paths import repo_root

ROOT = repo_root()
MANIFEST = ROOT / "ui" / "static" / "themes" / "manifest.json"
FROZEN = {"default", "luxury-gold"}
ICON_NAMES = [
    "sun", "moon", "cloud", "cpu", "hard-drive", "activity", "wifi", "settings",
    "layout-grid", "layout-template", "search", "plus", "bell", "users", "rss", "server", "plug",
    "home", "shield", "shield-check", "database", "zap", "eye", "palette", "arrow-left",
    "external-link", "power", "play", "pause", "refresh", "cloud-sun", "heart", "tag", "scroll-text",
]


def local_path(url_or_path: str) -> Path | None:
    if not url_or_path or not url_or_path.startswith("/static/"):
        return None
    rel = url_or_path.removeprefix("/static/")
    return ROOT / "ui" / "static" / rel


def validate_manifest_header(manifest: dict) -> list[str]:
    errors: list[str] = []
    if manifest.get("version") != 5:
        errors.append("manifest version must be 5")
    base = manifest.get("assetBase") or ""
    if base != "/static/themes":
        errors.append(f"assetBase should be /static/themes (got {base!r})")
    return errors


def validate_theme_css(tid: str, theme: dict) -> list[str]:
    css_file = theme.get("file")
    if not css_file:
        return []
    path = ROOT / "ui" / "static" / "themes" / css_file
    if path.is_file():
        return []
    return [f"{tid}: missing CSS {css_file}"]


def validate_icon_pack(tid: str, theme: dict) -> list[str]:
    errors: list[str] = []
    pack_path = local_path(theme.get("iconPack", ""))
    if not pack_path or not pack_path.is_file():
        return [f"{tid}: missing icon pack {theme.get('iconPack')}"]
    pack = json.loads(pack_path.read_text(encoding="utf-8"))
    pack_dir = pack_path.parent
    for name in ICON_NAMES:
        icon_file = pack.get("icons", {}).get(name)
        if not icon_file:
            errors.append(f"{tid}: pack missing icon {name}")
            continue
        if not (pack_dir / icon_file).is_file():
            errors.append(f"{tid}: missing SVG {name}")
    return errors


def validate_media_files(tid: str, theme: dict) -> list[str]:
    errors: list[str] = []
    for key in ("preview", "wallpaper"):
        val = theme.get(key)
        if not val:
            continue
        path = local_path(val)
        if path and not path.is_file():
            errors.append(f"{tid}: missing {key} file {val}")
    return errors


def validate_layout_css(tid: str, theme: dict) -> list[str]:
    profile = theme.get("uiProfile")
    if not theme.get("layoutCss") or not profile:
        return []
    layout_file = ROOT / "ui" / "static" / "theme-layouts" / f"{profile}.css"
    if layout_file.is_file():
        return []
    return [f"{tid}: missing layout CSS for profile {profile}"]


def validate_theme(tid: str, theme: dict) -> list[str]:
    if tid in FROZEN:
        return []
    errors: list[str] = []
    errors.extend(validate_theme_css(tid, theme))
    errors.extend(validate_icon_pack(tid, theme))
    errors.extend(validate_media_files(tid, theme))
    errors.extend(validate_layout_css(tid, theme))
    return errors


def main() -> int:
    with open(MANIFEST, encoding="utf-8") as f:
        manifest = json.load(f)

    errors = validate_manifest_header(manifest)
    for theme in manifest.get("themes", []):
        errors.extend(validate_theme(theme["id"], theme))

    if errors:
        print("Theme validation issues:")
        for err in errors:
            print(" -", err)
        return 1

    print("Theme assets validation passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
