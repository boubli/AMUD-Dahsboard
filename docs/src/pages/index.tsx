import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';

import styles from './index.module.css';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)} style={{ backgroundColor: '#1a1b26', backgroundImage: 'linear-gradient(to bottom right, #1a1b26, #24283b)' }}>
      <div className="container" style={{ position: 'relative', zIndex: 2 }}>
        <Heading as="h1" className="hero__title" style={{ color: '#fff', fontSize: '4rem', fontWeight: 800 }}>
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle" style={{ color: '#a9b1d6', fontSize: '1.5rem', marginBottom: '2rem' }}>
          {siteConfig.tagline}
        </p>
        <div className={styles.buttons} style={{ display: 'flex', gap: '1rem', justifyContent: 'center' }}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/intro"
            style={{ backgroundColor: '#7aa2f7', color: '#1a1b26', border: 'none', fontWeight: 'bold' }}>
            Get Started
          </Link>
          <Link
            className="button button--outline button--lg"
            to="https://github.com/boubli/AMUD-Dashboard"
            style={{ color: '#7aa2f7', borderColor: '#7aa2f7', fontWeight: 'bold' }}>
            GitHub Repository
          </Link>
        </div>
        <div style={{ marginTop: '3rem', borderRadius: '12px', overflow: 'hidden', boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)' }}>
          <img src="img/AMUD-Dashboard.png" alt="AMUD Dashboard Preview" style={{ width: '100%', display: 'block' }} />
        </div>
      </div>
    </header>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={`Home`}
      description="AMUD Ecosystem Landing Page">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
