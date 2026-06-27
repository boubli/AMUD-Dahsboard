#!/usr/bin/env python3
"""Per-profile icon path libraries — visually distinct shapes, not recolored duplicates."""

from __future__ import annotations

# Base stroke templates; {accent} placeholder for color.
_ROUNDED: dict[str, str] = {
    "sun": '<circle cx="12" cy="12" r="4.5" fill="{accent}" opacity="0.9"/><path d="M12 3v2.5M12 18.5V21M3 12h2.5M18.5 12H21M5.6 5.6l1.8 1.8M16.6 16.6l1.8 1.8M5.6 18.4l1.8-1.8M16.6 7.4l1.8-1.8" stroke="{accent}" stroke-width="1.5"/>',
    "moon": '<path d="M20 15a7 7 0 1 1-9-9 6 6 0 0 0 9 9z" fill="{accent}" opacity="0.85"/>',
    "settings": '<circle cx="12" cy="12" r="2.8" fill="{accent}"/><path d="M12 4v2M12 18v2M4 12h2M18 12h2" stroke="{accent}"/>',
    "home": '<path d="M5 11.5 12 5l7 6.5V19a1.5 1.5 0 0 1-1.5 1.5H6.5A1.5 1.5 0 0 1 5 19v-7.5z" fill="{accent}" opacity="0.2" stroke="{accent}"/>',
    "search": '<circle cx="10.5" cy="10.5" r="5.5" stroke="{accent}"/><path d="M15 15l4.5 4.5" stroke="{accent}"/>',
    "default": '<circle cx="12" cy="12" r="7" stroke="{accent}" fill="{accent}" opacity="0.15"/>',
}

_ANGULAR: dict[str, str] = {
    "sun": '<polygon points="12,4 14,10 20,10 15,14 17,20 12,16 7,20 9,14 4,10 10,10" fill="{accent}"/>',
    "moon": '<path d="M18 14 14 10 18 6 10 6 6 12 10 18 18 18 14 14z" fill="{accent}"/>',
    "settings": '<rect x="9" y="9" width="6" height="6" fill="{accent}"/><path d="M12 2v4M12 18v4M2 12h4M18 12h4" stroke="{accent}"/>',
    "home": '<polygon points="12,4 20,12 17,12 17,20 7,20 7,12 4,12" fill="none" stroke="{accent}"/>',
    "search": '<rect x="5" y="5" width="10" height="10" stroke="{accent}" fill="none"/><path d="M14 14l5 5" stroke="{accent}"/>',
    "cpu": '<rect x="6" y="6" width="12" height="12" stroke="{accent}" fill="none"/><rect x="9" y="9" width="6" height="6" fill="{accent}"/>',
    "default": '<polygon points="12,3 21,12 12,21 3,12" stroke="{accent}" fill="none"/>',
}

_BLOCK: dict[str, str] = {
    "sun": '<rect x="8" y="8" width="8" height="8" fill="{accent}"/><rect x="11" y="2" width="2" height="3" fill="{accent}"/><rect x="11" y="19" width="2" height="3" fill="{accent}"/>',
    "moon": '<rect x="8" y="6" width="10" height="12" fill="{accent}"/><rect x="6" y="8" width="4" height="8" fill="#000" opacity="0.35"/>',
    "settings": '<rect x="7" y="7" width="10" height="10" fill="none" stroke="{accent}"/><rect x="10" y="10" width="4" height="4" fill="{accent}"/>',
    "home": '<rect x="6" y="10" width="12" height="10" fill="{accent}"/><polygon points="12,4 18,10 6,10" fill="{accent}"/>',
    "play": '<rect x="7" y="6" width="4" height="12" fill="{accent}"/><rect x="13" y="6" width="4" height="12" fill="{accent}"/>',
    "pause": '<rect x="7" y="6" width="4" height="12" fill="{accent}"/><rect x="13" y="6" width="4" height="12" fill="{accent}"/>',
    "default": '<rect x="5" y="5" width="14" height="14" fill="{accent}"/>',
}

