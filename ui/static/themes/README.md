# AMUD bundled themes

Theme **CSS**, **icons**, **wallpapers**, and **gallery previews** ship inside `ui.tar.gz` at `/static/themes/` — fully offline after install.

## Layout

```
/static/themes/manifest.json          — manifest v5
/static/themes/_shared.css            — shared chrome rules
/static/themes/{id}.css               — per-theme variables + profile
/static/theme-layouts/{profile}.css   — layout profile (topbar, tabs, greeting)
/static/themes/icons/{id}/pack.json   — custom icon pack (28 SVGs)
/static/themes/wallpapers/{id}.webp   — background wallpaper
/static/themes/previews/{id}.webp     — settings gallery thumbnail
```

**Frozen themes:** `default` (AMUD Default) and `luxury-gold` keep Lucide icons and existing CSS.

## Regenerate

```bash
python scripts/fetch-theme-wallpapers.py   # download JPG sources (once)
python scripts/compress-theme-images.py    # WebP for ui.tar.gz
python scripts/generate-theme-overhaul.py  # CSS, icons, manifest, layouts
python scripts/validate-theme-assets.py
```

Sync docs copy: `bash scripts/sync-themes.sh`
