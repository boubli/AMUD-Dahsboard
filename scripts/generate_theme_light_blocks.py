#!/usr/bin/env python3
"""Scaffold per-theme light-mode CSS blocks from dark :root variables."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
THEME_DIRS = [
    ROOT / "ui" / "static" / "themes",
    ROOT / "docs" / "static" / "themes",
]


def parse_hex(hex_str: str):
    m = re.match(r"^#?([0-9a-f]{6})$", hex_str.strip(), re.I)
    if not m:
        return None
    n = int(m.group(1), 16)
    return (n >> 16) & 255, (n >> 8) & 255, n & 255


def rgb_hex(r, g, b):
    r, g, b = (max(0, min(255, round(v))) for v in (r, g, b))
    return f"#{r:02x}{g:02x}{b:02x}"


def mix(a, b, t):
    return tuple(a[i] + (b[i] - a[i]) * t for i in range(3))


def luminance(rgb):
    def chan(c):
        c = c / 255
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4

    r, g, b = (chan(v) for v in rgb)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def darken(hex_str, amount=0.15):
    c = parse_hex(hex_str)
    if not c:
        return hex_str
    return rgb_hex(*(v * (1 - amount) for v in c))


def extract_var(css: str, name: str) -> str:
    m = re.search(rf"--{re.escape(name)}\s*:\s*([^;]+);", css, re.I)
    return m.group(1).strip() if m else ""


def strip_old_light(css: str) -> str:
    css = re.sub(
        r':root\[data-theme="light"\]\s*\[data-theme-id="[^"]+"\]\s*\.glass-panel\s*\{[^}]*\}\s*',
        "",
        css,
    )
    return re.sub(r"\n*/\* AMUD light mode[\s\S]*?AMUD light mode end \*/\s*", "", css)


def build_light_block(theme_id: str, accent: str, card_rgb, bg_fallback: str) -> str:
    accent_rgb = parse_hex(accent) or (207, 100, 39)
    white = (255, 255, 255)
    bg = mix(accent_rgb, white, 0.94)
    bg_hex = rgb_hex(*bg)
    card = card_rgb if card_rgb else mix(accent_rgb, white, 0.88)
    accent_on_light = darken(accent, 0.22) if luminance(accent_rgb) > 0.55 else accent
    text_base = mix(accent_rgb, (15, 23, 42), 0.82)
    text_primary = rgb_hex(*text_base)
    text_secondary = rgb_hex(text_base[0] + 40, text_base[1] + 40, text_base[2] + 50)
    text_muted = rgb_hex(text_base[0] + 80, text_base[1] + 80, text_base[2] + 90)
    border_card = f"rgba({accent_rgb[0]}, {accent_rgb[1]}, {accent_rgb[2]}, 0.14)"
    accent_glow = f"rgba({accent_rgb[0]}, {accent_rgb[1]}, {accent_rgb[2]}, 0.2)"
    fallback_bg = bg_hex

    return f"""
/* AMUD light mode — {theme_id} */
:root[data-theme="light"][data-theme-id="{theme_id}"] {{
    --theme-bg-fallback: {fallback_bg};
    --theme-card-r: {round(card[0])};
    --theme-card-g: {round(card[1])};
    --theme-card-b: {round(card[2])};
    --bg-card: rgba({round(card[0])}, {round(card[1])}, {round(card[2])}, var(--glass-opacity));
    --accent-color: {accent_on_light};
    --accent-glow: {accent_glow};
    --text-primary: {text_primary};
    --text-secondary: {text_secondary};
    --text-muted: {text_muted};
    --border-card: {border_card};
    --border-hover: {accent_on_light};
    --success: #16a34a;
    --success-bg: rgba(22, 163, 74, 0.12);
    --danger: #dc2626;
    --danger-bg: rgba(220, 38, 38, 0.1);
    color-scheme: light;
}}

:root[data-theme="light"][data-theme-id="{theme_id}"] body {{
    background-color: var(--theme-bg-fallback);
    color: var(--text-primary);
}}
/* AMUD light mode end */
"""


def process_file(path: Path):
    theme_id = path.stem
    if theme_id.startswith("_"):
        return
    css = path.read_text(encoding="utf-8")
    accent = extract_var(css, "accent-color") or "#cf6427"
    cr, cg, cb = (extract_var(css, k) for k in ("theme-card-r", "theme-card-g", "theme-card-b"))
    card_rgb = (int(cr), int(cg), int(cb)) if cr and cg and cb else None
    css = strip_old_light(css).rstrip()
    css += build_light_block(theme_id, accent, card_rgb, extract_var(css, "theme-bg-fallback"))
    css += "\n"
    path.write_text(css, encoding="utf-8")
    print("updated", path.relative_to(ROOT))


def main():
    for theme_dir in THEME_DIRS:
        if not theme_dir.is_dir():
            print("skip", theme_dir)
            continue
        for path in sorted(theme_dir.glob("*.css")):
            process_file(path)


if __name__ == "__main__":
    main()
