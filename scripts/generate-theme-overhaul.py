#!/usr/bin/env python3
"""Generate manifest v4, distinct theme CSS, CDN icon packs, and themes-assets binaries."""

from __future__ import annotations

import json
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UI_THEMES = ROOT / "ui" / "static" / "themes"
ASSETS = ROOT / "themes-assets"
DOCS_THEMES = ROOT / "docs" / "static" / "themes"

ASSET_BASE = "https://cdn.jsdelivr.net/gh/boubli/AMUD-Dashboard@v1.6.1/themes-assets"

FROZEN = {"default", "luxury-gold"}

ICON_NAMES = [
    "sun", "moon", "cloud", "cpu", "hard-drive", "activity", "wifi", "settings",
    "layout-grid", "search", "plus", "bell", "users", "rss", "server", "plug",
    "home", "shield", "database", "zap", "eye", "palette", "arrow-left",
    "external-link", "power", "play", "pause", "refresh",
]

# Per-theme design spec: colors + ui profile + font
THEME_SPECS: dict[str, dict] = {
    "dracula": {"profile": "gothic", "accent": "#bd93f9", "bg": "#282a36", "card": (68, 71, 90), "font": "Cinzel", "font_q": "Cinzel:wght@400;600"},
    "nord": {"profile": "arctic", "accent": "#88c0d0", "bg": "#2e3440", "card": (46, 52, 64), "font": "Outfit", "font_q": "Outfit:wght@300;500;700"},
    "cyberpunk-neon": {"profile": "cyberpunk", "accent": "#ff2d95", "bg": "#0a0a0f", "card": (18, 18, 26), "font": "Orbitron", "font_q": "Orbitron:wght@500;700"},
    "sunset-warm": {"profile": "polaroid", "accent": "#f59e0b", "bg": "#1c1410", "card": (40, 28, 22), "font": "Lora", "font_q": "Lora:ital,wght@0,400;0,600;1,400"},
    "catppuccin-mocha": {"profile": "cafe", "accent": "#cba6f7", "bg": "#1e1e2e", "card": (49, 50, 68), "font": "Nunito", "font_q": "Nunito:wght@400;600;700"},
    "gruvbox-dark": {"profile": "retro_crt", "accent": "#fabd2f", "bg": "#282828", "card": (60, 56, 54), "font": "IBM Plex Mono", "font_q": "IBM+Plex+Mono:wght@400;600"},
    "tokyo-night": {"profile": "rain_city", "accent": "#7aa2f7", "bg": "#1a1b26", "card": (36, 40, 59), "font": "Inter", "font_q": "Inter:wght@400;600"},
    "one-dark": {"profile": "ide", "accent": "#61afef", "bg": "#282c34", "card": (40, 44, 52), "font": "Source Code Pro", "font_q": "Source+Code+Pro:wght@400;600"},
    "everforest": {"profile": "organic", "accent": "#a7c080", "bg": "#2d353b", "card": (52, 63, 58), "font": "Merriweather", "font_q": "Merriweather:wght@400;700"},
    "monokai": {"profile": "neon_dev", "accent": "#a6e22e", "bg": "#272822", "card": (46, 45, 40), "font": "Fira Code", "font_q": "Fira+Code:wght@400;600"},
    "rose-pine": {"profile": "editorial", "accent": "#ebbcba", "bg": "#191724", "card": (36, 32, 48), "font": "Playfair Display", "font_q": "Playfair+Display:wght@400;600"},
    "solarized-dark": {"profile": "clinical", "accent": "#268bd2", "bg": "#002b36", "card": (7, 54, 66), "font": "Roboto", "font_q": "Roboto:wght@400;500"},
    "terminal-phosphor": {"profile": "crt_green", "accent": "#33ff66", "bg": "#020c04", "card": (4, 18, 8), "font": "Share Tech Mono", "font_q": "Share+Tech+Mono", "procedural": True},
    "vaporwave-grid": {"profile": "vaporwave", "accent": "#ff71ce", "bg": "#1a0a2e", "card": (26, 10, 46), "font": "Pacifico", "font_q": "Pacifico", "procedural": True},
    "blueprint-tech": {"profile": "blueprint", "accent": "#60a5fa", "bg": "#0c1929", "card": (12, 36, 64), "font": "Roboto Mono", "font_q": "Roboto+Mono:wght@400;500", "procedural": True},
    "holographic-prism": {"profile": "holographic", "accent": "#a78bfa", "bg": "#0f0a1a", "card": (30, 20, 50), "font": "Quicksand", "font_q": "Quicksand:wght@400;600"},
    "brutalist-mono": {"profile": "brutalist", "accent": "#dc2626", "bg": "#e8e8e8", "card": (255, 255, 255), "font": "Archivo Black", "font_q": "Archivo+Black", "procedural": True, "light_ui": True},
    "aurora-borealis": {"profile": "aurora", "accent": "#2dd4bf", "bg": "#041016", "card": (8, 32, 40), "font": "Exo 2", "font_q": "Exo+2:wght@400;600"},
    "desert-dusk": {"profile": "sandstone", "accent": "#d97706", "bg": "#1a120c", "card": (48, 36, 28), "font": "Libre Baskerville", "font_q": "Libre+Baskerville:wght@400;700"},
    "ocean-depths": {"profile": "porthole", "accent": "#22d3ee", "bg": "#031018", "card": (6, 28, 42), "font": "Montserrat", "font_q": "Montserrat:wght@400;600"},
    "rainforest-mist": {"profile": "jungle", "accent": "#4ade80", "bg": "#0a1410", "card": (16, 36, 28), "font": "Cormorant Garamond", "font_q": "Cormorant+Garamond:wght@400;600"},
    "volcanic-ember": {"profile": "volcanic", "accent": "#f97316", "bg": "#120808", "card": (40, 16, 12), "font": "Bebas Neue", "font_q": "Bebas+Neue"},
    "terminal-amber": {"profile": "crt_amber", "accent": "#ffb000", "bg": "#120a00", "card": (24, 16, 4), "font": "VT323", "font_q": "VT323", "procedural": True},
    "terminal-matrix": {"profile": "matrix", "accent": "#00ff41", "bg": "#000a04", "card": (4, 16, 8), "font": "Courier Prime", "font_q": "Courier+Prime:wght@400;700", "procedural": True},
    "sakura-dream": {"profile": "sakura", "accent": "#f9a8d4", "bg": "#1a1018", "card": (48, 32, 44), "font": "Zen Maru Gothic", "font_q": "Zen+Maru+Gothic:wght@400;700"},
    "lavender-mist": {"profile": "lavender", "accent": "#c4b5fd", "bg": "#14101c", "card": (40, 32, 56), "font": "Comfortaa", "font_q": "Comfortaa:wght@400;600"},
    "rose-gold-blush": {"profile": "jewelry", "accent": "#e8b4b8", "bg": "#1a1214", "card": (44, 32, 36), "font": "Cormorant", "font_q": "Cormorant:wght@400;600"},
    "cotton-candy": {"profile": "carnival", "accent": "#f472b6", "bg": "#1a1020", "card": (48, 28, 52), "font": "Fredoka", "font_q": "Fredoka:wght@400;600"},
    "peach-blossom": {"profile": "orchard", "accent": "#fdba74", "bg": "#18120e", "card": (52, 36, 28), "font": "DM Serif Display", "font_q": "DM+Serif+Display"},
    "nebula-void": {"profile": "cosmic", "accent": "#a855f7", "bg": "#050510", "card": (20, 12, 40), "font": "Space Grotesk", "font_q": "Space+Grotesk:wght@400;600"},
    "arctic-frost": {"profile": "crystal", "accent": "#bae6fd", "bg": "#0a1218", "card": (220, 235, 245), "font": "Josefin Sans", "font_q": "Josefin+Sans:wght@300;600"},
    "steampunk-brass": {"profile": "steampunk", "accent": "#b45309", "bg": "#1a140c", "card": (48, 40, 28), "font": "Rye", "font_q": "Rye"},
    "zen-garden": {"profile": "zen", "accent": "#78716c", "bg": "#121210", "card": (32, 30, 28), "font": "Noto Serif JP", "font_q": "Noto+Serif+JP:wght@400;600"},
    "retro-arcade": {"profile": "arcade", "accent": "#ef4444", "bg": "#0a0818", "card": (24, 16, 48), "font": "Press Start 2P", "font_q": "Press+Start+2P"},
    "midnight-city": {"profile": "skyline", "accent": "#38bdf8", "bg": "#060810", "card": (12, 18, 32), "font": "Rajdhani", "font_q": "Rajdhani:wght@500;700"},
}

