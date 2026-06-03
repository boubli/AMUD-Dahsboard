import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: '~10MB Idle Footprint',
    Svg: require('@site/static/img/undraw_docusaurus_mountain.svg').default,
    description: (
      <>
        Built in pure, compiled Rust. AMUD executes native machine code with zero interpreter overhead, running the entire dashboard and telemetry layer in under 10MB of RAM.
      </>
    ),
  },
  {
    title: 'Zero-YAML Configuration',
    Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
    description: (
      <>
        No more spending hours manually writing text files. AMUD provides a modern, responsive, built-in User Interface to configure all of your apps and settings instantly.
      </>
    ),
  },
  {
    title: 'Native LXC Telemetry',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        AMUD natively polls your Proxmox VE host via the <code>pvesh</code> API to stream real-time CPU, RAM, and Status updates directly to your custom application cards.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        {/* <Svg className={styles.featureSvg} role="img" /> */}
      </div>
      <div className="text--center padding-horiz--md" style={{ marginTop: '2rem' }}>
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
