#!/usr/bin/env python3
"""Replace five broken themes with five new ones; mirror ui + docs manifests."""
from __future__ import annotations

import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] if (Path(__file__).name == "gen_themes_v189.py") else Path(".")
UI = ROOT / "ui" / "static" / "themes"
DOCS = ROOT / "docs" / "static" / "themes"

REMOVE = [
    "sunset-warm",
    "vaporwave-grid",
    "ocean-depths",
    "terminal-amber",
    "arctic-frost",
]

# old_id provides assets to clone (preview/wallpaper/icons/layout profile)
NEW = [
    {
        "id": "ember-hearth",
        "name": "Ember Hearth",
        "from": "sunset-warm",
        "category": "classic",
        "tags": ["warm", "copper", "charcoal"],
        "font": "https://fonts.googleapis.com/css2?family=Source+Sans+3:wght@400;600;700&display=swap",
        "font_family": "'Source Sans 3', sans-serif",
        "uiProfile": "polaroid",
        "layoutCss": "/static/theme-layouts/polaroid.css",
        "bg": "#1a1412",
        "card": (48, 36, 30),
        "accent": "#d97757",
        "light_bg": "#f7f1ec",
        "light_text": "#2a1f1a",
    },
    {
        "id": "neon-boulevard",
        "name": "Neon Boulevard",
        "from": "vaporwave-grid",
        "category": "classic",
        "tags": ["neon", "magenta", "cyan"],
        "font": "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;600;700&display=swap",
        "font_family": "'Space Grotesk', sans-serif",
        "uiProfile": "vaporwave",
        "layoutCss": "/static/theme-layouts/vaporwave.css",
        "bg": "#12101a",
        "card": (36, 28, 52),
        "accent": "#ff4ecd",
        "light_bg": "#f4f0fa",
        "light_text": "#2a2040",
    },
    {
        "id": "kelp-abyss",
        "name": "Kelp Abyss",
        "from": "ocean-depths",
        "category": "classic",
        "tags": ["teal", "green", "depth"],
        "font": "https://fonts.googleapis.com/css2?family=Nunito+Sans:wght@400;600;700&display=swap",
        "font_family": "'Nunito Sans', sans-serif",
        "uiProfile": "porthole",
        "layoutCss": "/static/theme-layouts/porthole.css",
        "bg": "#0f1716",
        "card": (20, 48, 44),
        "accent": "#2dd4bf",
        "light_bg": "#eef8f5",
        "light_text": "#14302c",
    },
    {
        "id": "amber-console",
        "name": "Amber Console",
        "from": "terminal-amber",
        "category": "terminal",
        "tags": ["amber", "crt", "console"],
        "font": "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;600&display=swap",
        "font_family": "'IBM Plex Mono', monospace",
        "uiProfile": "crt_amber",
        "layoutCss": "/static/theme-layouts/crt_amber.css",
        "bg": "#16120a",
        "card": (40, 32, 16),
        "accent": "#f0a020",
        "light_bg": "#f7f2e6",
        "light_text": "#2c2410",
    },
    {
        "id": "glacier-mist",
        "name": "Glacier Mist",
        "from": "arctic-frost",
        "category": "classic",
        "tags": ["cool", "frost", "slate"],
        "font": "https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;600;700&display=swap",
        "font_family": "'DM Sans', sans-serif",
        "uiProfile": "crystal",
        "layoutCss": "/static/theme-layouts/crystal.css",
        "bg": "#12161c",
        "card": (32, 40, 52),
        "accent": "#7dd3fc",
        "light_bg": "#f3f7fb",
        "light_text": "#1a2736",
    },
]


def hex_to_rgb(h: str):
    h = h.lstrip("#")
    return tuple(int(h[i : i + 2], 16) for i in (0, 2, 4))


