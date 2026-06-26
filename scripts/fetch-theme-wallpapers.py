#!/usr/bin/env python3
"""Download vendored 2560px theme wallpapers from Unsplash/Pexels (offline-safe after fetch)."""

from __future__ import annotations

import hashlib
import json
import sys
import urllib.request
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
UI_WP = ROOT / "ui" / "static" / "themes" / "wallpapers"
UI_PR = ROOT / "ui" / "static" / "themes" / "previews"
DOCS_WP = ROOT / "docs" / "static" / "themes" / "wallpapers"
DOCS_PR = ROOT / "docs" / "static" / "themes" / "previews"
MANIFEST = ROOT / "ui" / "static" / "themes" / "manifest.json"

# theme_id -> (source, url, credit line)
# Each URL chosen to match theme mood; all unique photos.
SOURCES: dict[str, tuple[str, str, str]] = {
    # Classic + advanced (refresh vendored copies; already curated)
    "dracula": (
        "unsplash",
        "https://images.unsplash.com/photo-1557682250-33bd709cbe85?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1557682250",
    ),
    "nord": (
        "pexels",
        "https://images.pexels.com/photos/1933239/pexels-photo-1933239.jpeg?auto=compress&cs=tinysrgb&w=2560",
        "Pexels 1933239 (aurora)",
    ),
    "cyberpunk-neon": (
        "pexels",
        "https://images.pexels.com/photos/3254761/pexels-photo-3254761.jpeg?auto=compress&cs=tinysrgb&w=2560",
        "Pexels 3254761 (neon)",
    ),
    "sunset-warm": (
        "unsplash",
        "https://images.unsplash.com/photo-1507525428034-b723cf961d3e?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1507525428034",
    ),
    "catppuccin-mocha": (
        "unsplash",
        "https://images.unsplash.com/photo-1558618666-fcd25c85cd64?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1558618666",
    ),
    "gruvbox-dark": (
        "unsplash",
        "https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1506905925346",
    ),
    "tokyo-night": (
        "pexels",
        "https://images.pexels.com/photos/2506923/pexels-photo-2506923.jpeg?auto=compress&cs=tinysrgb&w=2560",
        "Pexels 2506923 (city night)",
    ),
    "one-dark": (
        "unsplash",
        "https://images.unsplash.com/photo-1451187580459-43490279c0fa?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1451187580459",
    ),
    "everforest": (
        "unsplash",
        "https://images.unsplash.com/photo-1441974231531-c6227db76b6e?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1441974231531",
    ),
    "monokai": (
        "unsplash",
        "https://images.unsplash.com/photo-1470071459604-3b5ec3a7fe05?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1470071459604",
    ),
    "rose-pine": (
        "unsplash",
        "https://images.unsplash.com/photo-1464822759023-fed622ff2c3b?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1464822759023",
    ),
    "solarized-dark": (
        "unsplash",
        "https://images.unsplash.com/photo-1469474968028-56623f02e42e?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1469474968028",
    ),
    "default": (
        "unsplash",
        "https://images.unsplash.com/photo-1451187580459-43490279c0fa?w=2560&q=80&auto=format&fit=crop&sat=-40",
        "Unsplash photo-1451187580459 (desaturated)",
    ),
    "vaporwave-grid": (
        "pexels",
        "https://images.pexels.com/photos/360912/pexels-photo-360912.jpeg?auto=compress&cs=tinysrgb&w=2560",
        "Pexels 360912 (synthwave sunset)",
    ),
    "luxury-gold": (
        "unsplash",
        "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1618005182384 (gold abstract)",
    ),
    "holographic-prism": (
        "unsplash",
        "https://images.unsplash.com/photo-1614850523459-c2f4c699c52e?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1614850523459 (light prism)",
    ),
    # Nature
    "aurora-borealis": (
        "unsplash",
        "https://images.unsplash.com/photo-1531366936337-7c912a4589a7?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1531366936337 (aurora)",
    ),
    "desert-dusk": (
        "unsplash",
        "https://images.unsplash.com/photo-1509316785289-025f5b846b35?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1509316785289 (desert dunes)",
    ),
    "ocean-depths": (
        "unsplash",
        "https://images.unsplash.com/photo-1559827260-dc66d52bef19?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1559827260 (ocean)",
    ),
    "rainforest-mist": (
        "pexels",
        "https://images.pexels.com/photos/1770809/pexels-photo-1770809.jpeg?auto=compress&cs=tinysrgb&w=2560",
        "Pexels 1770809 (misty forest)",
    ),
    "volcanic-ember": (
        "pexels",
        "https://images.pexels.com/photos/417173/pexels-photo-417173.jpeg?auto=compress&cs=tinysrgb&w=2560",
        "Pexels 417173 (volcanic landscape)",
    ),
    # Feminine
    "sakura-dream": (
        "unsplash",
        "https://images.unsplash.com/photo-1522383225653-ed111181a951?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1522383225653 (cherry blossom)",
    ),
    "lavender-mist": (
        "pexels",
        "https://images.pexels.com/photos/931162/pexels-photo-931162.jpeg?auto=compress&cs=tinysrgb&w=2560",
        "Pexels 931162 (lavender field)",
    ),
    "rose-gold-blush": (
        "pexels",
        "https://images.pexels.com/photos/360756/pexels-photo-360756.jpeg?auto=compress&cs=tinysrgb&w=2560",
        "Pexels 360756 (pink roses)",
    ),
    "cotton-candy": (
        "unsplash",
        "https://images.unsplash.com/photo-1501594907352-04cda38ebc29?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1501594907352 (pastel sky)",
    ),
    "peach-blossom": (
        "unsplash",
        "https://images.unsplash.com/photo-1490750967868-88aa4486c946?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1490750967868 (peach flowers)",
    ),
    # Variety
    "nebula-void": (
        "unsplash",
        "https://images.unsplash.com/photo-1419242902214-272b3f66ee7a?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1419242902214 (milky way)",
    ),
    "arctic-frost": (
        "pexels",
        "https://images.pexels.com/photos/1287145/pexels-photo-1287145.jpeg?auto=compress&cs=tinysrgb&w=2560",
        "Pexels 1287145 (arctic snow)",
    ),
    "steampunk-brass": (
        "unsplash",
        "https://images.unsplash.com/photo-1581092160562-40aa08e78837?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1581092160562 (industrial gears)",
    ),
    "zen-garden": (
        "unsplash",
        "https://images.unsplash.com/photo-1528360983277-13d401cdc186?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1528360983277 (zen garden)",
    ),
    "retro-arcade": (
        "unsplash",
        "https://images.unsplash.com/photo-1511512578047-dfb367046420?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1511512578047 (arcade)",
    ),
    "midnight-city": (
        "unsplash",
        "https://images.unsplash.com/photo-1514565131-fce0801e5785?w=2560&q=85&auto=format&fit=crop",
        "Unsplash photo-1514565131 (city skyline night)",
    ),
    # Preview thumbnails for CSS-only themes
    "terminal-phosphor": (
        "unsplash",
        "https://images.unsplash.com/photo-1550751827-4bd374c3f58b?w=1280&q=85&auto=format&fit=crop",
        "Unsplash photo-1550751827 (green terminal mood)",
    ),
    "terminal-amber": (
        "unsplash",
        "https://images.unsplash.com/photo-1629654297299-c8506221ca97?w=1280&q=85&auto=format&fit=crop",
        "Unsplash photo-1629654297299 (amber CRT mood)",
    ),
    "terminal-matrix": (
        "unsplash",
        "https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?w=1280&q=85&auto=format&fit=crop",
        "Unsplash photo-1526374965328 (matrix code)",
    ),
    "blueprint-tech": (
        "unsplash",
        "https://images.unsplash.com/photo-1503387762-592deb58ef4e?w=1280&q=85&auto=format&fit=crop",
        "Unsplash photo-1503387762 (blueprint)",
    ),
    "brutalist-mono": (
        "unsplash",
        "https://images.unsplash.com/photo-1487958449943-2429e8be8625?w=1280&q=85&auto=format&fit=crop",
        "Unsplash photo-1487958449943 (concrete architecture)",
    ),
}

