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

  headTags: [
    {
      tagName: 'meta',
      attributes: {
        name: 'description',
        content:
          'AMUD Dashboard — compiled Rust homelab dashboard replacing Homepage/Homarr. Zero YAML, live Proxmox/Docker telemetry, 150+ integrations, LDAP, import tools.',
      },
    },
    {
      tagName: 'meta',
      attributes: {
        name: 'keywords',
        content:
          'AMUD Dashboard, homelab dashboard, rust dashboard, proxmox dashboard, self-hosted, homepage alternative, heimdall alternative, homarr, zero yaml, sqlite',
      },
    },
    {
      tagName: 'meta',
      attributes: {name: 'author', content: 'Youssef Boubli'},
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'alternate',
        type: 'text/plain',
        href: '/AMUD-Dashboard/llms.txt',
        title: 'LLMs',
      },
    },
    {
      tagName: 'script',
      attributes: {type: 'application/ld+json'},
      innerHTML: JSON.stringify({
        '@context': 'https://schema.org',
        '@type': 'SoftwareApplication',
        name: 'AMUD Dashboard',
        alternateName: 'Advanced Modern Unified Dashboard',
        description:
          'AMUD Dashboard is a compiled Rust homelab control center with zero-YAML SQLite configuration and live Proxmox/Docker telemetry.',
        applicationCategory: 'SystemApplication',
        operatingSystem: 'Linux',
        offers: {
          '@type': 'Offer',
          price: '0',
          priceCurrency: 'USD',
        },
        author: {
          '@type': 'Person',
          name: 'Youssef Boubli',
          email: 'bbb.vloger@gmail.com',
          url: 'https://github.com/boubli',
        },
        url: 'https://boubli.github.io/AMUD-Dashboard/',
        downloadUrl: 'https://github.com/boubli/AMUD-Dashboard/releases',
        softwareHelp: 'https://boubli.github.io/AMUD-Dashboard/docs/faq',
        codeRepository: 'https://github.com/boubli/AMUD-Dashboard',
      }),
    },
  ],

  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'de', 'fr', 'es'],
    localeConfigs: {
      en: { label: 'English' },
      de: { label: 'Deutsch' },
      fr: { label: 'Français' },
      es: { label: 'Español' },
    },
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
        blog: {
          routeBasePath: 'blog',
          showReadingTime: true,
          blogSidebarCount: 0,
          blogTitle: 'AMUD Dashboard Blog',
          blogDescription:
            'Homelab notes from building AMUD Dashboard — a Rust dashboard with zero YAML, live Proxmox telemetry, and way too much SQLite.',
          postsPerPage: 12,
          onUntruncatedBlogPosts: 'ignore',
          feedOptions: {
            type: 'all',
            copyright: `Copyright © ${new Date().getFullYear()} AMUD Dashboard`,
          },
          editUrl: 'https://github.com/boubli/AMUD-Dashboard/tree/main/docs/',
        },
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
      title: 'AMUD Dashboard',
      logo: {
        alt: 'AMUD Dashboard Logo',
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
          to: '/blog',
          label: 'Blog',
          position: 'left',
        },
        {
          to: '/themes',
          label: 'Themes',
          position: 'left',
        },
        {
          to: '/docs/changelog',
          label: 'Changelog',
          position: 'left',
        },
        {
          to: '/docs/roadmap',
          label: 'Roadmap',
          position: 'left',
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
              label: 'Blog',
              to: '/blog',
            },
            {
              label: 'Getting Started',
              to: '/docs/intro',
            },
            {
              label: 'Custom Themes',
              to: '/themes',
            },
            {
              label: 'Configuration',
              to: '/docs/configuration',
            },
            {
              label: 'FAQ',
              to: '/docs/faq',
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
              label: 'Changelog',
              to: '/docs/changelog',
            },
            {
              label: 'Roadmap',
              to: '/docs/roadmap',
            },
            {
              label: 'GitHub Releases (binaries)',
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
