/** Cover image paths (relative to static/) for blog post cards. */
const DEFAULT = 'img/AMUD-Dashboard.png';
const ARCH = 'img/amud-architecture.svg';
const NORD = 'themes/assets/AMUD-Theme-Nord.png';
const DRACULA = 'themes/assets/AMUD-Theme-Dracula.png';
const NEON = 'themes/assets/AMUD-Theme-Neon.png';

const COVERS: Record<string, string> = {
  'why-i-ditched-heavy-dashboards': DEFAULT,
  'proxmox-one-command-install': ARCH,
  'zero-yaml-sqlite': DEFAULT,
  'realtime-telemetry-no-shell-scripts': ARCH,
  'lxc-status-badges': DEFAULT,
  'docker-homelab-35mb': DEFAULT,
  'plex-jellyfin-live-badges': DEFAULT,
  'home-assistant-dashboard-card': DEFAULT,
  'twelve-free-themes': NORD,
  'securing-homelab-dashboard': DRACULA,
  'reverse-proxy-websockets': DRACULA,
  'amud-vs-heimdall-homepage-homarr': DEFAULT,
  'bare-metal-linux': ARCH,
  'backup-amud-db': DEFAULT,
  'fix-checking-badge': DEFAULT,
  'admin-vs-guest': NEON,
  'wall-mounted-dashboard': NORD,
  'portainer-deploy': DEFAULT,
  'why-rust': ARCH,
  'contribute-to-amud': DEFAULT,
  'why-two-binaries': ARCH,
  'migrated-from-homepage': DEFAULT,
  'sqlite-wal-not-postgres': ARCH,
  'encrypted-secrets-at-rest': DRACULA,
};

export function blogCoverForSlug(slug: string): string {
  return COVERS[slug] ?? DEFAULT;
}
