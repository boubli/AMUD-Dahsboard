# AMUD Custom Themes

Dashboard theme CSS and preview screenshots for the [Theme Gallery](https://boubli.github.io/AMUD-Dashboard/themes).

## Structure

```
docs/static/themes/
├── assets/                    # Preview screenshots (PNG) — shown in gallery
│   ├── AMUD-Theme-Dracula.png
│   ├── AMUD-Theme-Nord.png
│   ├── AMUD-Theme-Neon.png
│   ├── AMUD-Theme-Sunset-Warm.png
│   └── AMUD-Theme-{Name}.png  # add new previews here
├── dracula.css
├── nord.css
└── …                          # one .css file per theme
```

## How users apply themes

1. Open the Theme Gallery and preview themes with screenshots
2. Click **Copy CSS**
3. Paste into **Settings → Appearance → Custom CSS** on the dashboard

No URL import — copy and paste only.

## Add a new theme

1. Add `your-theme.css` in this folder
2. Add preview PNG to `assets/AMUD-Theme-Your-Theme.png` so users can see how it looks
3. Register in `docs/src/data/themes.ts`

Push to `main` — the gallery deploys automatically via GitHub Actions.
