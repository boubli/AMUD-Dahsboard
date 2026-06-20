# AMUD Custom Themes

Dashboard theme CSS and preview screenshots for the [Theme Gallery](https://boubli.github.io/AMUD-Dashboard/themes).

## Structure

```
docs/static/themes/
├── assets/                    # Preview screenshots (PNG)
│   ├── AMUD-Theme-Dracula.png
│   ├── AMUD-Theme-Nord.png
│   ├── AMUD-Theme-Neon.png
│   ├── AMUD-Theme-Sunset-Warm.png
│   └── AMUD-Theme-{Name}.png  # add new previews here
├── dracula.css
├── nord.css
└── …                          # one .css file per theme
```

## GitHub Pages URLs

- CSS: `https://boubli.github.io/AMUD-Dashboard/themes/{name}.css`
- Previews: `https://boubli.github.io/AMUD-Dashboard/themes/assets/{name}.png`

## Add a new theme

1. Add `your-theme.css` in this folder
2. Add preview PNG to `assets/AMUD-Theme-Your-Theme.png`
3. Register in `docs/src/data/themes.ts`

Push to `main` — the gallery deploys automatically via GitHub Actions.
