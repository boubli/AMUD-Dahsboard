/**
 * Generates branded 16:9 SVG blog covers (original artwork, no stock photos).
 * Run from docs/: node scripts/generate-blog-covers.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.join(__dirname, '..', 'static', 'img', 'blog');

function cover({label, subtitle, body}) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1280 720" width="1280" height="720">
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
}

const covers = {
  'proxmox.svg': cover({
    label: 'Proxmox Install',
    subtitle: 'One-command LXC deployment',
    body: `<rect x="-90" y="-70" width="180" height="140" rx="12" fill="#1f2937" stroke="#ff6b2b" stroke-width="3"/>
      <rect x="-70" y="-45" width="140" height="18" rx="4" fill="#374151"/>
      <rect x="-70" y="-18" width="100" height="10" rx="3" fill="#4b5563"/>
      <rect x="-70" y="0" width="120" height="10" rx="3" fill="#4b5563"/>
      <rect x="-70" y="18" width="80" height="10" rx="3" fill="#4b5563"/>
      <path d="M55 -55 L95 -55 L95 55 L55 55 Z" fill="url(#accent)" opacity="0.9"/>
      <text x="75" y="8" text-anchor="middle" fill="#0b0e14" font-size="28" font-weight="800">PVE</text>`,
  }),
  'sqlite.svg': cover({
    label: 'SQLite Config',
    subtitle: 'Zero YAML, one database file',
    body: `<ellipse cx="0" cy="-35" rx="95" ry="28" fill="#1f2937" stroke="#ff6b2b" stroke-width="3"/>
      <path d="M-95 -35 L-95 45 Q-95 75 0 75 Q95 75 95 45 L95 -35" fill="#111827" stroke="#ff6b2b" stroke-width="3"/>
      <ellipse cx="0" cy="5" rx="95" ry="28" fill="none" stroke="rgba(255,107,43,0.35)" stroke-width="2"/>
      <ellipse cx="0" cy="35" rx="95" ry="28" fill="none" stroke="rgba(255,107,43,0.25)" stroke-width="2"/>
      <text x="0" y="12" text-anchor="middle" fill="#ff9f5a" font-size="24" font-weight="700">amud.db</text>`,
  }),
  'docker.svg': cover({
    label: 'Docker Deploy',
    subtitle: '~35MB RAM homelab stack',
    body: `<rect x="-110" y="-55" width="70" height="70" rx="8" fill="#2496ed" opacity="0.85"/>
      <rect x="-30" y="-55" width="70" height="70" rx="8" fill="#2496ed" opacity="0.7"/>
      <rect x="50" y="-55" width="70" height="70" rx="8" fill="#2496ed" opacity="0.55"/>
      <rect x="-70" y="25" width="140" height="36" rx="8" fill="#1f2937" stroke="#ff6b2b" stroke-width="2"/>
      <text x="0" y="49" text-anchor="middle" fill="#f4f4f5" font-size="18" font-weight="600">amud_app + amud_agent</text>`,
  }),
  'plex.svg': cover({
    label: 'Plex & Jellyfin',
    subtitle: 'Live now-playing badges',
    body: `<circle cx="0" cy="0" r="72" fill="#1f2937" stroke="#ff6b2b" stroke-width="3"/>
      <polygon points="-18,-28 34,0 -18,28" fill="url(#accent)"/>
      <rect x="-95" y="88" width="190" height="12" rx="6" fill="#374151"/>
      <rect x="-60" y="108" width="120" height="8" rx="4" fill="#4b5563"/>`,
  }),
  'homeassistant.svg': cover({
    label: 'Home Assistant',
    subtitle: 'Stats on your dashboard cards',
    body: `<path d="M0 -70 L78 10 L48 10 L48 70 L-48 70 L-48 10 L-78 10 Z" fill="#41bdf5" opacity="0.9"/>
      <rect x="-55" y="20" width="110" height="48" rx="8" fill="#1f2937" stroke="#ff6b2b" stroke-width="2"/>
      <text x="0" y="52" text-anchor="middle" fill="#f4f4f5" font-size="20" font-weight="600">23.4°C · 12 lights</text>`,
  }),
  'themes-grid.svg': cover({
    label: '12 Free Themes',
    subtitle: 'Nord, Dracula, Tokyo Night & more',
    body: Array.from({length: 12}, (_, i) => {
      const col = i % 4;
      const row = Math.floor(i / 4);
      const colors = ['#2e3440', '#282a36', '#1a1b26', '#0f1117', '#1e1e2e', '#2d1f1f', '#1a2f1a', '#2b1b3d', '#1f2d3d', '#2a1810', '#1a1a2e', '#0d1117'];
      const x = -150 + col * 100;
      const y = -70 + row * 55;
      return `<rect x="${x}" y="${y}" width="85" height="42" rx="8" fill="${colors[i]}" stroke="rgba(255,255,255,0.12)" stroke-width="1.5"/>`;
    }).join('\n      '),
  }),
  'security.svg': cover({
    label: 'Homelab Security',
    subtitle: 'Auth, TLS, and network boundaries',
    body: `<rect x="-55" y="-20" width="110" height="90" rx="14" fill="none" stroke="url(#accent)" stroke-width="8"/>
      <circle cx="0" cy="-5" r="22" fill="#1f2937" stroke="#ff6b2b" stroke-width="3"/>
      <rect x="-12" y="35" width="24" height="28" rx="6" fill="#1f2937" stroke="#ff6b2b" stroke-width="3"/>`,
  }),
  'nginx.svg': cover({
    label: 'Reverse Proxy',
    subtitle: 'Nginx + WebSocket upgrades',
    body: `<rect x="-120" y="-40" width="90" height="80" rx="10" fill="#1f2937" stroke="#ff6b2b" stroke-width="2"/>
      <text x="-75" y="8" text-anchor="middle" fill="#f4f4f5" font-size="16" font-weight="700">Nginx</text>
      <path d="M-20 0 H40" stroke="#ff6b2b" stroke-width="4" marker-end="url(#arrow)"/>
      <rect x="50" y="-40" width="90" height="80" rx="10" fill="#111827" stroke="#ff9f5a" stroke-width="2"/>
      <text x="95" y="8" text-anchor="middle" fill="#ff9f5a" font-size="14" font-weight="700">AMUD</text>`,
  }),
  'comparison.svg': cover({
    label: 'Dashboard Shootout',
    subtitle: 'AMUD vs Heimdall vs Homepage vs Homarr',
    body: `<rect x="-150" y="-50" width="65" height="100" rx="8" fill="#111827" stroke="#ff6b2b" stroke-width="3"/>
      <text x="-117" y="8" text-anchor="middle" fill="#ff6b2b" font-size="13" font-weight="700">AMUD</text>
      <rect x="-70" y="-40" width="55" height="80" rx="8" fill="#1f2937" stroke="#6b7280" stroke-width="2"/>
      <rect x="0" y="-40" width="55" height="80" rx="8" fill="#1f2937" stroke="#6b7280" stroke-width="2"/>
      <rect x="70" y="-40" width="55" height="80" rx="8" fill="#1f2937" stroke="#6b7280" stroke-width="2"/>`,
  }),
  'linux.svg': cover({
    label: 'Bare Metal Linux',
    subtitle: 'No Docker, no LXC',
    body: `<circle cx="0" cy="0" r="70" fill="#1f2937" stroke="#ff6b2b" stroke-width="3"/>
      <path d="M-8 -35 C-35 -10 -35 25 -5 45 C15 58 35 45 35 20 C35 -5 15 -30 -8 -35 Z" fill="#f4f4f5"/>
      <circle cx="-18" cy="-5" r="5" fill="#0b0e14"/>
      <circle cx="12" cy="-5" r="5" fill="#0b0e14"/>`,
  }),
  'backup.svg': cover({
    label: 'Backup amud.db',
    subtitle: 'One file, whole dashboard',
    body: `<rect x="-70" y="-55" width="90" height="110" rx="8" fill="#1f2937" stroke="#ff6b2b" stroke-width="2"/>
      <text x="-25" y="8" text-anchor="middle" fill="#ff9f5a" font-size="14" font-weight="700">.db</text>
      <path d="M45 -20 L75 10 L60 10 L60 55 L30 55 L30 10 L15 10 Z" fill="url(#accent)"/>`,
  }),
  'tablet.svg': cover({
    label: 'Wall Dashboard',
    subtitle: 'Old tablet, guest mode, kiosk browser',
    body: `<rect x="-55" y="-75" width="110" height="150" rx="14" fill="#111827" stroke="#ff6b2b" stroke-width="3"/>
      <rect x="-45" y="-60" width="90" height="115" rx="6" fill="#0b0e14"/>
      <rect x="-30" y="-45" width="60" height="8" rx="3" fill="#ff6b2b" opacity="0.8"/>
      <rect x="-30" y="-28" width="60" height="8" rx="3" fill="#374151"/>
      <rect x="-30" y="-11" width="40" height="8" rx="3" fill="#22c55e" opacity="0.8"/>`,
  }),
  'portainer.svg': cover({
    label: 'Portainer Deploy',
    subtitle: 'Stack deploy without SSH',
    body: `<rect x="-90" y="-60" width="180" height="120" rx="14" fill="#1f2937" stroke="#13bef9" stroke-width="3"/>
      <circle cx="-40" cy="-15" r="18" fill="#13bef9" opacity="0.25"/>
      <circle cx="40" cy="15" r="22" fill="#13bef9" opacity="0.35"/>
      <text x="0" y="10" text-anchor="middle" fill="#f4f4f5" font-size="22" font-weight="700">Stacks</text>`,
  }),
  'rust.svg': cover({
    label: 'Why Rust',
    subtitle: 'Static binaries, low idle RAM',
    body: `<circle cx="0" cy="0" r="72" fill="#1f2937" stroke="#ff6b2b" stroke-width="3"/>
      <path d="M-35 25 L0 -45 L35 25 L20 25 L20 45 L-20 45 L-20 25 Z" fill="url(#accent)"/>
      <circle cx="28" cy="-28" r="14" fill="#0b0e14" stroke="#ff6b2b" stroke-width="3"/>`,
  }),
  'opensource.svg': cover({
    label: 'Contribute',
    subtitle: 'Issues, PRs, and homelab feedback',
    body: `<circle cx="0" cy="0" r="72" fill="#1f2937" stroke="#f4f4f5" stroke-width="3"/>
      <path d="M0 20 C-35 20 -55 0 -55 -20 C-55 -42 -35 -55 0 -55 C35 -55 55 -42 55 -20 C55 0 35 20 0 20 Z" fill="#f4f4f5"/>
      <circle cx="0" cy="-18" r="18" fill="#0b0e14"/>
      <path d="M-55 -8 Q-72 10 -55 28" fill="none" stroke="#f4f4f5" stroke-width="8" stroke-linecap="round"/>`,
  }),
  'encryption.svg': cover({
    label: 'Encrypted Secrets',
    subtitle: 'AES-GCM at rest in SQLite',
    body: `<rect x="-60" y="-45" width="120" height="90" rx="12" fill="#111827" stroke="#ff6b2b" stroke-width="3"/>
      <text x="0" y="8" text-anchor="middle" fill="#ff9f5a" font-size="20" font-weight="800">AES</text>
      <circle cx="55" cy="-30" r="20" fill="#1f2937" stroke="#22c55e" stroke-width="3"/>
      <path d="M47 -30 L53 -22 L65 -38" fill="none" stroke="#22c55e" stroke-width="4" stroke-linecap="round"/>`,
  }),
  'lxc.svg': cover({
    label: 'LXC Badges',
    subtitle: 'Start, stop, and status from cards',
    body: `<rect x="-100" y="-45" width="80" height="90" rx="10" fill="#111827" stroke="#22c55e" stroke-width="3"/>
      <text x="-60" y="8" text-anchor="middle" fill="#22c55e" font-size="14" font-weight="700">RUN</text>
      <rect x="20" y="-45" width="80" height="90" rx="10" fill="#111827" stroke="#ef4444" stroke-width="3"/>
      <text x="60" y="8" text-anchor="middle" fill="#ef4444" font-size="14" font-weight="700">STOP</text>`,
  }),
};

fs.mkdirSync(OUT, {recursive: true});
for (const [name, svg] of Object.entries(covers)) {
  fs.writeFileSync(path.join(OUT, name), svg.trim() + '\n');
}
console.log(`Wrote ${Object.keys(covers).length} covers to ${OUT}`);