PROFILE_BLOCKS: dict[str, str] = {
    "gothic": """
.brand-title, .greeting-title { font-family: var(--theme-font); letter-spacing: 0.06em; }
.app-card.glass-panel { border-radius: 4px 24px 4px 24px; border: 1px solid rgba(189, 147, 249, 0.35); clip-path: polygon(0 0, 100% 0, 100% calc(100% - 12px), calc(100% - 12px) 100%, 0 100%); }
.topbar { border-bottom: 2px solid rgba(189, 147, 249, 0.2); background: linear-gradient(180deg, rgba(40,42,54,0.95), rgba(30,31,44,0.88)); }
.btn-primary { border-radius: 2px; box-shadow: 0 0 18px var(--accent-glow); text-transform: uppercase; letter-spacing: 0.08em; }
.status-badge.status-online { border: 1px solid var(--accent-color); }
""",
    "arctic": """
.topbar { background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.12); border-radius: 999px; margin: 0.5rem 0; padding: 0.5rem 1rem; backdrop-filter: blur(20px); }
.app-card.glass-panel { border-radius: 20px; border: 1px solid rgba(255,255,255,0.2); box-shadow: inset 0 1px 0 rgba(255,255,255,0.15); }
.greeting-title { font-weight: 300; letter-spacing: 0.12em; text-transform: uppercase; font-size: 0.95rem; }
""",
    "cyberpunk": """
.topbar { clip-path: polygon(0 0, 100% 0, 98% 100%, 2% 100%); border-bottom: 2px solid var(--accent-color); }
.app-card.glass-panel { clip-path: polygon(8px 0, 100% 0, 100% calc(100% - 8px), calc(100% - 8px) 100%, 0 100%, 0 8px); border: 1px solid var(--accent-color); }
.brand-title { text-shadow: 0 0 12px var(--accent-color); }
.btn-primary { clip-path: polygon(6px 0, 100% 0, calc(100% - 6px) 100%, 0 100%); }
body::after { content:""; position:fixed; inset:0; pointer-events:none; background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(255,45,149,0.03) 2px, rgba(255,45,149,0.03) 4px); }
""",
    "polaroid": """
.app-card.glass-panel { border-radius: 4px; padding-bottom: 0.5rem; box-shadow: 0 8px 24px rgba(0,0,0,0.45), 0 0 0 8px rgba(255,255,255,0.06); transform: rotate(-0.4deg); }
.app-card.glass-panel:nth-child(even) { transform: rotate(0.35deg); }
.greeting-widget { border: 6px solid rgba(255,255,255,0.08); }
""",
    "cafe": """
.app-card.glass-panel { border-radius: 28px; border: 2px solid rgba(203,166,247,0.2); }
.topbar-action, .btn-primary { border-radius: 999px; }
.brand-title { font-weight: 700; }
""",
    "retro_crt": """
.app-card.glass-panel { border: 3px solid #3c3836; border-radius: 4px; box-shadow: inset 0 0 30px rgba(0,0,0,0.3); }
.brand-title, .metric-value { font-family: var(--theme-font); }
""",
    "rain_city": """
.app-card.glass-panel { border-radius: 8px; box-shadow: 0 4px 20px rgba(0,0,0,0.5), inset 0 -1px 0 rgba(122,162,247,0.2); }
.topbar { border-bottom: 1px solid rgba(122,162,247,0.15); }
body::before { content:""; position:fixed; inset:0; pointer-events:none; background: linear-gradient(105deg, transparent 40%, rgba(122,162,247,0.04) 50%, transparent 60%); animation: amud-rain-sweep 8s linear infinite; }
@keyframes amud-rain-sweep { 0% { transform: translateX(-30%); } 100% { transform: translateX(30%); } }
""",
    "ide": """
.category-tabs .filter-tab { border-radius: 4px 4px 0 0; font-family: var(--theme-font); font-size: 0.8rem; }
.app-card.glass-panel { border-radius: 6px; border-left: 3px solid var(--accent-color); }
.topbar { background: #21252b; border-bottom: 1px solid #181a1f; }
""",
    "organic": """
.app-card.glass-panel { border-radius: 30% 70% 70% 30% / 30% 30% 70% 70%; border: 1px solid rgba(167,192,128,0.25); }
.bento-grid { gap: 1.25rem; }
""",
    "neon_dev": """
.metric-value { color: var(--accent-color); }
.app-card.glass-panel { border: 1px solid rgba(166,226,46,0.2); box-shadow: 0 0 12px rgba(166,226,46,0.08); }
""",
    "editorial": """
.brand-title, .greeting-title { font-family: var(--theme-font); border-bottom: 1px solid var(--accent-color); padding-bottom: 0.25rem; }
.app-card.glass-panel { border-radius: 2px; border-top: 3px solid var(--accent-color); }
""",
    "clinical": """
.app-card.glass-panel { border-radius: 2px; border: 1px solid rgba(38,139,210,0.25); }
.bento-grid { gap: 0.65rem; }
.metric-block { border: 1px solid rgba(255,255,255,0.06); }
""",
    "crt_green": """
:root { --radius-xl: 4px; --theme-font: 'Share Tech Mono', monospace; }
body, .brand-title, .btn-primary { font-family: var(--theme-font); }
body::after { content:""; position:fixed; inset:0; pointer-events:none; z-index:9999; background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.08) 2px, rgba(0,0,0,0.08) 4px); }
.app-card.glass-panel { border: 1px solid rgba(51,255,102,0.35); box-shadow: 0 0 16px rgba(51,255,102,0.12); }
""",
    "vaporwave": """
:root { --theme-body-stack: linear-gradient(180deg, #1a0a2e 0%, #3d1a5c 35%, #ff6b9d 65%, #ffb347 100%); --theme-procedural: 1; }
.brand-title { font-style: italic; background: linear-gradient(90deg, #ff71ce, #01cdfe); -webkit-background-clip: text; background-clip: text; -webkit-text-fill-color: transparent; }
.app-card.glass-panel { border: 1px solid rgba(255,113,206,0.3); }
""",
    "blueprint": """
:root { --theme-body-stack: linear-gradient(#0c1929, #0c1929), linear-gradient(rgba(96,165,250,0.15) 1px, transparent 1px), linear-gradient(90deg, rgba(96,165,250,0.15) 1px, transparent 1px); --theme-procedural: 1; background-size: auto, 24px 24px, 24px 24px; }
.app-card.glass-panel { border: 1px dashed rgba(96,165,250,0.5); border-radius: 0; }
.topbar { border: 1px dashed rgba(96,165,250,0.4); }
""",
    "holographic": """
.app-card.glass-panel { border: 2px solid transparent; background-clip: padding-box; position: relative; }
.app-card.glass-panel::before { content:""; position:absolute; inset:-2px; border-radius: inherit; background: linear-gradient(135deg, #a78bfa, #f472b6, #22d3ee, #a78bfa); z-index:-1; animation: amud-holo 6s linear infinite; }
@keyframes amud-holo { 0% { filter: hue-rotate(0deg); } 100% { filter: hue-rotate(360deg); } }
""",
    "brutalist": """
:root { --radius-xl: 0; --theme-body-stack: linear-gradient(180deg, #e8e8e8, #d4d4d4); --theme-procedural: 1; color: #0a0a0a; }
.app-card.glass-panel { border: 3px solid #0a0a0a; box-shadow: 6px 6px 0 #0a0a0a; border-radius: 0; }
.btn-primary { border-radius: 0 !important; border: 3px solid #0a0a0a !important; box-shadow: 4px 4px 0 #0a0a0a; }
.topbar { border-bottom: 3px solid #0a0a0a; }
""",
    "aurora": """
body::before { content:""; position:fixed; inset:0; pointer-events:none; background: linear-gradient(120deg, transparent, rgba(45,212,191,0.12), transparent, rgba(99,102,241,0.1), transparent); animation: amud-aurora 12s ease-in-out infinite; }
@keyframes amud-aurora { 0%,100% { opacity:0.6; transform: translateY(0); } 50% { opacity:1; transform: translateY(-3%); } }
.app-card.glass-panel { border-radius: 16px 4px 16px 4px; }
""",
    "sandstone": """
.app-card.glass-panel { border-radius: 6px; border: 2px solid rgba(217,119,6,0.25); background-image: linear-gradient(145deg, rgba(255,255,255,0.03), transparent); }
""",
    "porthole": """
.app-card.glass-panel { border-radius: 50%; aspect-ratio: unset; min-height: var(--bento-row-height); border: 4px solid rgba(34,211,238,0.35); box-shadow: inset 0 0 40px rgba(0,0,0,0.5); }
.app-card-header { justify-content: center; text-align: center; }
""",
    "jungle": """
.app-card.glass-panel { border-radius: 24px 8px 24px 8px; border: 1px solid rgba(74,222,128,0.2); backdrop-filter: blur(24px); }
""",
    "volcanic": """
.app-card.glass-panel { border: 1px solid rgba(249,115,22,0.35); box-shadow: 0 0 20px rgba(249,115,22,0.1); }
body::after { content:""; position:fixed; inset:0; pointer-events:none; background: radial-gradient(ellipse at 50% 100%, rgba(249,115,22,0.08), transparent 60%); }
""",
    "crt_amber": """
:root { --theme-font: 'VT323', monospace; --radius-xl: 2px; }
body, .brand-title { font-family: var(--theme-font); font-size: 1.1rem; }
.app-card.glass-panel { border: 2px solid rgba(255,176,0,0.4); box-shadow: inset 0 0 20px rgba(255,176,0,0.05); }
""",
    "matrix": """
:root { --theme-font: 'Courier Prime', monospace; }
body::before { content:""; position:fixed; inset:0; pointer-events:none; opacity:0.15; background: repeating-linear-gradient(0deg, rgba(0,255,65,0.15) 0, transparent 2px, transparent 4px); animation: amud-matrix-scroll 20s linear infinite; }
@keyframes amud-matrix-scroll { 0% { background-position: 0 0; } 100% { background-position: 0 400px; } }
""",
    "sakura": """
.app-card.glass-panel { border-radius: 20px; border: 1px solid rgba(249,168,212,0.3); background: linear-gradient(160deg, rgba(249,168,212,0.06), transparent); }
""",
    "lavender": """
.app-card.glass-panel { border-radius: 32px; filter: saturate(1.1); box-shadow: 0 8px 32px rgba(196,181,253,0.08); }
""",
    "jewelry": """
.app-card.glass-panel { border: 1px solid rgba(232,180,184,0.4); border-radius: 8px; box-shadow: inset 0 0 0 1px rgba(255,255,255,0.05); }
.brand-title { letter-spacing: 0.15em; text-transform: uppercase; font-size: 0.9rem; }
""",
    "carnival": """
.app-card.glass-panel { border-radius: 24px; border: 3px solid transparent; border-image: linear-gradient(135deg, #f472b6, #a78bfa, #38bdf8) 1; }
""",
    "orchard": """
.app-card.glass-panel { border-radius: 16px 16px 4px 16px; border-top: 4px solid var(--accent-color); }
""",
    "cosmic": """
body::before { content:""; position:fixed; inset:0; pointer-events:none; background: radial-gradient(ellipse at 20% 30%, rgba(168,85,247,0.15), transparent 50%), radial-gradient(ellipse at 80% 70%, rgba(59,130,246,0.1), transparent 45%); }
.app-card.glass-panel { border: 1px solid rgba(168,85,247,0.25); }
""",
    "crystal": """
.app-card.glass-panel { border-radius: 4px; border: 1px solid rgba(186,230,253,0.35); clip-path: polygon(12px 0, 100% 0, 100% calc(100% - 12px), calc(100% - 12px) 100%, 0 100%, 0 12px); }
""",
    "steampunk": """
.app-card.glass-panel { border: 2px solid rgba(180,83,9,0.5); border-radius: 4px; box-shadow: inset 0 0 0 2px rgba(0,0,0,0.2), 4px 4px 0 rgba(0,0,0,0.35); }
.topbar { border-bottom: 3px double rgba(180,83,9,0.4); }
""",
    "zen": """
.app-card.glass-panel { border-radius: 4px; border: none; box-shadow: 0 1px 0 rgba(255,255,255,0.06); }
.bento-grid { gap: 2rem; }
.topbar { background: transparent; border: none; }
""",
    "arcade": """
:root { --radius-xl: 0; --theme-font: 'Press Start 2P', cursive; }
.brand-title { font-size: 0.75rem; line-height: 1.6; }
.app-card.glass-panel { border: 4px solid var(--accent-color); border-radius: 0; image-rendering: pixelated; }
.btn-primary { border-radius: 0; font-family: var(--theme-font); font-size: 0.65rem; }
""",
    "skyline": """
.topbar { background: linear-gradient(180deg, rgba(6,8,16,0.2), rgba(6,8,16,0.9)), linear-gradient(90deg, transparent 48%, rgba(56,189,248,0.15) 50%, transparent 52%); background-size: auto, 40px 100%; }
.app-card.glass-panel { border: 1px solid rgba(56,189,248,0.2); box-shadow: 0 0 8px rgba(56,189,248,0.06); }
""",
}

