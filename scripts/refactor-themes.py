#!/usr/bin/env python3
"""Refactor bundled AMUD themes to use CSS variables for glass/wallpaper control."""

from __future__ import annotations

import re
from pathlib import Path

THEMES_DIR = Path(__file__).resolve().parent.parent / "ui" / "static" / "themes"

PROCEDURAL = {
    "brutalist-mono": {
        "theme_bg": "#e8e8e8",
        "theme_body_stack": "linear-gradient(180deg, #e8e8e8 0%, #d4d4d4 100%)",
        "card": (255, 255, 255),
    },
    "blueprint-tech": {
        "theme_bg": "#0a1628",
        "theme_body_stack": (
            "linear-gradient(rgba(30, 144, 255, 0.08) 1px, transparent 1px), "
            "linear-gradient(90deg, rgba(30, 144, 255, 0.08) 1px, transparent 1px), "
            "linear-gradient(180deg, #0a1628 0%, #0d2040 100%)"
        ),
        "card": (10, 22, 40),
    },
    "vaporwave-grid": {
        "theme_bg": "#1a0a2e",
        "theme_body_stack": (
            "linear-gradient(180deg, #1a0a2e 0%, #3d1a5c 35%, #ff6b9d 65%, #ffb347 100%)"
        ),
        "card": (26, 10, 46),
    },
    "terminal-matrix": {
        "theme_bg": "#000a02",
        "theme_body_stack": (
            "linear-gradient(180deg, rgba(0, 10, 2, 0.98) 0%, rgba(0, 20, 6, 0.95) 100%), "
            "linear-gradient(rgba(0, 255, 65, 0.04) 1px, transparent 1px), "
            "linear-gradient(90deg, rgba(0, 255, 65, 0.04) 1px, transparent 1px)"
        ),
        "card": (0, 12, 4),
    },
    "terminal-amber": {
        "theme_bg": "#140a00",
        "theme_body_stack": (
            "linear-gradient(180deg, rgba(20, 10, 0, 0.98) 0%, rgba(30, 18, 4, 0.95) 100%), "
            "linear-gradient(rgba(255, 176, 0, 0.05) 1px, transparent 1px), "
            "linear-gradient(90deg, rgba(255, 176, 0, 0.05) 1px, transparent 1px)"
        ),
        "card": (20, 12, 4),
    },
    "terminal-phosphor": {
        "theme_bg": "#020c04",
        "theme_body_stack": (
            "linear-gradient(180deg, rgba(2, 12, 4, 0.98) 0%, rgba(4, 24, 10, 0.95) 100%), "
            "linear-gradient(rgba(57, 255, 136, 0.04) 1px, transparent 1px), "
            "linear-gradient(90deg, rgba(57, 255, 136, 0.04) 1px, transparent 1px)"
        ),
        "card": (4, 18, 8),
    },
}

RGBA_CARD = re.compile(
    r"--bg-card:\s*rgba\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*[\d.]+\s*\)\s*;",
    re.I,
)
HEX_CARD = re.compile(r"--bg-card:\s*(#[0-9a-fA-F]{3,8})\s*;", re.I)
BG_COLOR = re.compile(r"background-color:\s*(#[0-9a-fA-F]{3,8})\s*;", re.I)
RGBA_IN_GRADIENT = re.compile(
    r"rgba\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*[\d.]+\s*\)", re.I
)
BODY_BLOCK = re.compile(r"\nbody\s*\{[^}]*\}\s*", re.S)
GLASS_HOVER = re.compile(r"\n\.glass-panel:hover\s*\{[^}]*\}\s*", re.S)
APP_CARD_HOVER = re.compile(
    r"\n\.app-card\.glass-panel:hover\s*\{[^}]*\}\s*", re.S
)
GLASS_IMPORTANT = re.compile(
    r"\n\.glass-panel\s*\{[^}]*!important[^}]*\}\s*", re.S
)
ROOT_GLASS_OVERRIDES = re.compile(
    r"\s*--glass-blur-intensity:[^;]+;\s*"
    r"|\s*--glass-opacity:[^;]+;\s*"
    r"|\s*--radius-xl:[^;]+;\s*",
    re.I,
)
WALLPAPER_HIDE = re.compile(
    r"\n\.wallpaper-bg,\s*\n\.wallpaper-overlay\s*\{[^}]*\}\s*", re.S
)


def hex_to_rgb(hex_color: str) -> tuple[int, int, int]:
    h = hex_color.lstrip("#")
    if len(h) == 3:
        h = "".join(c * 2 for c in h)
    return int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)


def theme_vars_block(
    name: str,
    content: str,
    card: tuple[int, int, int],
    bg_fallback: str,
    overlay1: tuple[int, int, int] | None,
    overlay2: tuple[int, int, int] | None,
    body_stack: str | None,
) -> str:
    lines = [
        "    --theme-bg-fallback: {};".format(bg_fallback),
        "    --theme-card-r: {};".format(card[0]),
        "    --theme-card-g: {};".format(card[1]),
        "    --theme-card-b: {};".format(card[2]),
        "    --bg-card: rgba(var(--theme-card-r), var(--theme-card-g), var(--theme-card-b), var(--glass-opacity));",
    ]
    if body_stack:
        lines.append("    --theme-body-stack: {};".format(body_stack))
        lines.append("    --theme-procedural: 1;")
    elif overlay1 and overlay2:
        lines.extend(
            [
                "    --theme-overlay-r1: {};".format(overlay1[0]),
                "    --theme-overlay-g1: {};".format(overlay1[1]),
                "    --theme-overlay-b1: {};".format(overlay1[2]),
                "    --theme-overlay-r2: {};".format(overlay2[0]),
                "    --theme-overlay-g2: {};".format(overlay2[1]),
                "    --theme-overlay-b2: {};".format(overlay2[2]),
            ]
        )
    return "\n".join(lines)