_ORNATE: dict[str, str] = {
    "sun": '<circle cx="12" cy="12" r="3.5" fill="{accent}"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.5 4.5l1.5 1.5M18 18l1.5 1.5" stroke="{accent}"/><path d="M6 12c0-3.3 2.7-6 6-6" stroke="{accent}" opacity="0.5"/>',
    "moon": '<path d="M19 13.5A7.5 7.5 0 1 1 10.5 5a6 6 0 0 0 8.5 8.5z" stroke="{accent}" fill="none"/><circle cx="16" cy="8" r="1" fill="{accent}"/>',
    "shield": '<path d="M12 2l9 4v5c0 5.5-4 9.5-9 11-5-1.5-9-5.5-9-11V6l9-4z" stroke="{accent}" fill="{accent}" opacity="0.12"/>',
    "palette": '<path d="M12 2a10 10 0 0 0 0 20c1.5 0 2.5-1.2 2.5-2.5 0-.8-.4-1.5-1-2 2.2-.5 4.5-2.5 4.5-6A10 10 0 0 0 12 2z" stroke="{accent}" fill="none"/>',
    "default": '<path d="M12 3c5 0 9 4 9 9s-4 9-9 9" stroke="{accent}" fill="none"/>',
}

_MINIMAL: dict[str, str] = {
    "sun": '<circle cx="12" cy="12" r="5" stroke="{accent}" fill="none"/>',
    "moon": '<path d="M18 14a6 6 0 0 0-8-8 5 5 0 0 1 8 8z" stroke="{accent}" fill="none"/>',
    "cloud": '<path d="M7 16h10a3 3 0 0 0 0-6 4 4 0 0 0-7.5-1.5A2.5 2.5 0 0 0 7 16z" stroke="{accent}" fill="none"/>',
    "activity": '<path d="M4 14h3l2-8 3 16 2-6h6" stroke="{accent}" fill="none"/>',
    "default": '<circle cx="12" cy="12" r="1.5" fill="{accent}"/>',
}

_TECH: dict[str, str] = {
    "cpu": '<rect x="4" y="4" width="16" height="16" stroke="{accent}" fill="none"/><line x1="4" y1="12" x2="20" y2="12" stroke="{accent}" opacity="0.4"/><line x1="12" y1="4" x2="12" y2="20" stroke="{accent}" opacity="0.4"/>',
    "server": '<rect x="3" y="5" width="18" height="5" stroke="{accent}"/><rect x="3" y="14" width="18" height="5" stroke="{accent}"/><circle cx="7" cy="7.5" r="0.8" fill="{accent}"/>',
    "database": '<ellipse cx="12" cy="7" rx="7" ry="2.5" stroke="{accent}"/><path d="M5 7v8c0 1.4 3.1 2.5 7 2.5s7-1.1 7-2.5V7" stroke="{accent}"/>',
    "wifi": '<path d="M2 10a14 14 0 0 1 20 0M6 13a9 9 0 0 1 12 0M10 16a4 4 0 0 1 4 0" stroke="{accent}" fill="none"/>',
    "default": '<rect x="6" y="6" width="12" height="12" stroke="{accent}" stroke-dasharray="3 2" fill="none"/>',
}

_ORGANIC: dict[str, str] = {
    "sun": '<circle cx="12" cy="12" r="4" fill="{accent}"/><path d="M12 5c2 3 2 5 0 7s-2 4 0 7" stroke="{accent}" fill="none" opacity="0.6"/>',
    "cloud": '<path d="M6 17h11c2 0 3.5-1.5 3.5-3.5S17 10 15 10a4 4 0 0 0-7.8 1.2C5.5 11.5 4 13 4 15s1.5 2 2 2z" fill="{accent}" opacity="0.25" stroke="{accent}"/>',
    "home": '<path d="M4 12c4-6 8-8 8-8s4 2 8 8v7a1 1 0 0 1-1 1h-5v-5h-4v5H5a1 1 0 0 1-1-1v-7z" stroke="{accent}" fill="none"/>',
    "leaf": '<path d="M12 3c-4 6-4 12 0 18 4-6 4-12 0-18z" fill="{accent}" opacity="0.3" stroke="{accent}"/>',
    "default": '<ellipse cx="12" cy="12" rx="8" ry="6" stroke="{accent}" fill="none"/>',
}

_FILLED: dict[str, str] = {
    "sun": '<circle cx="12" cy="12" r="6" fill="{accent}"/>',
    "bell": '<path d="M6 17h12l-2-10a4 4 0 0 0-8 0L6 17z" fill="{accent}"/><rect x="10" y="17" width="4" height="2" fill="{accent}"/>',
    "plus": '<rect x="5" y="11" width="14" height="2" fill="{accent}"/><rect x="11" y="5" width="2" height="14" fill="{accent}"/>',
    "zap": '<polygon points="13,2 5,14 11,14 9,22 19,10 13,10" fill="{accent}"/>',
    "default": '<rect x="4" y="4" width="16" height="16" rx="2" fill="{accent}"/>',
}

