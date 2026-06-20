---
sidebar_position: 6
---

# Custom Themes Gallery

AMUD supports full visual customization through **Custom CSS injection**. You can override any default style by pasting CSS into **Settings → Customization → Custom CSS** and clicking **Save**. Changes apply instantly to all users.

Below are 4 ready-to-use themes. Click the **Copy** button on any code block, paste it into the Custom CSS text area, and save.

:::tip Combining with Built-in Settings
These themes work alongside the built-in branding options (accent color, background image, overlay theme, glass blur, etc.). If a theme sets `--accent-color` but you also pick a color in the UI, the UI setting wins because it is injected into `:root` after the custom CSS. To ensure a theme fully applies, either reset the built-in accent/overlay settings to their defaults, or remove the `--accent-color` line from the theme CSS and control it from the UI instead.
:::

:::info Recovery
If a theme breaks your layout, see [Recovering from Broken Custom CSS](./troubleshooting.md#recovering-from-broken-custom-css) for instructions on clearing the CSS via the database.
:::

---

## 1. Dracula

A classic dark purple hacker aesthetic inspired by the popular [Dracula](https://draculatheme.com) color scheme. Deep charcoal backgrounds with soft purple accents and high-contrast pastel text.

| Element | Color |
|---------|-------|
| Background | `#282a36` |
| Cards | `#44475a` |
| Accent | `#bd93f9` (purple) |
| Text | `#f8f8f2` |
| Success | `#50fa7b` |
| Danger | `#ff5555` |

```css
/* ═══════════════════════════════════════════
   AMUD Theme: Dracula
   A dark purple hacker aesthetic
   ═══════════════════════════════════════════ */

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
    background-image:
        linear-gradient(135deg, rgba(40, 42, 54, 0.95) 0%, rgba(30, 31, 44, 0.90) 100%),
        var(--brand-bg-image);
}

.glass-panel {
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.35);
}

.glass-panel:hover {
    box-shadow: 0 8px 32px rgba(189, 147, 249, 0.12);
}

/* Subtle purple glow on the header title */
.header-title {
    text-shadow: 0 0 20px rgba(189, 147, 249, 0.25);
}
```

---

## 2. Nord

A clean, cool Arctic blue palette with soft contrasts inspired by the [Nord](https://www.nordtheme.com) color scheme. Calm, professional, and easy on the eyes during long monitoring sessions.

| Element | Color |
|---------|-------|
| Background | `#2e3440` |
| Cards | `#3b4252` |
| Accent | `#88c0d0` (frost blue) |
| Text | `#eceff4` |
| Success | `#a3be8c` |
| Danger | `#bf616a` |

```css
/* ═══════════════════════════════════════════
   AMUD Theme: Nord
   Clean Arctic blue, calm and professional
   ═══════════════════════════════════════════ */

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
    background-image:
        linear-gradient(160deg, rgba(46, 52, 64, 0.95) 0%, rgba(59, 66, 82, 0.88) 100%),
        var(--brand-bg-image);
}

.glass-panel {
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.20);
}

.glass-panel:hover {
    box-shadow: 0 8px 28px rgba(136, 192, 208, 0.08);
}

/* Frost-blue underline effect on the tagline */
.header-tagline {
    border-bottom: 1px solid rgba(136, 192, 208, 0.15);
    padding-bottom: 0.25rem;
}
```

---

## 3. Cyberpunk Neon

High-contrast neon pink and electric cyan on deep black. An aggressive sci-fi aesthetic with glowing card edges and vivid status colors. For the homelab operator who wants their dashboard to feel like a hacking terminal from a cyberpunk movie.

| Element | Color |
|---------|-------|
| Background | `#0a0a0f` |
| Cards | `#12121a` |
| Accent | `#ff2d95` (hot pink) |
| Text | `#e0e0ff` |
| Success | `#39ff14` (neon green) |
| Danger | `#ff3131` |

```css
/* ═══════════════════════════════════════════
   AMUD Theme: Cyberpunk Neon
   Aggressive neon pink on deep black
   ═══════════════════════════════════════════ */

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
    background-image:
        linear-gradient(135deg, rgba(10, 10, 15, 0.97) 0%, rgba(15, 5, 20, 0.93) 100%),
        var(--brand-bg-image);
}

/* Neon glow on glass panels */
.glass-panel {
    box-shadow:
        0 0 1px rgba(255, 45, 149, 0.30),
        0 4px 20px rgba(0, 0, 0, 0.50);
}

.glass-panel:hover {
    box-shadow:
        0 0 8px rgba(255, 45, 149, 0.25),
        0 0 24px rgba(255, 45, 149, 0.10),
        0 8px 32px rgba(0, 0, 0, 0.45);
    border-color: rgba(255, 45, 149, 0.45);
}

/* Glowing header */
.header-title {
    text-shadow:
        0 0 10px rgba(255, 45, 149, 0.40),
        0 0 30px rgba(255, 45, 149, 0.15);
}

/* Scanline overlay effect (subtle) */
body::after {
    content: "";
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 9999;
    background: repeating-linear-gradient(
        0deg,
        transparent,
        transparent 2px,
        rgba(0, 0, 0, 0.03) 2px,
        rgba(0, 0, 0, 0.03) 4px
    );
}
```

---

## 4. Sunset Warm

Warm earthy tones with amber and orange accents. A cozy, inviting palette that feels like a golden-hour dashboard. Great for wall-mounted displays or anyone tired of cold blue interfaces.

| Element | Color |
|---------|-------|
| Background | `#1a1410` |
| Cards | `#2a2018` |
| Accent | `#f59e0b` (amber) |
| Text | `#fef3c7` |
| Success | `#84cc16` |
| Danger | `#f43f5e` |

```css
/* ═══════════════════════════════════════════
   AMUD Theme: Sunset Warm
   Cozy amber tones with earthy warmth
   ═══════════════════════════════════════════ */

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
    background-image:
        linear-gradient(145deg, rgba(26, 20, 16, 0.95) 0%, rgba(35, 22, 12, 0.90) 100%),
        var(--brand-bg-image);
}

.glass-panel {
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.30);
}

.glass-panel:hover {
    box-shadow: 0 8px 28px rgba(245, 158, 11, 0.10);
}

/* Warm glow on header */
.header-title {
    text-shadow: 0 0 16px rgba(245, 158, 11, 0.20);
}

/* Amber-tinted telemetry bars */
.telemetry-bar-fill {
    background: linear-gradient(90deg, #f59e0b, #f97316) !important;
}
```

---

## Creating Your Own Theme

You can create a custom theme by overriding any of the CSS variables above. Here is a minimal starting template:

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