def refactor_file(path: Path) -> None:
    name = path.stem
    content = path.read_text(encoding="utf-8")
    original = content

    body_m = BODY_BLOCK.search(original)
    body_text = body_m.group(0) if body_m else ""

    content = ROOT_GLASS_OVERRIDES.sub("", content)
    content = BODY_BLOCK.sub("\n", content)
    content = GLASS_HOVER.sub("\n", content)
    content = APP_CARD_HOVER.sub("\n", content)
    content = WALLPAPER_HIDE.sub("\n", content)

    rgba_matches = RGBA_IN_GRADIENT.findall(body_text)
    content = RGBA_CARD.sub("", content)
    content = HEX_CARD.sub("", content)

    proc = PROCEDURAL.get(name)
    if proc:
        card = proc["card"]
        bg = proc["theme_bg"]
        stack = proc["theme_body_stack"]
        vars_extra = theme_vars_block(name, content, card, bg, None, None, stack)
    else:
        card_m = RGBA_CARD.search(original) or None
        if card_m:
            card = (int(card_m.group(1)), int(card_m.group(2)), int(card_m.group(3)))
        else:
            hex_m = HEX_CARD.search(original)
            card = hex_to_rgb(hex_m.group(1)) if hex_m else (15, 20, 25)

        bg_m = BG_COLOR.search(body_text) or BG_COLOR.search(original)
        bg = bg_m.group(1) if bg_m else "#0a0b10"

        if len(rgba_matches) >= 2:
            overlay1 = tuple(int(x) for x in rgba_matches[0])
            overlay2 = tuple(int(x) for x in rgba_matches[1])
        elif len(rgba_matches) == 1:
            overlay1 = tuple(int(x) for x in rgba_matches[0])
            overlay2 = card
        else:
            overlay1 = (8, 10, 18)
            overlay2 = card

        vars_extra = theme_vars_block(name, content, card, bg, overlay1, overlay2, None)

    if ":root {" in content:
        content = content.replace(":root {", ":root {\n" + vars_extra, 1)
    else:
        content = ":root {\n" + vars_extra + "\n}\n\n" + content

  # Soften brutalist glass-panel !important block
    if name == "brutalist-mono":
        content = content.replace(
            ".glass-panel {\n    border-radius: 0 !important;\n    border: 3px solid #0a0a0a !important;\n    backdrop-filter: none !important;\n    -webkit-backdrop-filter: none !important;\n    box-shadow: 6px 6px 0 #0a0a0a !important;\n}",
            ".glass-panel {\n    border-radius: var(--radius-xl);\n    border: 3px solid #0a0a0a;\n    backdrop-filter: blur(var(--glass-blur-intensity));\n    -webkit-backdrop-filter: blur(var(--glass-blur-intensity));\n    box-shadow: 6px 6px 0 #0a0a0a;\n}",
        )
        content = content.replace(
            ":root[data-theme=\"dark\"] .glass-panel,\n:root[data-theme=\"light\"] .glass-panel {\n    background: #ffffff !important;\n}",
            ":root[data-theme=\"dark\"] .glass-panel,\n:root[data-theme=\"light\"] .glass-panel {\n    background: rgba(var(--theme-card-r), var(--theme-card-g), var(--theme-card-b), var(--glass-opacity));\n}",
        )

    for term in ("terminal-matrix", "terminal-amber", "terminal-phosphor"):
        if name == term:
            content = re.sub(
                r"\.glass-panel\s*\{[^}]*border-radius:\s*0\s*!important;[^}]*\}",
                ".glass-panel {\n    border-radius: var(--radius-xl);\n    backdrop-filter: blur(var(--glass-blur-intensity));\n    -webkit-backdrop-filter: blur(var(--glass-blur-intensity));\n    box-shadow: 0 0 16px rgba(0, 255, 65, 0.12), inset 0 0 20px rgba(0, 255, 65, 0.02);\n}",
                content,
                count=1,
            )

    if name == "blueprint-tech":
        content = re.sub(
            r"\.glass-panel\s*\{[^}]*backdrop-filter:\s*blur\(4px\)\s*!important;[^}]*\}",
            ".glass-panel {\n    backdrop-filter: blur(var(--glass-blur-intensity));\n    -webkit-backdrop-filter: blur(var(--glass-blur-intensity));\n    border: 1px solid rgba(30, 144, 255, 0.25);\n}",
            content,
            count=1,
        )

    # Tie decorative pseudo-element opacity to user overlay strength
    content = re.sub(
        r"(body::before[^}]*opacity:\s*)([\d.]+)(\s*;)",
        r"\1calc(var(--wallpaper-overlay-strength) * 0.6)\3",
        content,
    )
    content = re.sub(
        r"(body::after[^}]*opacity:\s*)([\d.]+)(\s*;)",
        r"\1calc(var(--wallpaper-overlay-strength) * 0.5)\3",
        content,
    )

    path.write_text(content, encoding="utf-8")
    print("refactored", path.name)


def main() -> None:
    for css in sorted(THEMES_DIR.glob("*.css")):
        refactor_file(css)


if __name__ == "__main__":
    main()
