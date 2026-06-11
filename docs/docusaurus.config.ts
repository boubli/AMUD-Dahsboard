import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'AMUD Dashboard',
  tagline: 'Unify Your Homelab: The Zero-YAML, UI-Driven Cockpit.',
  favicon: 'img/AMUD-logo.png',

  // Set the production url of your site here
  url: 'https://boubli.github.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/AMUD-Dashboard/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'boubli', // Usually your GitHub org/user name.
  projectName: 'AMUD-Dashboard', // Usually your repo name.
  trailingSlash: false,

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl:
            'https://github.com/boubli/AMUD-Dashboard/tree/main/docs/',
        },
        blog: false, // Disable the blog plugin
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/AMUD-Dashboard.png',
    colorMode: {
      defaultMode: 'dark',
      disableSwitch: false,
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'AMUD',
      logo: {
        alt: 'AMUD Logo',
        src: 'img/AMUD-logo.png',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          to: '/docs/donate',
          label: 'Donate',
          position: 'right',
        },
        {
          href: 'https://github.com/boubli/AMUD-Dashboard',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      logo: {
        alt: 'AMUD Dashboard Logo',
        src: 'img/AMUD-logo.png',
        href: 'https://github.com/boubli/AMUD-Dashboard',
        width: 60,
        height: 60,
      },
      links: [
        {
          title: 'Documentation',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/intro',
            },
            {
              label: 'Configuration',
              to: '/docs/configuration',
            },
            {
              label: 'Troubleshooting',
              to: '/docs/troubleshooting',
            },
          ],
        },
        {
          title: 'Support & Community',
          items: [
            {
              label: 'GitHub Discussions',
              href: 'https://github.com/boubli/AMUD-Dashboard/discussions',
            },
            {
              label: 'Report an Issue',
              href: 'https://github.com/boubli/AMUD-Dashboard/issues',
            },
            {
              label: 'Releases & Changelogs',
              href: 'https://github.com/boubli/AMUD-Dashboard/releases',
            },
          ],
        },
        {
          title: '💖 Donate',
          items: [
            {
              label: 'GitHub Sponsors',
              href: 'https://github.com/sponsors/boubli',
            },
            {
              label: 'Stripe (Card)',
              href: 'https://buy.stripe.com/cNi14n6b9a7v5Jg4Rq4ko00',
            },
            {
              label: 'Ko-fi',
              href: 'https://ko-fi.com/Youssefboubli',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} <strong>AMUD Dashboard</strong>.<br/><span style="color: #ff6b2b; font-size: 0.9em;">Unify Your Homelab: The Zero-YAML, UI-Driven Cockpit.</span>`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
  themes: [
    [
      require.resolve("@easyops-cn/docusaurus-search-local"),
      {
        hashed: true,
      },
    ],
  ],
};

export default config;
