# Bundled AMUD themes (offline CSS + CDN assets)

Theme **CSS** ships inside `ui.tar.gz` at `/static/themes/`.

Large assets (icons, wallpapers, previews) live in **`themes-assets/`** at the repo root and are served via jsDelivr:

```
https://cdn.jsdelivr.net/gh/boubli/AMUD-Dashboard@main/themes-assets
```

## Manifest v4

`manifest.json` includes `assetBase` and per-theme relative paths for `iconPack`, `preview`, and `wallpaper`.

## Regenerate

```bash
python scripts/generate-theme-overhaul.py
python scripts/validate-theme-assets.py
```

Commit `themes-assets/` and push to GitHub so CDN URLs resolve for users.

## Settings UI

Open **Settings → Appearance → Theme Gallery**. Click a card to preview, then **Save Settings**.

**Frozen themes (do not regenerate CSS):** `default`, `luxury-gold`.