ICON_SHAPES: dict[str, str] = {
    "sun": '<circle cx="12" cy="12" r="4" fill="{accent}"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" stroke="{accent}"/>',
    "moon": '<path d="M20 14.5A8.5 8.5 0 1 1 9.5 4 7 7 0 0 0 20 14.5z" stroke="{accent}" fill="none"/>',
    "cloud": '<path d="M6 16h11a4 4 0 0 0 0-8 5 5 0 0 0-9.8 1.5A3.5 3.5 0 0 0 6 16z" stroke="{accent}" fill="none"/>',
    "cpu": '<rect x="5" y="5" width="14" height="14" rx="2" stroke="{accent}" fill="none"/><path d="M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3" stroke="{accent}"/>',
    "hard-drive": '<rect x="3" y="6" width="18" height="12" rx="2" stroke="{accent}" fill="none"/><path d="M7 16h.01M11 16h6" stroke="{accent}"/>',
    "activity": '<path d="M3 12h4l2-7 4 14 2-7h6" stroke="{accent}" fill="none"/>',
    "wifi": '<path d="M2 8.5a14 14 0 0 1 20 0M5 12a9 9 0 0 1 14 0M8.5 15.5a4 4 0 0 1 7 0M12 19h.01" stroke="{accent}" fill="none"/>',
    "settings": '<circle cx="12" cy="12" r="3" stroke="{accent}" fill="none"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4" stroke="{accent}"/>',
    "layout-grid": '<rect x="3" y="3" width="8" height="8" rx="1" stroke="{accent}" fill="none"/><rect x="13" y="3" width="8" height="8" rx="1" stroke="{accent}" fill="none"/><rect x="3" y="13" width="8" height="8" rx="1" stroke="{accent}" fill="none"/><rect x="13" y="13" width="8" height="8" rx="1" stroke="{accent}" fill="none"/>',
    "search": '<circle cx="11" cy="11" r="6" stroke="{accent}" fill="none"/><path d="M16 16l5 5" stroke="{accent}"/>',
    "plus": '<path d="M12 5v14M5 12h14" stroke="{accent}"/>',
    "bell": '<path d="M6 17h12M10 20h4M5 9a7 7 0 0 1 14 0c0 5 2 6 2 6H3s2-1 2-6" stroke="{accent}" fill="none"/>',
    "users": '<circle cx="9" cy="8" r="3" stroke="{accent}" fill="none"/><path d="M2 19c0-3 3-5 7-5s7 2 7 5M16 8a3 3 0 1 1 0 6M22 19c0-2.5-2-4.5-5-4.5" stroke="{accent}" fill="none"/>',
    "rss": '<path d="M4 19a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM4 5v6a8 8 0 0 1 8 8h6" stroke="{accent}" fill="none"/><path d="M4 11v2a6 6 0 0 1 6 6h2" stroke="{accent}" fill="none"/>',
    "server": '<rect x="3" y="4" width="18" height="6" rx="1" stroke="{accent}" fill="none"/><rect x="3" y="14" width="18" height="6" rx="1" stroke="{accent}" fill="none"/><path d="M7 7h.01M7 17h.01" stroke="{accent}"/>',
    "plug": '<path d="M8 7V3M16 7V3M8 11h8v8a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2v-8z" stroke="{accent}" fill="none"/>',
    "home": '<path d="M4 10.5 12 4l8 6.5V20a1 1 0 0 1-1 1h-5v-6H10v6H5a1 1 0 0 1-1-1v-9.5z" stroke="{accent}" fill="none"/>',
    "shield": '<path d="M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6l8-3z" stroke="{accent}" fill="none"/>',
    "database": '<ellipse cx="12" cy="6" rx="8" ry="3" stroke="{accent}" fill="none"/><path d="M4 6v6c0 1.7 3.6 3 8 3s8-1.3 8-3V6M4 12v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6" stroke="{accent}" fill="none"/>',
    "zap": '<path d="M13 2 4 14h7l-1 8 10-14h-7l0-6z" stroke="{accent}" fill="none"/>',
    "eye": '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z" stroke="{accent}" fill="none"/><circle cx="12" cy="12" r="2.5" stroke="{accent}" fill="none"/>',
    "palette": '<path d="M12 3a9 9 0 1 0 8 13.5 2.5 2.5 0 0 1-3-3A6 6 0 0 1 19 9a9 9 0 0 0-7-6z" stroke="{accent}" fill="none"/><circle cx="8" cy="10" r="1" fill="{accent}"/><circle cx="12" cy="7" r="1" fill="{accent}"/>',
    "arrow-left": '<path d="M19 12H5M11 6l-6 6 6 6" stroke="{accent}" fill="none"/>',
    "external-link": '<path d="M14 3h7v7M10 14 21 3M21 14v6a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h6" stroke="{accent}" fill="none"/>',
    "power": '<path d="M12 2v8M8.5 4.5a7 7 0 1 0 7 0" stroke="{accent}" fill="none"/>',
    "play": '<polygon points="8,5 19,12 8,19" fill="{accent}"/>',
    "pause": '<rect x="7" y="5" width="4" height="14" fill="{accent}"/><rect x="13" y="5" width="4" height="14" fill="{accent}"/>',
    "refresh": '<path d="M4 4v5h5M20 20v-5h-5M5 19a8 8 0 0 0 13-2M19 5a8 8 0 0 0-13 2" stroke="{accent}" fill="none"/>',
    "default": '<circle cx="12" cy="12" r="8" stroke="{accent}" fill="none"/>',
}


