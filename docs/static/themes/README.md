# AMUD Custom Themes

Dashboard theme CSS for the [Theme Gallery](https://boubli.github.io/AMUD-Dashboard/themes) and offline use on the dashboard at `/static/themes/`.

## Structure

```
docs/static/themes/          # Gallery + GitHub Pages
ui/static/themes/            # Shipped with ui.tar.gz (offline)
├── manifest.json              # Bundled theme list (ui only)
├── assets/                    # Preview screenshots (PNG)
├── wallpapers/                # 2K wallpapers (JPG, all wallpaper themes)
├── previews/                  # Grid thumbnails (JPG)
├── dracula.css
├── terminal-phosphor.css      # Advanced layout themes
└── …
```

## How users apply themes

**Offline (no internet):** Settings → Appearance → **Theme Gallery** → click a theme → **Save Changes**.

**Online gallery:** [Theme Gallery](https://boubli.github.io/AMUD-Dashboard/themes) — Copy CSS, Download CSS, or Copy wallpaper.

**37 themes** · manifest v3 · wallpapers in `wallpapers/` · previews in `previews/`

## Add a new theme

1. Add `your-theme.css` to **both** `docs/static/themes/` and `ui/static/themes/`
2. Add preview PNG to `assets/AMUD-Theme-Your-Theme.png`
3. Optional wallpaper JPG in `wallpapers/`
4. Add `docs/src/data/themes/definitions/your-theme.ts` and register in `definitions/index.ts`
5. Add entry to `ui/static/themes/manifest.json`

Push to `main` — gallery deploys via GitHub Actions; themes ship with the next release `ui.tar.gz`.
