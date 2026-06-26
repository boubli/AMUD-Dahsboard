#!/usr/bin/env python3
"""Generate unique AMUD theme wallpaper JPEGs (2560x1440) from theme palettes."""

from __future__ import annotations

import hashlib
import random
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
UI_WP = ROOT / "ui" / "static" / "themes" / "wallpapers"
UI_PR = ROOT / "ui" / "static" / "themes" / "previews"
DOCS_WP = ROOT / "docs" / "static" / "themes" / "wallpapers"
DOCS_PR = ROOT / "docs" / "static" / "themes" / "previews"

W, H = 2560, 1440

THEME_PALETTES: dict[str, tuple[tuple[int, int, int], tuple[int, int, int], tuple[int, int, int]]] = {
    "aurora-borealis": ((10, 22, 40), (30, 20, 60), (94, 234, 212)),
    "arctic-frost": ((12, 24, 32), (20, 40, 55), (125, 211, 252)),
    "cotton-candy": ((26, 21, 48), (48, 30, 55), (167, 139, 250)),
    "lavender-mist": ((30, 24, 48), (45, 35, 65), (196, 181, 253)),
    "holographic-prism": ((18, 10, 35), (40, 15, 55), (236, 72, 153)),
    "retro-arcade": ((10, 0, 24), (20, 5, 40), (255, 0, 255)),
    "vaporwave-grid": ((25, 5, 45), (80, 20, 90), (0, 255, 255)),
    "desert-dusk": ((42, 24, 16), (80, 40, 20), (245, 158, 11)),
    "luxury-gold": ((20, 15, 10), (45, 35, 20), (212, 175, 55)),
    "peach-blossom": ((42, 26, 20), (60, 38, 30), (253, 186, 116)),
    "volcanic-ember": ((26, 10, 8), (60, 20, 10), (249, 115, 22)),
    "nebula-void": ((10, 5, 24), (30, 10, 50), (168, 85, 247)),
    "rainforest-mist": ((15, 31, 24), (25, 50, 38), (110, 231, 183)),
    "zen-garden": ((26, 28, 24), (40, 45, 35), (168, 181, 160)),
    "steampunk-brass": ((26, 20, 16), (45, 35, 25), (212, 165, 116)),
    "ocean-depths": ((4, 18, 32), (8, 35, 60), (34, 211, 238)),
    "midnight-city": ((8, 12, 24), (15, 25, 45), (96, 165, 250)),
    "rose-gold-blush": ((40, 24, 24), (55, 35, 30), (232, 180, 184)),
    "sakura-dream": ((42, 21, 32), (60, 30, 50), (249, 168, 212)),
    "terminal-phosphor": ((2, 10, 4), (4, 18, 8), (51, 255, 102)),
    "terminal-amber": ((10, 6, 0), (20, 12, 4), (255, 176, 0)),
    "terminal-matrix": ((0, 10, 2), (0, 20, 6), (0, 255, 65)),
    "blueprint-tech": ((8, 20, 40), (12, 35, 60), (56, 189, 248)),
    "brutalist-mono": ((30, 30, 30), (50, 50, 50), (200, 200, 200)),
}

PREVIEW_ONLY = {
    "terminal-phosphor",
    "terminal-amber",
    "terminal-matrix",
    "blueprint-tech",
    "brutalist-mono",
}


def _rng(theme_id: str) -> random.Random:
    seed = int(hashlib.sha256(theme_id.encode()).hexdigest()[:16], 16)
    return random.Random(seed)


def _lerp(a: int, b: int, t: float) -> int:
    return int(a + (b - a) * t)


def _blend(c1: tuple[int, int, int], c2: tuple[int, int, int], t: float) -> tuple[int, int, int]:
    return (_lerp(c1[0], c2[0], t), _lerp(c1[1], c2[1], t), _lerp(c1[2], c2[2], t))


def generate_wallpaper(theme_id: str) -> Image.Image:
    c0, c1, accent = THEME_PALETTES[theme_id]
    rng = _rng(theme_id)

    strip = Image.new("RGB", (1, H))
    spx = strip.load()
    for y in range(H):
        spx[0, y] = _blend(c0, c1, y / H)

    img = strip.resize((W, H), Image.Resampling.BILINEAR).convert("RGBA")
    overlay = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)

    for i in range(5):
        cx = rng.randint(W // 6, 5 * W // 6)
        cy = rng.randint(H // 6, 5 * H // 6)
        radius = rng.randint(280, 720)
        alpha = rng.randint(35, 95) + i * 8
        draw.ellipse((cx - radius, cy - radius, cx + radius, cy + radius), fill=(*accent, alpha))

    img = Image.alpha_composite(img, overlay).convert("RGB")
    img = img.filter(ImageFilter.GaussianBlur(radius=2.5))

    noise = Image.effect_noise((W, H), rng.uniform(6.0, 14.0)).convert("RGB")
    img = Image.blend(img, noise, alpha=0.04)
    return img


def save_preview(img: Image.Image, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    img.resize((640, 400), Image.Resampling.LANCZOS).save(path, "JPEG", quality=85, optimize=True)


def main() -> None:
    UI_WP.mkdir(parents=True, exist_ok=True)
    UI_PR.mkdir(parents=True, exist_ok=True)
    DOCS_WP.mkdir(parents=True, exist_ok=True)
    DOCS_PR.mkdir(parents=True, exist_ok=True)

    for theme_id in THEME_PALETTES:
        img = generate_wallpaper(theme_id)
        if theme_id not in PREVIEW_ONLY:
            wp_path = UI_WP / f"{theme_id}.jpg"
            img.save(wp_path, "JPEG", quality=88, optimize=True)
            img.save(DOCS_WP / f"{theme_id}.jpg", "JPEG", quality=88, optimize=True)
        save_preview(img, UI_PR / f"{theme_id}.jpg")
        save_preview(img, DOCS_PR / f"{theme_id}.jpg")
        print(f"generated {theme_id}")

    for jpg in UI_WP.glob("*.jpg"):
        if jpg.stem in THEME_PALETTES:
            continue
        save_preview(Image.open(jpg), UI_PR / jpg.name)

    print("done")


if __name__ == "__main__":
    main()