def write_css(theme: dict, dest: Path):
    r, g, b = theme["card"]
    accent = theme["accent"]
    ar, ag, ab = hex_to_rgb(accent)
    tid = theme["id"]
    css = f"""/* AMUD Theme: {tid} */
@import url('_shared.css');
@import url('{theme["font"]}');

:root {{
    --theme-font: {theme["font_family"]};
    --theme-bg-fallback: {theme["bg"]};
    --theme-card-r: {r};
    --theme-card-g: {g};
    --theme-card-b: {b};
    --bg-card: rgba({r}, {g}, {b}, var(--glass-opacity));
    --accent-color: {accent};
    --accent-glow: rgba({ar}, {ag}, {ab}, 0.2);
    --text-primary: #f1f5f9;
    --text-secondary: #94a3b8;
    --text-muted: #64748b;
    --border-card: rgba({ar}, {ag}, {ab}, 0.22);
    --border-hover: {accent};
}}

body {{ font-family: var(--theme-font), system-ui, sans-serif; background-color: var(--theme-bg-fallback); }}
.brand-title, .greeting-title, .clock-time {{ font-family: var(--theme-font), system-ui, sans-serif; }}
.dashboard-container {{ position: relative; z-index: 1; }}
.app-card.glass-panel {{ border: 1px solid var(--border-card); box-shadow: 0 8px 28px rgba(0,0,0,0.28); }}
.clock-widget .clock-time {{ color: var(--accent-color); }}
.category-tabs .filter-tab:hover {{ color: var(--accent-color); }}

[data-theme-id="{tid}"] .topbar-action:hover {{ color: var(--accent-color); }}
[data-theme-id="{tid}"] .app-card:hover {{ border-color: var(--border-hover); }}
[data-theme-id="{tid}"] .metric-value {{ font-weight: 600; }}

:root[data-theme="light"][data-theme-id="{tid}"] {{
    --theme-bg-fallback: {theme["light_bg"]};
    --theme-card-r: {r};
    --theme-card-g: {g};
    --theme-card-b: {b};
    --bg-card: rgba({r}, {g}, {b}, calc(var(--glass-opacity) * 0.55 + 0.22));
    --accent-color: {accent};
    --accent-glow: rgba({ar}, {ag}, {ab}, 0.18);
    --text-primary: {theme["light_text"]};
    --text-secondary: #4b5563;
    --text-muted: #6b7280;
    --border-card: rgba({ar}, {ag}, {ab}, 0.2);
    --border-hover: {accent};
    --success: #16a34a;
    --success-bg: rgba(22, 163, 74, 0.12);
    --danger: #dc2626;
    --danger-bg: rgba(220, 38, 38, 0.1);
    color-scheme: light;
}}

:root[data-theme="light"][data-theme-id="{tid}"] body {{
    background-color: var(--theme-bg-fallback);
    color: var(--text-primary);
}}
"""
    dest.write_text(css, encoding="utf-8")


def ensure_assets(theme: dict, themes_root: Path):
    src = theme["from"]
    tid = theme["id"]
    # CSS
    write_css(theme, themes_root / f"{tid}.css")
    # preview / wallpaper
    for kind in ("previews", "wallpapers"):
        folder = themes_root / kind
        folder.mkdir(parents=True, exist_ok=True)
        src_file = folder / f"{src}.webp"
        dst_file = folder / f"{tid}.webp"
        if src_file.exists() and not dst_file.exists():
            shutil.copy2(src_file, dst_file)
        elif not dst_file.exists():
            # create empty placeholder note via tiny copy of any existing
            any_webp = next(folder.glob("*.webp"), None)
            if any_webp:
                shutil.copy2(any_webp, dst_file)
    # icon pack
    icons_src = themes_root / "icons" / src
    icons_dst = themes_root / "icons" / tid
    if icons_src.exists():
        if icons_dst.exists():
            shutil.rmtree(icons_dst)
        shutil.copytree(icons_src, icons_dst)
        pack = icons_dst / "pack.json"
        if pack.exists():
            data = json.loads(pack.read_text(encoding="utf-8"))
            data["base"] = f"/static/themes/icons/{tid}"
            pack.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def purge_old(themes_root: Path, tid: str):
    for path in [
        themes_root / f"{tid}.css",
        themes_root / "previews" / f"{tid}.webp",
        themes_root / "wallpapers" / f"{tid}.webp",
    ]:
        if path.exists():
            path.unlink()
    icons = themes_root / "icons" / tid
    if icons.exists():
        shutil.rmtree(icons)


def patch_manifest(path: Path):
    data = json.loads(path.read_text(encoding="utf-8"))
    themes = [t for t in data.get("themes", []) if t.get("id") not in REMOVE]
    existing_ids = {t["id"] for t in themes}
    for theme in NEW:
        ensure_assets(theme, path.parent)
        if theme["id"] in existing_ids:
            continue
        themes.append(
            {
                "id": theme["id"],
                "name": theme["name"],
                "file": f"{theme['id']}.css",
                "category": theme["category"],
                "tags": theme["tags"],
                "preview": f"/static/themes/previews/{theme['id']}.webp",
                "wallpaper": f"/static/themes/wallpapers/{theme['id']}.webp",
                "usesWallpaper": True,
                "fontUrl": theme["font"],
                "uiProfile": theme["uiProfile"],
                "iconPack": f"/static/themes/icons/{theme['id']}/pack.json",
                "layoutCss": theme["layoutCss"],
            }
        )
    for tid in REMOVE:
        purge_old(path.parent, tid)
    data["themes"] = themes
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    print(f"Updated {path} -> {len(themes)} themes")


def main():
    patch_manifest(UI / "manifest.json")
    if DOCS.exists():
        patch_manifest(DOCS / "manifest.json")


if __name__ == "__main__":
    main()
