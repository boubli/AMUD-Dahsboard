import type {ReactNode} from 'react';
import clsx from 'clsx';
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
        No more spending hours manually writing text files. AMUD provides a modern, responsive, built-in User Interface to configure all of your apps and settings instantly.
      </>
    ),
    className: 'bento-large',
  },
  {
    title: 'Native LXC Telemetry',
    icon: '📊',
    description: (
      <>
        AMUD natively polls your Proxmox VE host via the <code>pvesh</code> API to stream real-time CPU, RAM, and Status updates directly to your custom application cards.
      </>
    ),
    className: 'bento-tall',
  },
  {
    title: '35MB to 100MB Idle Footprint',
    icon: '🪶',
    description: (
      <>
        Built in pure, compiled Rust. AMUD executes native machine code with zero interpreter overhead, running the entire dashboard and telemetry layer in just 35-100MB of RAM.
      </>
    ),
    className: 'bento-standard',
  },
  {
    title: 'Smart Home Integration',
    icon: '🏠',
    description: (
      <>
        Connect AMUD to your Home Assistant instance to pull live telemetry. View your active lights, switches, and average home temperature directly on your dashboard.
      </>
    ),
    className: 'bento-standard',
  },
  {
    title: 'Built-in Wake-on-LAN',
    icon: '⚡',
    description: (
      <>
        Easily wake up your offline servers or VMs via UDP Magic Packets directly from the dashboard UI with a single click. No external tools needed.
      </>
    ),
    className: 'bento-wide',
  },
  {
    title: 'Database Backups & Custom CSS',
    icon: '🎨',
    description: (
      <>
        Export your entire SQLite database config and restore it anywhere. Inject custom CSS directly from the UI to theme the dashboard exactly how you like it.
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

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features} style={{ padding: '6rem 0' }}>
      <div className="container">
        <div className="bento-grid">
          {FeatureList.map((props) => (
            <Feature key={props.title} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
