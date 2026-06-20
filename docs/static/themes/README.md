# AMUD Custom Themes

Dashboard theme CSS and preview screenshots for the [Theme Gallery](https://boubli.github.io/AMUD-Dashboard/themes).

## Structure

```
docs/static/themes/
├── assets/                    # Dashboard preview screenshots (PNG)
├── wallpapers/                # Bundled 2K theme wallpapers (JPG)
├── dracula.css
├── nord.css
└── …
```

## How users apply themes

1. Open the Theme Gallery and preview themes with screenshots
2. Click **Copy CSS** → paste in **Settings → Appearance → Custom CSS**
3. Optional: **Copy wallpaper** → paste in **Settings → Appearance → Wallpaper**

## Add a new theme

1. Add `your-theme.css` in this folder
2. Add dashboard preview PNG to `assets/AMUD-Theme-Your-Theme.png`
3. Add matching 2K wallpaper JPG to `wallpapers/your-theme.jpg`
4. Register in `docs/src/data/themes.ts`

Push to `main` — the gallery deploys automatically via GitHub Actions.
