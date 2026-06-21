/** Cover image paths (relative to static/) — topic-specific, no random theme shots. */
const DASHBOARD = 'img/AMUD-Dashboard.png';
const ARCH = 'img/amud-architecture.svg';

const COVERS: Record<string, string> = {
  'why-i-ditched-heavy-dashboards': DASHBOARD,
  'proxmox-one-command-install': 'img/blog/proxmox.svg',
  'zero-yaml-sqlite': 'img/blog/sqlite.svg',
  'realtime-telemetry-no-shell-scripts': ARCH,
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
  'fix-checking-badge': DASHBOARD,
  'admin-vs-guest': 'img/blog/admin-cockpit.png',
  'wall-mounted-dashboard': 'img/blog/tablet.svg',
  'portainer-deploy': 'img/blog/portainer.svg',
  'why-rust-for-homelab-daemon': 'img/blog/rust.svg',
  'contribute-to-amud': 'img/blog/opensource.svg',
  'why-two-binaries': ARCH,
  'migrated-from-homepage': DASHBOARD,
  'sqlite-wal-not-postgres': 'img/blog/sqlite.svg',
  'encrypted-secrets-at-rest': 'img/blog/encryption.svg',
};

export function blogCoverForSlug(slug: string): string {
  return COVERS[slug] ?? DASHBOARD;
}

export function blogCoverMap(): Readonly<Record<string, string>> {
  return COVERS;
}
