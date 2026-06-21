---
sidebar_position: 6
---

# Custom Themes

AMUD supports full visual customization through **Custom CSS injection**. Copy CSS from the gallery and paste it into **Settings → Appearance → Custom CSS**.

:::tip Interactive Gallery
Browse, search, and **preview** all themes on the **[Theme Gallery](/themes)** page. Each card shows a screenshot so you can see how the theme looks before applying it. Click **Copy CSS**, then paste into your dashboard settings.
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
| Cyberpunk Neon | Neon pink on black |
| Sunset Warm | Amber golden-hour |
| Catppuccin Mocha | Soft pastel lavender |
| Gruvbox Dark | Warm retro terminal |
| Tokyo Night | Deep blue city night |
| One Dark | Classic Atom palette |
| Everforest | Muted green forest |
| Monokai | Neon green developer |
| Rose Pine | Elegant rose & pine |
| Solarized Dark | Low-contrast scientific |

Preview screenshots live in `docs/static/themes/assets/` and are shown on the [Theme Gallery](/themes).

Each theme also includes a **bundled 2K wallpaper** in `docs/static/themes/wallpapers/` — stable on GitHub Pages. In the gallery, click **Copy wallpaper** and paste into **Settings → Appearance → Wallpaper**.

Open the [Theme Gallery](/themes) for search, previews, **Copy CSS**, and **Copy wallpaper**.

---

## Theme CSS Reference

Below is the full CSS for each theme (also available as raw files under `docs/static/themes/` in the repo).

### Dracula

A classic dark purple hacker aesthetic inspired by [Dracula](https://draculatheme.com).

```css
/* AMUD Theme: Dracula */
:root {
    --accent-color: #bd93f9;
    --accent-glow: rgba(189, 147, 249, 0.15);
    --bg-card: rgba(68, 71, 90, 0.75);
    --border-card: rgba(189, 147, 249, 0.12);
    --border-hover: rgba(189, 147, 249, 0.30);
    --text-primary: #f8f8f2;
    --text-secondary: #bfbfda;
    --text-muted: #6272a4;
    --success: #50fa7b;
    --success-bg: rgba(80, 250, 123, 0.10);
    --danger: #ff5555;
    --danger-bg: rgba(255, 85, 85, 0.10);
}
body {
    background-color: #282a36;
    background-image: linear-gradient(135deg, rgba(40, 42, 54, 0.95) 0%, rgba(30, 31, 44, 0.90) 100%), var(--brand-bg-image);
}
.glass-panel { box-shadow: 0 4px 20px rgba(0, 0, 0, 0.35); }
.glass-panel:hover { box-shadow: 0 8px 32px rgba(189, 147, 249, 0.12); }
.header-title { text-shadow: 0 0 20px rgba(189, 147, 249, 0.25); }
```

### Nord

Arctic blue palette inspired by [Nord](https://www.nordtheme.com).

```css
/* AMUD Theme: Nord */
:root {
    --accent-color: #88c0d0;
    --accent-glow: rgba(136, 192, 208, 0.12);
    --bg-card: rgba(59, 66, 82, 0.70);
    --border-card: rgba(136, 192, 208, 0.10);
    --border-hover: rgba(136, 192, 208, 0.25);
    --text-primary: #eceff4;
    --text-secondary: #d8dee9;
    --text-muted: #7b88a1;
    --success: #a3be8c;
    --success-bg: rgba(163, 190, 140, 0.10);
    --danger: #bf616a;
    --danger-bg: rgba(191, 97, 106, 0.10);
}
body {
    background-color: #2e3440;
    background-image: linear-gradient(160deg, rgba(46, 52, 64, 0.95) 0%, rgba(59, 66, 82, 0.88) 100%), var(--brand-bg-image);
}
```

### Cyberpunk Neon

Neon pink on deep black with scanline overlay.

```css
/* AMUD Theme: Cyberpunk Neon */
:root {
    --accent-color: #ff2d95;
    --accent-glow: rgba(255, 45, 149, 0.18);
    --bg-card: rgba(18, 18, 26, 0.80);
    --border-card: rgba(255, 45, 149, 0.10);
    --border-hover: rgba(255, 45, 149, 0.40);
    --text-primary: #e0e0ff;
    --text-secondary: #a0a0cc;
    --text-muted: #5a5a80;
    --success: #39ff14;
    --success-bg: rgba(57, 255, 20, 0.08);
    --danger: #ff3131;
    --danger-bg: rgba(255, 49, 49, 0.10);
}
body {
    background-color: #0a0a0f;
    background-image: linear-gradient(135deg, rgba(10, 10, 15, 0.97) 0%, rgba(15, 5, 20, 0.93) 100%), var(--brand-bg-image);
}
```

### Sunset Warm

Warm amber tones for cozy wall displays.

```css
/* AMUD Theme: Sunset Warm */
:root {
    --accent-color: #f59e0b;
    --accent-glow: rgba(245, 158, 11, 0.15);
    --bg-card: rgba(42, 32, 24, 0.75);
    --border-card: rgba(245, 158, 11, 0.10);
    --border-hover: rgba(245, 158, 11, 0.28);
    --text-primary: #fef3c7;
    --text-secondary: #d4b896;
    --text-muted: #8a7560;
    --success: #84cc16;
    --success-bg: rgba(132, 204, 22, 0.10);
    --danger: #f43f5e;
    --danger-bg: rgba(244, 63, 94, 0.10);
}
body {
    background-color: #1a1410;
    background-image: linear-gradient(145deg, rgba(26, 20, 16, 0.95) 0%, rgba(35, 22, 12, 0.90) 100%), var(--brand-bg-image);
}
```

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

### Adding a theme to the gallery

1. Add `your-theme.css` to `docs/static/themes/`
2. Add a preview PNG to `docs/static/themes/assets/AMUD-Theme-Your-Theme.png`
3. Register the theme in `docs/src/data/themes.ts`

After merging to `main`, the theme appears on GitHub Pages automatically.