def hex_to_rgb(h: str) -> tuple[int, int, int]:
    h = h.lstrip("#")
    return int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)


def glow(accent: str) -> str:
    r, g, b = hex_to_rgb(accent)
    return f"rgba({r}, {g}, {b}, 0.18)"


def gen_css(theme_id: str, spec: dict) -> str:
    accent = spec["accent"]
    bg = spec["bg"]
    cr, cg, cb = spec["card"]
    font = spec["font"]
    font_q = spec["font_q"]
    profile = spec["profile"]
    light = spec.get("light_ui", False)
    proc = spec.get("procedural", False)

    text_primary = "#0a0a0a" if light else "#f8fafc"
    text_muted = "#525252" if light else "#94a3b8"

    lines = [
        f"/* AMUD Theme: {theme_id} — distinct UI profile: {profile} */",
        f"@import url('https://fonts.googleapis.com/css2?family={font_q}&display=swap');",
        "",
        ":root {",
        f"    --theme-font: '{font}', sans-serif;",
        f"    --theme-bg-fallback: {bg};",
        f"    --theme-card-r: {cr};",
        f"    --theme-card-g: {cg};",
        f"    --theme-card-b: {cb};",
        f"    --bg-card: rgba({cr}, {cg}, {cb}, var(--glass-opacity));",
        f"    --accent-color: {accent};",
        f"    --accent-glow: {glow(accent)};",
        f"    --text-primary: {text_primary};",
        f"    --text-secondary: {text_muted};",
        f"    --text-muted: {text_muted};",
        f"    --border-card: {glow(accent)};",
        f"    --border-hover: {accent};",
    ]
    if proc:
        lines.append("    --theme-procedural: 1;")
    lines.append("}")
    lines.append("")

    block = PROFILE_BLOCKS.get(profile, "")
    lines.append(block)

    # Shared chrome overrides
    lines.extend([
        "",
        "body { font-family: var(--theme-font), system-ui, sans-serif; }",
        ".brand-title, .greeting-title, .clock-time { font-family: var(--theme-font), system-ui, sans-serif; }",
        ".topbar { transition: background 0.2s ease, border-color 0.2s ease; }",
        ".telemetry-bar-container.glass-panel { margin-bottom: 0.75rem; }",
        ".filter-tab.active { border-color: var(--accent-color); color: var(--accent-color); }",
        ".btn-primary { background: var(--accent-color) !important; border-color: var(--accent-color) !important; }",
        ".btn-secondary { border-color: var(--border-card); }",
        ".weather-widget i, .weather-widget svg { color: var(--accent-color); }",
        ".ws-status-pill.ws-connected .ws-status-dot { background: var(--accent-color); }",
        ".status-badge.status-online { background: var(--success-bg); color: var(--success, #10b981); }",
        "::-webkit-scrollbar-thumb { background: var(--accent-color); }",
        "",
        f"/* Theme id hook: [data-theme-id=\"{theme_id}\"] */",
        f'[data-theme-id="{theme_id}"] .dashboard-container {{ position: relative; z-index: 1; }}',
    ])

    if not light:
        lines.append(f':root[data-theme="light"] [data-theme-id="{theme_id}"] .glass-panel {{ filter: brightness(1.05); }}')

    return "\n".join(lines) + "\n"


