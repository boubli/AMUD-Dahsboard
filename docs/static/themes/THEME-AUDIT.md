# AMUD Theme Compatibility Audit

Audit date: 2026-06-25 (v1.5.6.4 theme system overhaul).

## Variable contract (all bundled themes)

Every theme CSS must set in `:root`:

| Variable | Purpose |
|----------|---------|
| `--theme-bg-fallback` | Solid fallback behind wallpaper |
| `--theme-card-r/g/b` | Card tint RGB (opacity from user **Glass Panel Opacity**) |
| `--theme-overlay-r1/g1/b1` + `r2/g2/b2` | Wallpaper tint (strength from **Wallpaper overlay strength**) |
| `--theme-body-stack` | Procedural-only full background (no wallpaper layer) |

User sliders are enforced in `theme-guards.css` (`backdrop-filter`, `border-radius`, card `background`).

## Light mode contract (v1.7.7+)

Per-theme light palettes live on `:root[data-theme="light"][data-theme-id="…"]` (not one global gray stack in `style.css`).

| Variable | Purpose |
|----------|---------|
| `--theme-bg-fallback` | Light page background |
| `--theme-card-r/g/b` | Glass card tint (with user opacity slider) |
| `--accent-color` / `--accent-glow` | Accent tuned for light backgrounds |
| `--text-primary` / `--text-secondary` / `--text-muted` | Readable text hierarchy |
| `--border-card` / `--border-hover` | Borders |
| `--success` / `--danger` (+ `*-bg`) | Status colors |

Scaffold blocks: `python scripts/generate_theme_light_blocks.py` (mirrors `ui/static/themes` and `docs/static/themes`).

## Procedural themes (`usesWallpaper: false`)

`brutalist-mono`, `blueprint-tech`, `vaporwave-grid`, `terminal-matrix`, `terminal-amber`, `terminal-phosphor`, `taghawsa`

## WebGL themes (`backgroundScript` in manifest)

`taghawsa` — Scheme 5 animated gradient (orange `#F15A22`, teal `#004238`, black). Falls back to procedural CSS on mobile, reduced motion, light mode, and Settings page.

## QA matrix (pass 1 authoring + pass 2 smoke)

| Theme | Glass sliders | Overlay slider | Wallpaper URL | Creative elements | Pass 2 |
|-------|---------------|----------------|---------------|-------------------|--------|
| AMUD Default | pass | pass | pass | accent orbs | pass |
| dracula | pass | pass | pass | purple glow | pass |
| nord | pass | pass | pass | arctic palette | pass |
| cyberpunk-neon | pass | pass | pass | scanlines | pass |
| sunset-warm | pass | pass | pass | warm gradient accents | pass |
| catppuccin-mocha | pass | pass | pass | pastel palette | pass |
| gruvbox-dark | pass | pass | pass | retro earth tones | pass |
| tokyo-night | pass | pass | pass | night city palette | pass |
| one-dark | pass | pass | pass | developer palette | pass |
| everforest | pass | pass | pass | forest greens | pass |
| monokai | pass | pass | pass | classic dev colors | pass |
| rose-pine | pass | pass | pass | elegant rose | pass |
| solarized-dark | pass | pass | pass | scientific palette | pass |
| terminal-phosphor | pass | pass | n/a | CRT scanlines | pass |
| vaporwave-grid | pass | pass | n/a | perspective grid | pass |
| blueprint-tech | pass | pass | n/a | blueprint grid | pass |
| luxury-gold | pass | pass | pass | gold accents | pass |
| holographic-prism | pass | pass | pass | prism shimmer | pass |
| brutalist-mono | pass | pass | n/a | hard shadows | pass |
| aurora-borealis | pass | pass | pass | aurora tones | pass |
| desert-dusk | pass | pass | pass | desert warmth | pass |
| ocean-depths | pass | pass | pass | deep ocean | pass |
| rainforest-mist | pass | pass | pass | misty green | pass |
| volcanic-ember | pass | pass | pass | ember glow | pass |
| terminal-amber | pass | pass | n/a | amber CRT | pass |
| terminal-matrix | pass | pass | n/a | matrix drift | pass |
| sakura-dream | pass | pass | pass | sakura pink | pass |
| lavender-mist | pass | pass | pass | lavender haze | pass |
| rose-gold-blush | pass | pass | pass | rose gold | pass |
| cotton-candy | pass | pass | pass | candy pastels | pass |
| peach-blossom | pass | pass | pass | peach tones | pass |
| nebula-void | pass | pass | pass | space nebula | pass |
| arctic-frost | pass | pass | pass | ice minimal | pass |
| steampunk-brass | pass | pass | pass | brass vintage | pass |
| zen-garden | pass | pass | pass | calm zen | pass |
| retro-arcade | pass | pass | pass | arcade neon | pass |
| midnight-city | pass | pass | pass | urban night | pass |
| taghawsa | pass | pass | n/a | WebGL Scheme 5 + CSS fallback | pass |

## Global fixes (v1.5.6.4)

- `style.css`: canonical `body` stack uses `--wallpaper-overlay-strength` + `--theme-body-stack`
- `theme-guards.css`: glass sliders always win; no app-card hover lift
- Integration cards: 8 metric cells always (CPU/RAM show `—` when agent off)
- Settings: **Wallpaper overlay strength** slider added

## Bundled themes are dark-oriented

Light mode (`data-theme="light"`) in `style.css` clears body backgrounds. Use dark mode with custom themes for best results.
