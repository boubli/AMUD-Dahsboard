# Bundled AMUD themes (offline)

These CSS files ship inside `ui.tar.gz` at `/static/themes/` on your AMUD server.

## Manifest v5

`manifest.json` lists all themes with WebP preview thumbnails and matching wallpapers:

```
/static/themes/manifest.json
/static/themes/dracula.css
/static/themes/wallpapers/nord.webp
/static/themes/previews/nord.webp
```

## Settings UI

Open **Settings → Appearance → Theme Gallery** to preview themes visually. Click a card to load CSS and matching wallpaper into the live preview, then **Save Changes**.

Theme assets are pre-built in each AMUD release. The docs copy under `docs/static/themes/` mirrors `ui/static/themes/` for the online gallery.
