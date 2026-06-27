---
sidebar_position: 6
---

# Custom Themes

AMUD supports full visual customization through **Custom CSS injection**. Copy CSS from the gallery and paste it into **Settings → Appearance → Custom CSS**.

:::tip Interactive Gallery
Browse, search, and **preview** all themes on the **[Theme Gallery](/themes)** page. Each card shows a screenshot so you can see how the theme looks before applying it. Click **Copy CSS** or **Download CSS**, then paste into your dashboard settings.
:::

:::tip CDN theme assets (manifest v4)
Theme **CSS** ships in `ui.tar.gz`. **Icons, wallpapers, and gallery previews** load from GitHub via jsDelivr (`manifest.json` → `assetBase`). **AMUD Default** and **Luxury Gold** stay fully offline. Other themes need internet on first load (icons/wallpaper); CSS works offline after save.
:::

:::tip Offline default
**AMUD Default** uses the local `/static/wallpaper.png` and Lucide icons — no CDN required.
:::

:::tip Combining with Built-in Settings
Themes work alongside built-in quick settings (accent color, wallpaper URL, glass blur, grid columns, etc.). If a theme sets `--accent-color` but you also pick a color in the UI, the UI setting wins because it is injected into `:root` after custom CSS. Remove `--accent-color` from the theme CSS to control accent from the UI, or leave Custom CSS empty and use quick controls only. Wallpaper tint and background styling come from theme CSS or your wallpaper URL.
:::

:::info Recovery
If a theme breaks your layout, see [Recovering from Broken Custom CSS](./troubleshooting.md#recovering-from-broken-custom-css).
:::

---

## Available Themes

| Theme | Style |
|-------|-------|
| AMUD Default | Built-in orange glass — leave Custom CSS empty |
| Dracula | Dark purple hacker |
| Nord | Arctic blue, calm |
| Cyberpunk Neon | Neon pink on black + scanlines |
| Sunset Warm | Amber golden-hour |
| Catppuccin Mocha | Soft pastel lavender |
| Gruvbox Dark | Warm retro terminal |
| Tokyo Night | Deep blue city night |
| One Dark | Classic Atom palette |
| Everforest | Muted green forest |
| Monokai | Neon green developer |
| Rose Pine | Elegant rose & pine |
| Solarized Dark | Low-contrast scientific |
| **Terminal Phosphor** ★ | CRT green monospace + scanlines |
| **Vaporwave Grid** ★ | 80s sunset perspective grid |
| **Blueprint Tech** ★ | Engineering schematic cyan |
| **Luxury Gold** ★ | Obsidian + gold serif headers |
| **Holographic Prism** ★ | Animated iridescent borders |
| **Brutalist Mono** ★ | Raw concrete, no blur, bold mono |
| **Nature pack (5)** | Aurora, desert, ocean, rainforest, volcanic |
| **Terminal pack (3)** | Phosphor, amber CRT, matrix green |
| **Feminine pack (5)** | Sakura, lavender, rose gold, cotton candy, peach |
| **Variety pack (6)** | Nebula, arctic frost, steampunk, zen, arcade, midnight city |

★ = distinct UI profiles (layout, typography, icons) — not just color swaps

**CDN assets:** `themes-assets/` in the GitHub repo (icons, wallpapers, previews). Base URL is set in `manifest.json` (`assetBase`).

Preview thumbnails and wallpapers are loaded from jsDelivr when you browse the theme gallery or apply a theme.

**Raw CSS files:** [docs/static/themes/](https://github.com/boubli/AMUD-Dashboard/tree/main/docs/static/themes) (docs site) and **`ui/static/themes/`** (shipped with your dashboard for offline use).

Open the [Theme Gallery](/themes) for search, previews, **Copy CSS**, **Download CSS**, and **Copy wallpaper**.

---

## Creating Your Own Theme

Override CSS variables in **Settings → Custom CSS**:

```css
/* My Custom AMUD Theme */
:root {
    --accent-color: #your-color;
    --accent-glow: rgba(r, g, b, 0.15);
    --bg-card: rgba(r, g, b, 0.70);
    --border-card: rgba(r, g, b, 0.10);
    --border-hover: rgba(r, g, b, 0.25);
    --text-primary: #your-text;
    --text-secondary: #your-secondary;
    --text-muted: #your-muted;
    --success: #your-green;
    --danger: #your-red;
}

body {
    background-color: #your-base-bg;
}
```

### Available CSS Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `--accent-color` | Primary brand/accent color | `#cf6427` |
| `--accent-glow` | Subtle glow tint derived from accent | `rgba(207,100,39,0.15)` |
| `--bg-card` | Glass card background | `rgba(15,20,25,0.45)` |
| `--border-card` | Card border at rest | `rgba(255,255,255,0.08)` |
| `--border-hover` | Card border on hover | `rgba(255,255,255,0.16)` |
| `--text-primary` | Main text color | `#f8fafc` |
| `--text-secondary` | Secondary/label text | `#94a3b8` |
| `--text-muted` | Subtle/hint text | `#64748b` |
| `--success` | Online/healthy status | `#10b981` |
| `--danger` | Error/stopped status | `#ef4444` |
| `--glass-blur-intensity` | Backdrop blur amount | `16px` |
| `--glass-opacity` | Glass panel opacity | `0.45` |
| `--radius-xl` | Large border radius | `16px` |

Advanced themes can also style `.glass-panel`, `.header-title`, `body::before`, and animations — see `terminal-phosphor.css` or `vaporwave-grid.css` for examples.

### Adding a theme to the gallery

1. Add `your-theme.css` to `docs/static/themes/` **and** `ui/static/themes/`
2. Add a preview PNG to `docs/static/themes/assets/AMUD-Theme-Your-Theme.png`
3. Add `docs/src/data/themes/definitions/your-theme.ts` and register it in `definitions/index.ts`
4. Add an entry to `ui/static/themes/manifest.json`

After merging to `main`, the theme appears on GitHub Pages and ships offline with the next AMUD release.
