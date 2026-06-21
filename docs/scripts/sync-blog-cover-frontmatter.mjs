/**
 * Sync image: frontmatter in all blog posts from blog-covers.ts
 * Run from docs/: node scripts/sync-blog-cover-frontmatter.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const blogDir = path.join(__dirname, '..', 'blog');

const COVERS = {
  'why-i-ditched-heavy-dashboards': 'img/AMUD-Dashboard.png',
  'proxmox-one-command-install': 'img/blog/proxmox.svg',
  'zero-yaml-sqlite': 'img/blog/sqlite.svg',
  'realtime-telemetry-no-shell-scripts': 'img/amud-architecture.svg',
  'lxc-status-badges': 'img/blog/lxc.svg',
  'docker-homelab-35mb': 'img/blog/docker.svg',
  'plex-jellyfin-live-badges': 'img/blog/plex.svg',
  'home-assistant-dashboard-card': 'img/blog/homeassistant.svg',
  'twelve-free-themes': 'img/blog/themes-grid.svg',
  'securing-homelab-dashboard': 'img/blog/security.svg',
  'reverse-proxy-websockets': 'img/blog/nginx.svg',
  'amud-vs-heimdall-homepage-homarr': 'img/blog/comparison.svg',
  'bare-metal-linux-install': 'img/blog/linux.svg',
  'backup-amud-db': 'img/blog/backup.svg',
  'fix-checking-badge': 'img/AMUD-Dashboard.png',
  'admin-vs-guest': 'img/blog/admin-cockpit.png',
  'wall-mounted-dashboard': 'img/blog/tablet.svg',
  'portainer-deploy': 'img/blog/portainer.svg',
  'why-rust-for-homelab-daemon': 'img/blog/rust.svg',
  'contribute-to-amud': 'img/blog/opensource.svg',
  'why-two-binaries': 'img/amud-architecture.svg',
  'migrated-from-homepage': 'img/AMUD-Dashboard.png',
  'sqlite-wal-not-postgres': 'img/blog/sqlite.svg',
  'encrypted-secrets-at-rest': 'img/blog/encryption.svg',
};

function readSlug(content) {
  const match = content.match(/^slug:\s*(\S+)\s*$/m);
  if (match) return match[1];
  const fileMatch = content.match(/^---\n([\s\S]*?)\n---/);
  return null;
}

for (const file of fs.readdirSync(blogDir).filter((f) => f.endsWith('.md'))) {
  const filePath = path.join(blogDir, file);
  let content = fs.readFileSync(filePath, 'utf8');
  if (!content.startsWith('---')) continue;

  const slugMatch = content.match(/^slug:\s*(\S+)\s*$/m);
  const slug =
    slugMatch?.[1] ??
    file.replace(/^\d{4}-\d{2}-\d{2}-/, '').replace(/\.md$/, '');
  const cover = COVERS[slug];
  if (!cover) continue;

  const end = content.indexOf('---', 3);
  const fm = content.slice(0, end + 3);
  const body = content.slice(end + 3);

  const updatedFm = fm.includes('image:')
    ? fm.replace(/^image:.*$/m, `image: ${cover}`)
    : fm.replace(/^(---\n)/, `$1image: ${cover}\n`);

  fs.writeFileSync(filePath, updatedFm + body);
  console.log(`${slug} -> ${cover}`);
}