def gen_icon_svg(name: str, accent: str, profile: str) -> str:
    inner = ICON_SHAPES.get(name, ICON_SHAPES["default"]).format(accent=accent)
    sw = "3" if profile in ("brutalist", "arcade") else "2"
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" '
        f'stroke-width="{sw}" stroke-linecap="round" stroke-linejoin="round">{inner}</svg>'
    )


def load_old_manifest() -> dict:
    with open(UI_THEMES / "manifest.json", encoding="utf-8") as f:
        return json.load(f)


def build_manifest_v4(old: dict) -> dict:
    themes_out = []
    for t in old["themes"]:
        tid = t["id"]
        entry = dict(t)
        if tid == "default":
            entry["preview"] = "/static/wallpaper.png"
            entry["wallpaper"] = "/static/wallpaper.png"
            entry["usesWallpaper"] = True
            themes_out.append(entry)
            continue
        if tid == "luxury-gold":
            # keep local paths for frozen theme
            themes_out.append(entry)
            continue
        spec = THEME_SPECS.get(tid)
        if spec:
            entry["fontUrl"] = f"https://fonts.googleapis.com/css2?family={spec['font_q']}&display=swap"
            entry["uiProfile"] = spec["profile"]
            entry["iconPack"] = f"icons/{tid}/pack.json"
        entry["preview"] = f"previews/{tid}.jpg"
        if t.get("usesWallpaper", True) and tid in THEME_SPECS:
            entry["wallpaper"] = f"wallpapers/{tid}.jpg"
        elif not t.get("usesWallpaper", True):
            entry["wallpaper"] = ""
            entry["usesWallpaper"] = False
        themes_out.append(entry)

    return {
        "version": 4,
        "assetBase": ASSET_BASE,
        "description": old.get("description", "") + " Large assets (icons, wallpapers) load from GitHub CDN.",
        "categories": old["categories"],
        "themes": themes_out,
    }


