# AMUD bundled themes

Theme **CSS**, **icons**, **wallpapers**, **WebGL backgrounds**, and **gallery previews** ship inside `ui.tar.gz` at `/static/themes/` — fully offline after install.

## Layout

```
/static/themes/manifest.json          — manifest v5
/static/themes/_shared.css            — shared chrome rules
/static/themes/{id}.css               — per-theme variables + profile
/static/themes/backgrounds/{id}-bg.js — optional WebGL background (e.g. taghawsa)
/static/vendor/three.min.js           — Three.js (loaded only for WebGL themes)
/static/theme-layouts/{profile}.css   — layout profile (topbar, tabs, greeting)
/static/themes/icons/{id}/pack.json   — custom icon pack (28 SVGs)
/static/themes/wallpapers/{id}.webp   — background wallpaper
/static/themes/previews/{id}.webp     — settings gallery thumbnail
```

**Frozen themes:** `default` (AMUD Default) and `luxury-gold` keep Lucide icons and existing CSS.

Theme assets are **pre-built in each release** and ship in `ui.tar.gz`.
