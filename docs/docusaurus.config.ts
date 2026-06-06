import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'AMUD Dashboard',
  tagline: 'Zero-YAML, UI-Driven Homelab Cockpit',
  favicon: 'img/favicon.ico',

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
      links: [
        {
          title: 'Documentation',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/intro',
            },
            {
              label: 'Troubleshooting',
              to: '/docs/troubleshooting',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub Repository',
              href: 'https://github.com/boubli/AMUD-Dashboard',
            },
          ],
        },
        {
          title: 'Support',
          items: [
            {
              label: 'Donate',
              to: '/docs/donate',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} AMUD Dashboard. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
