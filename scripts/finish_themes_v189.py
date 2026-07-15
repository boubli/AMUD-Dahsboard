#!/usr/bin/env python3
"""Finish v1.8.9 theme icon packs + remove leftover broken theme assets."""
from __future__ import annotations

import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UI = ROOT / "ui" / "static" / "themes"
DOCS = ROOT / "docs" / "static" / "themes"
REMOVE = [
    "sunset-warm",
    "vaporwave-grid",
    "ocean-depths",
    "terminal-amber",
    "arctic-frost",
]


def delete_theme_assets(root: Path) -> None:
    for tid in REMOVE:
        for p in [
            root / f"{tid}.css",
            root / "previews" / f"{tid}.webp",
            root / "previews" / f"{tid}.jpg",
            root / "wallpapers" / f"{tid}.webp",
            root / "wallpapers" / f"{tid}.jpg",
            root / "icons" / tid,
        ]:
            if p.is_dir():
                shutil.rmtree(p)
                print("rmtree", p)
            elif p.exists():
                p.unlink()
                print("unlink", p)


def copy_pack(src_name: str, dst_name: str, color_map: dict[str, str]) -> None:
    for root in (UI, DOCS):
        src = root / "icons" / src_name
        dst = root / "icons" / dst_name
        if not src.exists():
            print("missing src", src)
            continue
        if dst.exists():
            shutil.rmtree(dst)
        shutil.copytree(src, dst)
        pack = dst / "pack.json"
        if pack.exists():
            data = json.loads(pack.read_text(encoding="utf-8"))
            data["base"] = f"/static/themes/icons/{dst_name}"
            pack.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
        for svg in dst.glob("*.svg"):
            text = svg.read_text(encoding="utf-8")
            for old, new in color_map.items():
                text = text.replace(old, new)
            svg.write_text(text, encoding="utf-8")
        print("pack", dst)


def main() -> None:
    copy_pack(
        "volcanic-ember",
        "taghawsa",
        {
            "#e85d04": "#F15A22",
            "#E85D04": "#F15A22",
            "#ff6b35": "#F15A22",
            "#dc2f02": "#F15A22",
            "#f48c06": "#00A896",
            "#faa307": "#00A896",
        },
    )
    copy_pack(
        "nord",
        "default",
        {
            "#88c0d0": "#cf6427",
            "#8FBCBB": "#cf6427",
            "#81a1c1": "#e07a3d",
            "#5e81ac": "#cf6427",
            "#eceff4": "#f8fafc",
        },
    )
    copy_pack(
        "steampunk-brass",
        "luxury-gold",
        {
            "#c9a227": "#d4af37",
            "#b8860b": "#c9a227",
            "#daa520": "#d4af37",
            "#cd853f": "#c9a227",
        },
    )

    delete_theme_assets(UI)
    delete_theme_assets(DOCS)

    for css_path in (UI / "taghawsa.css", DOCS / "taghawsa.css"):
        if not css_path.exists():
            continue
        text = css_path.read_text(encoding="utf-8")
        text = text.replace(
            "--theme-bg-fallback: #000000;", "--theme-bg-fallback: #0f1412;"
        )
        text = text.replace(
            "linear-gradient(180deg, #000000 0%, #004238 45%, #0a0e27 70%, #000000 100%)",
            "linear-gradient(180deg, #0f1412 0%, #004238 45%, #0a0e27 70%, #0f1412 100%)",
        )
        css_path.write_text(text, encoding="utf-8")
        print("fixed", css_path)

    for mp in (UI / "manifest.json", DOCS / "manifest.json"):
        data = json.loads(mp.read_text(encoding="utf-8"))
        for theme in data["themes"]:
            tid = theme["id"]
            if tid in ("default", "luxury-gold", "taghawsa"):
                theme["iconPack"] = f"/static/themes/icons/{tid}/pack.json"
        mp.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
        print("manifest", mp)


if __name__ == "__main__":
    main()
