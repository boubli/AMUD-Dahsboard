# Bundled AMUD themes (offline)

These CSS files ship inside `ui.tar.gz` at `/static/themes/` on your AMUD server.

## Manifest v3

`manifest.json` lists all themes with preview thumbnails and matching wallpapers:

```
/static/themes/manifest.json
/static/themes/dracula.css
/static/themes/wallpapers/nord.jpg
/static/themes/previews/nord.jpg
```

## Settings UI

Open **Settings → Appearance → Theme Gallery** to preview themes visually. Click a card to load CSS and matching wallpaper into the live preview, then **Save Changes**.

Keep in sync with `docs/static/themes/` when adding new themes:

```powershell
./scripts/sync-themes.ps1
```

Re-fetch real photos (vendored locally, offline-safe):

```bash
python scripts/fetch-theme-wallpapers.py
```

Legacy gradient generator (deprecated): `scripts/generate-theme-wallpapers.py`