PREVIEW_ONLY = {
    "terminal-phosphor",
    "terminal-amber",
    "terminal-matrix",
    "blueprint-tech",
    "brutalist-mono",
}


def download(url: str, dest: Path) -> None:
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "AMUD-Dashboard/1.5 theme-wallpaper-fetch"},
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = resp.read()
    if len(data) < 10_000:
        raise RuntimeError(f"download too small ({len(data)} bytes) from {url}")
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(data)


def save_preview(src: Path, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    with Image.open(src) as img:
        img.convert("RGB").resize((640, 400), Image.Resampling.LANCZOS).save(
            dest, "JPEG", quality=85, optimize=True
        )


def license_label(source: str) -> str:
    return "Unsplash License" if source == "unsplash" else "Pexels License"


def install_wallpaper(theme_id: str, tmp: Path, source: str, credit: str) -> tuple[str, str, str]:
    for wp_dir in (UI_WP, DOCS_WP):
        out = wp_dir / f"{theme_id}.jpg"
        if out.exists():
            out.unlink()
        out.write_bytes(tmp.read_bytes())
    return (f"{theme_id}.jpg", credit, license_label(source))


def install_previews(theme_id: str, tmp: Path) -> None:
    for pr_dir in (UI_PR, DOCS_PR):
        save_preview(tmp, pr_dir / f"{theme_id}.jpg")


def fetch_theme(
    theme_id: str, source: str, url: str, credit: str
) -> tuple[str, str, str] | None:
    tmp = UI_WP / f"{theme_id}.download"
    download(url, tmp)
    try:
        credit_row = None
        if theme_id not in PREVIEW_ONLY:
            credit_row = install_wallpaper(theme_id, tmp, source, credit)
        install_previews(theme_id, tmp)
        return credit_row
    finally:
        tmp.unlink(missing_ok=True)


def verify_unique_wallpapers() -> list[list[str]]:
    hashes: dict[str, list[str]] = {}
    for jpg in UI_WP.glob("*.jpg"):
        digest = hashlib.sha256(jpg.read_bytes()).hexdigest()
        hashes.setdefault(digest, []).append(jpg.name)
    return [names for names in hashes.values() if len(names) > 1]


def main() -> int:
    UI_WP.mkdir(parents=True, exist_ok=True)
    UI_PR.mkdir(parents=True, exist_ok=True)
    DOCS_WP.mkdir(parents=True, exist_ok=True)
    DOCS_PR.mkdir(parents=True, exist_ok=True)

    failed: list[str] = []
    credit_rows: list[tuple[str, str, str]] = []

    for theme_id, (source, url, credit) in SOURCES.items():
        try:
            credit_row = fetch_theme(theme_id, source, url, credit)
            if credit_row:
                credit_rows.append(credit_row)
            print(f"OK  {theme_id}")
        except (OSError, RuntimeError) as exc:
            failed.append(f"{theme_id}: {exc}")
            print(f"FAIL {theme_id}: {exc}", file=sys.stderr)

    if failed:
        print(f"\n{len(failed)} download(s) failed", file=sys.stderr)
        return 1

    dups = verify_unique_wallpapers()
    if dups:
        print("WARNING duplicate wallpaper hashes:", dups, file=sys.stderr)
        return 1

    write_credits(credit_rows)
    print(f"done — {len(credit_rows)} wallpaper files, all unique hashes")
    return 0


def write_credits(rows: list[tuple[str, str, str]]) -> None:
  lines = [
        "# Wallpaper image credits",
        "",
        "Wallpapers are **vendored** in this repository (2560px JPEG) for offline use and stable GitHub Pages URLs.",
        "Re-fetch with `python scripts/fetch-theme-wallpapers.py`.",
        "",
        "| File | Source | License |",
        "|------|--------|---------|",
    ]
  for file_name, source, license_name in sorted(rows, key=lambda r: r[0]):
        lines.append(f"| `{file_name}` | {source} | {license_name} |")
  text = "\n".join(lines) + "\n"
  for path in (
        ROOT / "docs" / "static" / "themes" / "wallpapers" / "CREDITS.md",
        ROOT / "ui" / "static" / "themes" / "wallpapers" / "CREDITS.md",
    ):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