# Lucide-like fallbacks merged into each library
_COMMON: dict[str, str] = {
    "moon": '<path d="M20 14.5A8.5 8.5 0 1 1 9.5 4 7 7 0 0 0 20 14.5z" stroke="{accent}" fill="none"/>',
    "cloud": '<path d="M6 16h11a4 4 0 0 0 0-8 5 5 0 0 0-9.8 1.5A3.5 3.5 0 0 0 6 16z" stroke="{accent}" fill="none"/>',
    "cpu": '<rect x="5" y="5" width="14" height="14" rx="2" stroke="{accent}" fill="none"/><path d="M9 2v3M15 2v3M9 19v3M15 19v3" stroke="{accent}"/>',
    "hard-drive": '<rect x="3" y="6" width="18" height="12" rx="2" stroke="{accent}" fill="none"/><path d="M7 16h10" stroke="{accent}"/>',
    "activity": '<path d="M3 12h4l2-7 4 14 2-7h6" stroke="{accent}" fill="none"/>',
    "wifi": '<path d="M2 8.5a14 14 0 0 1 20 0M5 12a9 9 0 0 1 14 0M8.5 15.5a4 4 0 0 1 7 0" stroke="{accent}" fill="none"/>',
    "settings": '<circle cx="12" cy="12" r="3" stroke="{accent}" fill="none"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2" stroke="{accent}"/>',
    "layout-grid": '<rect x="3" y="3" width="8" height="8" rx="1" stroke="{accent}"/><rect x="13" y="3" width="8" height="8" rx="1" stroke="{accent}"/><rect x="3" y="13" width="8" height="8" rx="1" stroke="{accent}"/><rect x="13" y="13" width="8" height="8" rx="1" stroke="{accent}"/>',
    "search": '<circle cx="11" cy="11" r="6" stroke="{accent}" fill="none"/><path d="M16 16l5 5" stroke="{accent}"/>',
    "plus": '<path d="M12 5v14M5 12h14" stroke="{accent}"/>',
    "bell": '<path d="M6 17h12M10 20h4M5 9a7 7 0 0 1 14 0c0 5 2 6 2 6H3s2-1 2-6" stroke="{accent}" fill="none"/>',
    "users": '<circle cx="9" cy="8" r="3" stroke="{accent}" fill="none"/><path d="M2 19c0-3 3-5 7-5s7 2 7 5" stroke="{accent}" fill="none"/>',
    "rss": '<circle cx="6" cy="18" r="2" fill="{accent}"/><path d="M4 12v4a4 4 0 0 0 4 4h4" stroke="{accent}" fill="none"/>',
    "server": '<rect x="3" y="4" width="18" height="6" rx="1" stroke="{accent}"/><rect x="3" y="14" width="18" height="6" rx="1" stroke="{accent}"/>',
    "plug": '<path d="M8 7V3M16 7V3M8 11h8v8a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2v-8z" stroke="{accent}" fill="none"/>',
    "home": '<path d="M4 10.5 12 4l8 6.5V20a1 1 0 0 1-1 1h-5v-6H10v6H5a1 1 0 0 1-1-1z" stroke="{accent}" fill="none"/>',
    "shield": '<path d="M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6l8-3z" stroke="{accent}" fill="none"/>',
    "database": '<ellipse cx="12" cy="6" rx="8" ry="3" stroke="{accent}"/><path d="M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6" stroke="{accent}"/>',
    "zap": '<path d="M13 2 4 14h7l-1 8 10-14h-7z" stroke="{accent}" fill="none"/>',
    "eye": '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z" stroke="{accent}"/><circle cx="12" cy="12" r="2.5" stroke="{accent}"/>',
    "palette": '<path d="M12 3a9 9 0 1 0 8 13.5 2.5 2.5 0 0 1-3-3A6 6 0 0 1 19 9a9 9 0 0 0-7-6z" stroke="{accent}"/>',
    "arrow-left": '<path d="M19 12H5M11 6l-6 6 6 6" stroke="{accent}"/>',
    "external-link": '<path d="M14 3h7v7M10 14 21 3M21 14v6a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h6" stroke="{accent}"/>',
    "power": '<path d="M12 2v8M8.5 4.5a7 7 0 1 0 7 0" stroke="{accent}"/>',
    "play": '<polygon points="8,5 19,12 8,19" fill="{accent}"/>',
    "pause": '<rect x="7" y="5" width="4" height="14" fill="{accent}"/><rect x="13" y="5" width="4" height="14" fill="{accent}"/>',
    "refresh": '<path d="M4 4v5h5M20 20v-5h-5M5 19a8 8 0 0 0 13-2M19 5a8 8 0 0 0-13 2" stroke="{accent}"/>',
    "sun": '<circle cx="12" cy="12" r="4" fill="{accent}"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2" stroke="{accent}"/>',
    "layout-template": '<rect x="3" y="3" width="18" height="8" rx="1" stroke="{accent}"/><rect x="3" y="13" width="8" height="8" rx="1" stroke="{accent}"/><rect x="13" y="13" width="8" height="8" rx="1" stroke="{accent}"/>',
    "cloud-sun": '<circle cx="12" cy="9" r="3" fill="{accent}" opacity="0.35"/><path d="M7 17h11a4 4 0 0 0 0-8 5 5 0 0 0-9.5-2" stroke="{accent}" fill="none"/><path d="M12 3v1M12 21v1M4 12H3M21 12h-1" stroke="{accent}"/>',
    "heart": '<path d="M12 20s-7-4.4-9-8.5C1.5 8 4 5 7.5 5c2 0 3.5 1.5 4.5 3 1-1.5 2.5-3 4.5-3 3.5 0 6 3 4.5 6.5C19 15.6 12 20 12 20z" stroke="{accent}" fill="{accent}" opacity="0.15"/>',
    "tag": '<path d="M12 3 3 12v9h9l9-9-9-9z" stroke="{accent}" fill="none"/><circle cx="8" cy="8" r="1.5" fill="{accent}"/>',
    "shield-check": '<path d="M12 3l8 3v6c0 5.5-3.5 9.5-8 11-4.5-1.5-8-5.5-8-11V6l8-3z" stroke="{accent}" fill="none"/><path d="M9 12l2 2 4-4" stroke="{accent}"/>',
    "scroll-text": '<path d="M8 4h11M8 8h11M8 12h7" stroke="{accent}"/><path d="M4 4v16l3-2 3 2V4H4z" stroke="{accent}" fill="none"/>',
}

