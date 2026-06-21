/**
 * Generates branded 16:9 SVG blog covers from blog-cover-art.json.
 * Run from docs/: node scripts/generate-blog-covers.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.join(__dirname, '..', 'static', 'img', 'blog');
const art = JSON.parse(
  fs.readFileSync(path.join(__dirname, 'blog-cover-art.json'), 'utf8'),
);

const SHELL = (body, label, subtitle) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1280 720" width="1280" height="720">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#0b0e14"/>
      <stop offset="100%" stop-color="#141a24"/>
    </linearGradient>
    <radialGradient id="glow" cx="50%" cy="42%" r="55%">
      <stop offset="0%" stop-color="#ff6b2b" stop-opacity="0.22"/>
      <stop offset="100%" stop-color="#ff6b2b" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="accent" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#ff6b2b"/>
      <stop offset="100%" stop-color="#ff9f5a"/>
    </linearGradient>
  </defs>
  <rect width="1280" height="720" fill="url(#bg)"/>
  <rect width="1280" height="720" fill="url(#glow)"/>
  <rect x="48" y="48" width="1184" height="624" rx="24" fill="none" stroke="rgba(255,255,255,0.06)" stroke-width="2"/>
  <g transform="translate(640 250)">${body}</g>
  <text x="640" y="500" text-anchor="middle" fill="#f4f4f5" font-family="system-ui,Segoe UI,sans-serif" font-size="42" font-weight="700">${label}</text>
  <text x="640" y="548" text-anchor="middle" fill="#a1a1aa" font-family="system-ui,Segoe UI,sans-serif" font-size="22">${subtitle}</text>
  <text x="640" y="640" text-anchor="middle" fill="#ff6b2b" font-family="system-ui,Segoe UI,sans-serif" font-size="16" font-weight="600" letter-spacing="0.12em">AMUD DASHBOARD BLOG</text>
</svg>`;

function gridBody(colors) {
  return colors
    .map((color, i) => {
      const col = i % 4;
      const row = Math.floor(i / 4);
      const x = -150 + col * 100;
      const y = -70 + row * 55;
      return `<rect x="${x}" y="${y}" width="85" height="42" rx="8" fill="${color}" stroke="rgba(255,255,255,0.12)" stroke-width="1.5"/>`;
    })
    .join('\n      ');
}

function renderCover(entry) {
  const body = entry.grid ? gridBody(entry.grid) : entry.body;
  return SHELL(body, entry.label, entry.subtitle).trim() + '\n';
}

fs.mkdirSync(OUT, {recursive: true});
for (const [name, entry] of Object.entries(art)) {
  fs.writeFileSync(path.join(OUT, name), renderCover(entry));
}
console.log(`Wrote ${Object.keys(art).length} covers to ${OUT}`);
