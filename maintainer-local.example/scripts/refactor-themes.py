#!/usr/bin/env python3
"""Refactor bundled AMUD themes to use CSS variables for glass/wallpaper control."""

from __future__ import annotations

import re
from collections.abc import Callable
from pathlib import Path

from amud_paths import repo_root

THEMES_DIR = repo_root() / "ui" / "static" / "themes"

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
HEX_CARD = re.compile(r"--bg-card:\s*(#[0-9a-f]{3,8})\s*;", re.I)
BG_COLOR = re.compile(r"background-color:\s*(#[0-9a-f]{3,8})\s*;", re.I)
RGBA_IN_GRADIENT = re.compile(
    r"rgba\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*[\d.]+\s*\)", re.I
)
ROOT_GLASS_OVERRIDES = re.compile(
    r"\s*--glass-blur-intensity:[^;]+;\s*"
    r"|\s*--glass-opacity:[^;]+;\s*"
    r"|\s*--radius-xl:[^;]+;\s*",
    re.I,
)

TERMINAL_THEMES = frozenset({"terminal-matrix", "terminal-amber", "terminal-phosphor"})

TERMINAL_GLASS_REPLACEMENT = (
    ".glass-panel {\n"
    "    border-radius: var(--radius-xl);\n"
    "    backdrop-filter: blur(var(--glass-blur-intensity));\n"
    "    -webkit-backdrop-filter: blur(var(--glass-blur-intensity));\n"
    "    box-shadow: 0 0 16px rgba(0, 255, 65, 0.12), "
    "inset 0 0 20px rgba(0, 255, 65, 0.02);\n"
    "}"
)

BLUEPRINT_GLASS_REPLACEMENT = (
    ".glass-panel {\n"
    "    backdrop-filter: blur(var(--glass-blur-intensity));\n"
    "    -webkit-backdrop-filter: blur(var(--glass-blur-intensity));\n"
    "    border: 1px solid rgba(30, 144, 255, 0.25);\n"
    "}"
)


def _rule_block_end(content: str, open_brace: int) -> int | None:
    depth = 0
    for i in range(open_brace, len(content)):
        ch = content[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i + 1
    return None


def _rule_block_span(content: str, selector_pos: int) -> tuple[int, int] | None:
    brace = content.find("{", selector_pos)
    if brace == -1:
        return None
    end = _rule_block_end(content, brace)
    if end is None:
        return None
    while end < len(content) and content[end] in " \t\r\n":
        end += 1
    return selector_pos, end


def _remove_rules(
    content: str,
    selector: str,
    *,
    block_match: Callable[[str], bool] | None = None,
) -> str:
    start = 0
    parts: list[str] = []
    while start < len(content):
        pos = content.find(selector, start)
        if pos == -1:
            parts.append(content[start:])
            break
        span = _rule_block_span(content, pos)
        if span is None:
            parts.append(content[start : pos + 1])
            start = pos + 1
            continue
        block = content[span[0] : span[1]]
        if block_match is None or block_match(block):
            parts.append(content[start:span[0]])
            parts.append("\n")
            start = span[1]
        else:
            parts.append(content[start : span[1]])
            start = span[1]
    return "".join(parts)


def _replace_first_glass_panel(
    content: str,
    needle: str,
    replacement: str,
) -> str:
    pos = 0
    while True:
        pos = content.find(".glass-panel", pos)
        if pos == -1:
            return content
        span = _rule_block_span(content, pos)
        if span is None:
            pos += 1
            continue
        block = content[span[0] : span[1]]
        if needle in block:
            return content[: span[0]] + replacement + content[span[1] :]
        pos = span[1]
    return content


def _extract_rule(content: str, selector: str) -> str:
    pos = content.find(selector)
    if pos == -1:
        return ""
    span = _rule_block_span(content, pos)
    if span is None:
        return ""
    return content[span[0] : span[1]]


def hex_to_rgb(hex_color: str) -> tuple[int, int, int]:
    h = hex_color.lstrip("#")
    if len(h) == 3:
        h = "".join(c * 2 for c in h)
    return int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)