_STYLE_LIBS: dict[str, dict[str, str]] = {
    "rounded": {**_COMMON, **_ROUNDED},
    "angular": {**_COMMON, **_ANGULAR},
    "block": {**_COMMON, **_BLOCK},
    "ornate": {**_COMMON, **_ORNATE},
    "minimal": {**_COMMON, **_MINIMAL},
    "tech": {**_COMMON, **_TECH},
    "organic": {**_COMMON, **_ORGANIC},
    "filled": {**_COMMON, **_FILLED},
}

# One unique style per uiProfile (32 themes)
PROFILE_ICON_STYLE: dict[str, str] = {
    "gothic": "ornate",
    "arctic": "rounded",
    "cyberpunk": "angular",
    "polaroid": "minimal",
    "cafe": "rounded",
    "retro_crt": "block",
    "rain_city": "tech",
    "ide": "tech",
    "organic": "organic",
    "neon_dev": "angular",
    "editorial": "ornate",
    "clinical": "minimal",
    "crt_green": "block",
    "vaporwave": "angular",
    "blueprint": "tech",
    "holographic": "rounded",
    "brutalist": "filled",
    "aurora": "organic",
    "sandstone": "minimal",
    "porthole": "tech",
    "jungle": "organic",
    "volcanic": "filled",
    "crt_amber": "block",
    "matrix": "block",
    "sakura": "organic",
    "lavender": "rounded",
    "jewelry": "ornate",
    "carnival": "filled",
    "orchard": "organic",
    "cosmic": "minimal",
    "crystal": "rounded",
    "steampunk": "ornate",
    "zen": "minimal",
    "arcade": "block",
    "skyline": "tech",
}

PROFILE_STROKE: dict[str, tuple[str, str, str]] = {
    "brutalist": ("3", "square", "miter"),
    "arcade": ("3", "square", "miter"),
    "matrix": ("2.5", "square", "miter"),
    "crt_green": ("2", "square", "miter"),
    "crt_amber": ("2", "square", "miter"),
    "minimal": ("1.5", "round", "round"),
    "clinical": ("1.5", "round", "round"),
    "zen": ("1.5", "round", "round"),
}


def icon_inner(profile: str, name: str, accent: str) -> str:
    style = PROFILE_ICON_STYLE.get(profile, "rounded")
    lib = _STYLE_LIBS.get(style, _STYLE_LIBS["rounded"])
    return lib.get(name, lib["default"]).format(accent=accent)


def icon_stroke_attrs(profile: str) -> tuple[str, str, str]:
    return PROFILE_STROKE.get(profile, ("2", "round", "round"))


def build_svg(profile: str, name: str, accent: str) -> str:
    inner = icon_inner(profile, name, accent)
    sw, cap, join = icon_stroke_attrs(profile)
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" '
        f'stroke="{accent}" stroke-width="{sw}" stroke-linecap="{cap}" stroke-linejoin="{join}">'
        f"{inner}</svg>"
    )
