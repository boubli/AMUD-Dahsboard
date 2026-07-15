import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  icon: string;
  description: ReactNode;
  className: string;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Zero-YAML Configuration',
    icon: '⚙️',
    description: (
      <>
        Every app, category, theme, and integration is stored in SQLite and edited through the built-in UI. No YAML files, no restarts for layout changes.
      </>
    ),
    className: 'bento-large',
  },
  {
    title: 'Native Proxmox & Docker Telemetry',
    icon: '📊',
    description: (
      <>
        <code>amud-agent</code> streams host CPU, RAM, GPU, disk, and bandwidth over WebSockets. LXC and Docker containers get live status badges and start/stop controls — native HTTPS, zero shell scripts.
      </>
    ),
    className: 'bento-tall',
  },
  {
    title: '35MB to 100MB Idle Footprint',
    icon: '🪶',
    description: (
      <>
        Pure compiled Rust. Server and agent combined idle at 35–100MB RAM with sub-millisecond route execution — no PHP-FPM or Node.js runtime.
      </>
    ),
    className: 'bento-standard',
  },
  {
    title: 'Live Media & Smart Home',
    icon: '🏠',
    description: (
      <>
        Jellyfin, Plex, and Home Assistant integrations show now-playing streams, lights, switches, and temperature right on your dashboard cards.
      </>
    ),
    className: 'bento-standard',
  },
  {
    title: 'Homelab Integrations',
    icon: '🔌',
    description: (
      <>
        Pi-hole, AdGuard, Radarr, Sonarr, Overseerr, Jellyseerr, and RSS feeds — each app card can show live stats. RSS headlines are guest-friendly.
      </>
    ),
    className: 'bento-standard',
  },
  {
    title: 'Wake-on-LAN & Webhooks',
    icon: '⚡',
    description: (
      <>
        Wake offline machines with one click. Webhooks fire on container start/stop and agent connect/disconnect — Discord, Telegram, or generic JSON.
      </>
    ),
    className: 'bento-wide',
  },
  {
    title: '18 Themes, Drag-and-Drop, Light Mode',
    icon: '🎨',
    description: (
      <>
        Bundled offline themes, custom CSS injection, bento card spans, admin drag-and-drop reorder, video wallpapers, and a full light-mode palette.
      </>
    ),
    className: 'bento-standard',
  },
  {
    title: 'Security Built In',
    icon: '🔒',
    description: (
      <>
        Admin and Guest roles, Argon2id passwords, encrypted API keys at rest, audit log, CSRF protection, rate limits, and SSRF-safe outbound requests.
      </>
    ),
    className: 'bento-standard',
  },
  {
    title: 'Backup, Updates & Audit',
    icon: '💾',
    description: (
      <>
        Export and restore your SQLite database from Settings. Native installs get in-app updates from GitHub Releases. Every admin action is audit-logged.
      </>
    ),
    className: 'bento-standard',
  },
];

function Feature({title, icon, description, className}: Readonly<FeatureItem>) {
  return (
    <div className={clsx('glass-card bento-item', className)}>
      <div className="bento-icon-wrapper">
        {icon}
      </div>
      <Heading as="h3" className="bento-title">
        {title}
      </Heading>
      <p className="bento-description">
        {description}
      </p>
    </div>
  );
}

function HomepageDemos(): ReactNode {
  return (
    <div className="homepage-demos" style={{ maxWidth: '960px', margin: '0 auto 4rem auto' }}>
      <Heading as="h2" style={{ textAlign: 'center', marginBottom: '2rem', color: '#fff' }}>
        See it in action
      </Heading>
      <div className="homepage-demos__grid">
        <figure>
          <img src="img/AMUD-Dashboard_add_app.png" alt="Adding an app in AMUD Dashboard" />
          <figcaption>Add an app — no YAML</figcaption>
        </figure>
        <figure>
          <img src="img/AMUD-Dashboard_settings_Branding.png" alt="AMUD Settings — Branding" />
          <figcaption>Settings — Branding</figcaption>
        </figure>
        <figure>
          <img src="img/amud-update.gif" alt="Updating AMUD from Settings" />
          <figcaption>One-click update (native install)</figcaption>
        </figure>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features} style={{ padding: '6rem 0' }}>
      <div className="container">
        <HomepageDemos />
        <div className="bento-grid">
          {FeatureList.map((props) => (
            <Feature key={props.title} {...props} />
          ))}
        </div>
        <div style={{ textAlign: 'center', marginTop: '3rem' }}>
          <Link className="button button--primary button--lg" to="/docs/features">
            Full feature list →
          </Link>
        </div>
      </div>
    </section>
  );
}
