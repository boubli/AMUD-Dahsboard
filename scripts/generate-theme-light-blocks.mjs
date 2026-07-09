#!/usr/bin/env node
/**
 * Scaffold per-theme light-mode CSS blocks from dark :root variables.
 * Run: node scripts/generate-theme-light-blocks.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const THEME_DIRS = [
  path.join(ROOT, 'ui', 'static', 'themes'),
  path.join(ROOT, 'docs', 'static', 'themes'),
];

function parseHex(hex) {
  const m = hex.trim().match(/^#?([0-9a-f]{6})$/i);
  if (!m) return null;
  const n = parseInt(m[1], 16);
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
}

function rgbToHex(r, g, b) {
  const clamp = (v) => Math.max(0, Math.min(255, Math.round(v)));
  return (
    '#' +
    [clamp(r), clamp(g), clamp(b)]
      .map((v) => v.toString(16).padStart(2, '0'))
      .join('')
  );
}

function mix(a, b, t) {
  return {
    r: a.r + (b.r - a.r) * t,
    g: a.g + (b.g - a.g) * t,
    b: a.b + (b.b - a.b) * t,
  };
}

function luminance({ r, g, b }) {
  const s = [r, g, b].map((v) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * s[0] + 0.7152 * s[1] + 0.0722 * s[2];
}

function darken(hex, amount = 0.15) {
  const c = parseHex(hex);
  if (!c) return hex;
  return rgbToHex(c.r * (1 - amount), c.g * (1 - amount), c.b * (1 - amount));
}

function extractVar(css, name) {
  const re = new RegExp(`--${name}\\s*:\\s*([^;]+);`, 'i');
  const m = css.match(re);
  return m ? m[1].trim() : '';
}

function stripOldLightBlock(css) {
  return css
    .replace(
      /:root\[data-theme="light"\]\s*\[data-theme-id="[^"]+"\]\s*\.glass-panel\s*\{[^}]*\}\s*/g,
      '',
    )
    .replace(/\n*\/\* AMUD light mode[\s\S]*?AMUD light mode end \*\/\s*/g, '');
}

function buildLightBlock(themeId, accent, cardRgb, bgFallback) {
  const accentRgb = parseHex(accent) || { r: 207, g: 100, b: 39 };
  const white = { r: 255, g: 255, b: 255 };
  const bg = mix(accentRgb, white, 0.94);
  const bgHex = rgbToHex(bg.r, bg.g, bg.b);
  const card =
    cardRgb ||
    mix(accentRgb, white, 0.88);
  const accentOnLight =
    luminance(accentRgb) > 0.55 ? darken(accent, 0.22) : accent;
  const textBase = mix(accentRgb, { r: 15, g: 23, b: 42 }, 0.82);
  const textPrimary = rgbToHex(textBase.r, textBase.g, textBase.b);
  const textSecondary = rgbToHex(
    textBase.r + 40,
    textBase.g + 40,
    textBase.b + 50,
  );
  const textMuted = rgbToHex(
    textBase.r + 80,
    textBase.g + 80,
    textBase.b + 90,
  );
  const borderCard = `rgba(${Math.round(accentRgb.r)}, ${Math.round(accentRgb.g)}, ${Math.round(accentRgb.b)}, 0.14)`;
  const accentGlow = `rgba(${Math.round(accentRgb.r)}, ${Math.round(accentRgb.g)}, ${Math.round(accentRgb.b)}, 0.2)`;
  const fallbackBg = bgFallback && parseHex(bgFallback) ? bgHex : bgHex;

  return `
/* AMUD light mode — ${themeId} */
:root[data-theme="light"][data-theme-id="${themeId}"] {
    --theme-bg-fallback: ${fallbackBg};
    --theme-card-r: ${Math.round(card.r)};
    --theme-card-g: ${Math.round(card.g)};
    --theme-card-b: ${Math.round(card.b)};
    --bg-card: rgba(${Math.round(card.r)}, ${Math.round(card.g)}, ${Math.round(card.b)}, var(--glass-opacity));
    --accent-color: ${accentOnLight};
    --accent-glow: ${accentGlow};
    --text-primary: ${textPrimary};
    --text-secondary: ${textSecondary};
    --text-muted: ${textMuted};
    --border-card: ${borderCard};
    --border-hover: ${accentOnLight};
    --success: #16a34a;
    --success-bg: rgba(22, 163, 74, 0.12);
    --danger: #dc2626;
    --danger-bg: rgba(220, 38, 38, 0.1);
    color-scheme: light;
}

:root[data-theme="light"][data-theme-id="${themeId}"] body {
    background-color: var(--theme-bg-fallback);
    color: var(--text-primary);
}
/* AMUD light mode end */
`;
}

function processThemeFile(filePath) {
  const themeId = path.basename(filePath, '.css');
  if (themeId.startsWith('_')) return;
  let css = fs.readFileSync(filePath, 'utf8');
  const accent = extractVar(css, 'accent-color') || '#cf6427';
  const bgFallback = extractVar(css, 'theme-bg-fallback');
  const cr = extractVar(css, 'theme-card-r');
  const cg = extractVar(css, 'theme-card-g');
  const cb = extractVar(css, 'theme-card-b');
  const cardRgb =
    cr && cg && cb
      ? { r: parseInt(cr, 10), g: parseInt(cg, 10), b: parseInt(cb, 10) }
      : null;

  css = stripOldLightBlock(css).trimEnd();
  css += buildLightBlock(themeId, accent, cardRgb, bgFallback);
  css += '\n';
  fs.writeFileSync(filePath, css, 'utf8');
  console.log('updated', path.relative(ROOT, filePath));
}

for (const dir of THEME_DIRS) {
  if (!fs.existsSync(dir)) {
    console.warn('skip missing', dir);
    continue;
  }
  for (const name of fs.readdirSync(dir)) {
    if (!name.endsWith('.css') || name.startsWith('_')) continue;
    processThemeFile(path.join(dir, name));
  }
}

console.log('done');
