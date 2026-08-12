# AMUD Theme Compatibility Audit

Audit date: 2026-08-12 (updated for v1.9.2 — 41 themes including Crimson Flare, Glow and Glass, Neumorphism).

## Variable contract (all bundled themes)

Every theme CSS must set in `:root`:

| Variable | Purpose |
|----------|---------|
| `--theme-bg-fallback` | Solid fallback behind wallpaper |
| `--theme-card-r/g/b` | Card tint RGB (opacity from user **Glass Panel Opacity**) |
| `--theme-overlay-r1/g1/b1` + `r2/g2/b2` | Wallpaper tint (strength from **Wallpaper overlay strength**) |
| `--theme-body-stack` | Procedural-only full background (no wallpaper layer) |

User sliders are enforced in `theme-guards.css` (`backdrop-filter`, `border-radius`, card `background`).

## Light mode contract (v1.7.7+ / fixed v1.8.9)

Per-theme light palettes live on `:root[data-theme="light"][data-theme-id="…"]`. **`data-theme-id` must be on `<html>`** (not only `body`) so light blocks match.

| Variable | Purpose |
|----------|---------|
| `--theme-bg-fallback` | Light page background |
| `--theme-card-r/g/b` | Glass card tint (with user opacity slider) |
| `--accent-color` / `--accent-glow` | Accent tuned for light backgrounds |
| `--text-primary` / `--text-secondary` / `--text-muted` | Readable text hierarchy |
| `--border-card` / `--border-hover` | Borders |
| `--success` / `--danger` (+ `*-bg`) | Status colors |

## Procedural / non-wallpaper themes (`usesWallpaper: false`)

`brutalist-mono`, `blueprint-tech`, `terminal-matrix`, `terminal-phosphor`, `taghawsa`

## WebGL themes (`backgroundScript` in manifest)

`taghawsa` — Scheme 5 animated gradient (orange `#F15A22`, teal `#004238`, deep charcoal). Falls back to procedural CSS on mobile, reduced motion, light mode, and Settings page.

## Removed in v1.8.9 (do not document as current)

`sunset-warm`, `vaporwave-grid`, `ocean-depths`, `terminal-amber`, `arctic-frost` → replaced by `ember-hearth`, `neon-boulevard`, `kelp-abyss`, `amber-console`, `glacier-mist`.

## Current theme set (41)

Classic / advanced / nature / terminal / feminine / variety packs, plus Taghawsa, the five v1.8.9 replacements, Glow and Glass, Neumorphism, and Crimson Flare. See `ui/static/themes/manifest.json` (docs mirror: `docs/static/themes/manifest.json`).