def theme_vars_block(
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


def _strip_legacy_blocks(content: str) -> tuple[str, str]:
    body_text = _extract_rule(content, "body") or _extract_rule(content, "\nbody")
    content = ROOT_GLASS_OVERRIDES.sub("", content)
    content = _remove_rules(content, "body")
    content = _remove_rules(content, "\nbody")
    content = _remove_rules(content, ".glass-panel:hover")
    content = _remove_rules(content, ".app-card.glass-panel:hover")
    content = _remove_rules(
        content,
        ".glass-panel",
        block_match=lambda block: "!important" in block,
    )
    content = _remove_rules(content, ".wallpaper-bg")
    return content, body_text


def _card_from_original(original: str) -> tuple[int, int, int]:
    card_m = RGBA_CARD.search(original)
    if card_m:
        return (int(card_m.group(1)), int(card_m.group(2)), int(card_m.group(3)))
    hex_m = HEX_CARD.search(original)
    return hex_to_rgb(hex_m.group(1)) if hex_m else (15, 20, 25)


def _overlays_from_body(
    rgba_matches: list[tuple[str, str, str]],
    card: tuple[int, int, int],
) -> tuple[tuple[int, int, int], tuple[int, int, int]]:
    if len(rgba_matches) >= 2:
        return (
            tuple(int(x) for x in rgba_matches[0]),
            tuple(int(x) for x in rgba_matches[1]),
        )
    if len(rgba_matches) == 1:
        return (tuple(int(x) for x in rgba_matches[0]), card)
    return ((8, 10, 18), card)


def _build_vars_extra(
    name: str,
    original: str,
    body_text: str,
) -> str:
    proc = PROCEDURAL.get(name)
    if proc:
        return theme_vars_block(proc["card"], proc["theme_bg"], None, None, proc["theme_body_stack"])

    card = _card_from_original(original)
    bg_m = BG_COLOR.search(body_text) or BG_COLOR.search(original)
    bg = bg_m.group(1) if bg_m else "#0a0b10"
    rgba_matches = RGBA_IN_GRADIENT.findall(body_text)
    overlay1, overlay2 = _overlays_from_body(rgba_matches, card)
    return theme_vars_block(card, bg, overlay1, overlay2, None)


def _inject_root_vars(content: str, vars_extra: str) -> str:
    if ":root {" in content:
        return content.replace(":root {", ":root {\n" + vars_extra, 1)
    return ":root {\n" + vars_extra + "\n}\n\n" + content


def _fix_brutalist_glass(content: str) -> str:
    content = content.replace(
        ".glass-panel {\n    border-radius: 0 !important;\n    border: 3px solid #0a0a0a !important;\n    backdrop-filter: none !important;\n    -webkit-backdrop-filter: none !important;\n    box-shadow: 6px 6px 0 #0a0a0a !important;\n}",
        ".glass-panel {\n    border-radius: var(--radius-xl);\n    border: 3px solid #0a0a0a;\n    backdrop-filter: blur(var(--glass-blur-intensity));\n    -webkit-backdrop-filter: blur(var(--glass-blur-intensity));\n    box-shadow: 6px 6px 0 #0a0a0a;\n}",
    )
    return content.replace(
        ":root[data-theme=\"dark\"] .glass-panel,\n:root[data-theme=\"light\"] .glass-panel {\n    background: #ffffff !important;\n}",
        ":root[data-theme=\"dark\"] .glass-panel,\n:root[data-theme=\"light\"] .glass-panel {\n    background: rgba(var(--theme-card-r), var(--theme-card-g), var(--theme-card-b), var(--glass-opacity));\n}",
    )


def _fix_theme_glass_panels(name: str, content: str) -> str:
    if name == "brutalist-mono":
        content = _fix_brutalist_glass(content)
    if name in TERMINAL_THEMES:
        content = _replace_first_glass_panel(
            content,
            "border-radius: 0 !important;",
            TERMINAL_GLASS_REPLACEMENT,
        )
    if name == "blueprint-tech":
        content = _replace_first_glass_panel(
            content,
            "backdrop-filter: blur(4px) !important;",
            BLUEPRINT_GLASS_REPLACEMENT,
        )
    return content


def _set_pseudo_opacity(content: str, pseudo: str, factor: str) -> str:
    pos = content.find(pseudo)
    if pos == -1:
        return content
    span = _rule_block_span(content, pos)
    if span is None:
        return content
    block = content[span[0] : span[1]]
    key = "opacity:"
    key_pos = block.find(key)
    if key_pos == -1:
        return content
    value_start = key_pos + len(key)
    while value_start < len(block) and block[value_start] in " \t":
        value_start += 1
    value_end = value_start
    while value_end < len(block) and block[value_end] not in ";":
        value_end += 1
    new_block = (
        block[:value_start]
        + f"calc(var(--wallpaper-overlay-strength) * {factor})"
        + block[value_end:]
    )
    return content[: span[0]] + new_block + content[span[1] :]


def _tie_pseudo_overlay_opacity(content: str) -> str:
    content = _set_pseudo_opacity(content, "body::before", "0.6")
    return _set_pseudo_opacity(content, "body::after", "0.5")


def refactor_file(path: Path) -> None:
    name = path.stem
    original = path.read_text(encoding="utf-8")

    content, body_text = _strip_legacy_blocks(original)
    content = RGBA_CARD.sub("", content)
    content = HEX_CARD.sub("", content)

    vars_extra = _build_vars_extra(name, original, body_text)
    content = _inject_root_vars(content, vars_extra)
    content = _fix_theme_glass_panels(name, content)
    content = _tie_pseudo_overlay_opacity(content)

    path.write_text(content, encoding="utf-8")
    print("refactored", path.name)


def main() -> None:
    for css in sorted(THEMES_DIR.glob("*.css")):
        refactor_file(css)


if __name__ == "__main__":
    main()
