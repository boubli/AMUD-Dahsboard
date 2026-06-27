#!/usr/bin/env python3
"""Convert theme JPG wallpapers/previews to WebP under ui/static/themes/."""

from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image

from amud_paths import repo_root

ROOT = repo_root()
PAIRS = (
    (ROOT / "ui" / "static" / "themes" / "wallpapers", 1920, 82),
    (ROOT / "ui" / "static" / "themes" / "previews", 640, 80),
    (ROOT / "docs" / "static" / "themes" / "wallpapers", 1920, 82),
    (ROOT / "docs" / "static" / "themes" / "previews", 640, 80),
)


def convert_dir(src_dir: Path, max_width: int, quality: int) -> int:
    if not src_dir.is_dir():
        return 0
    count = 0
    for jpg in sorted(src_dir.glob("*.jpg")):
        webp = jpg.with_suffix(".webp")
        with Image.open(jpg) as img:
            img = img.convert("RGB")
            w, h = img.size
            if w > max_width:
                nh = int(h * max_width / w)
                img = img.resize((max_width, nh), Image.Resampling.LANCZOS)
            img.save(webp, "WEBP", quality=quality, method=6)
        count += 1
        print(f"  {jpg.name} -> {webp.name}")
    return count


def main() -> int:
    total = 0
    for directory, max_w, q in PAIRS:
        print(f"Converting {directory} (max {max_w}px, q={q})")
        total += convert_dir(directory, max_w, q)
    if total == 0:
        print("No JPG files found. Run scripts/fetch-theme-wallpapers.py first.", file=sys.stderr)
        return 1
    print(f"Converted {total} image(s) to WebP")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