def copy_assets(old: dict) -> None:
    src_dirs_wp = [UI_THEMES / "wallpapers", ROOT / "docs" / "static" / "themes" / "wallpapers"]
    src_dirs_pr = [UI_THEMES / "previews", ROOT / "docs" / "static" / "themes" / "previews"]
    dst_wp = ASSETS / "wallpapers"
    dst_pr = ASSETS / "previews"
    dst_wp.mkdir(parents=True, exist_ok=True)
    dst_pr.mkdir(parents=True, exist_ok=True)

    for t in old["themes"]:
        tid = t["id"]
        if tid in FROZEN:
            continue
        wp = None
        pr = None
        for d in src_dirs_wp:
            p = d / f"{tid}.jpg"
            if p.exists():
                wp = p
                break
        for d in src_dirs_pr:
            p = d / f"{tid}.jpg"
            if p.exists():
                pr = p
                break
        if wp:
            shutil.copy2(wp, dst_wp / f"{tid}.jpg")
        if pr:
            shutil.copy2(pr, dst_pr / f"{tid}.jpg")


def gen_icon_packs() -> None:
    icons_root = ASSETS / "icons"
    icons_root.mkdir(parents=True, exist_ok=True)
    for tid, spec in THEME_SPECS.items():
        pack_dir = icons_root / tid
        pack_dir.mkdir(parents=True, exist_ok=True)
        icons_map = {}
        accent = spec["accent"]
        profile = spec["profile"]
        for name in ICON_NAMES:
            fname = f"{name}.svg"
            svg = gen_icon_svg(name, accent, profile)
            (pack_dir / fname).write_text(svg, encoding="utf-8")
            icons_map[name] = fname
        pack = {"version": 1, "base": f"icons/{tid}", "icons": icons_map}
        (pack_dir / "pack.json").write_text(json.dumps(pack, indent=2), encoding="utf-8")


def write_css_files() -> None:
    for tid, spec in THEME_SPECS.items():
        css = gen_css(tid, spec)
        (UI_THEMES / f"{tid}.css").write_text(css, encoding="utf-8")


def sync_docs() -> None:
    DOCS_THEMES.mkdir(parents=True, exist_ok=True)
    shutil.copy2(UI_THEMES / "manifest.json", DOCS_THEMES / "manifest.json")
    for css in UI_THEMES.glob("*.css"):
        shutil.copy2(css, DOCS_THEMES / css.name)


def main() -> None:
    old = load_old_manifest()
    write_css_files()
    gen_icon_packs()
    copy_assets(old)
    manifest = build_manifest_v4(old)
    (UI_THEMES / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    (ASSETS / "README.md").write_text(
        "# AMUD theme CDN assets\n\nIcons, wallpapers, and previews served via jsDelivr.\n"
        f"Base: `{ASSET_BASE}`\n",
        encoding="utf-8",
    )
    sync_docs()
    print(f"Generated {len(THEME_SPECS)} CSS files, icon packs, manifest v4, themes-assets/")


if __name__ == "__main__":
    main()
