export type ThemePalette = {
  background: string;
  card: string;
  accent: string;
  text: string;
};

export type AmudTheme = {
  id: string;
  name: string;
  description: string;
  tags: string[];
  cssFile: string;
  /** Preview PNG in docs/static/themes/assets/ */
  previewImage: string;
  palette: ThemePalette;
  inspiration?: string;
  inspirationUrl?: string;
};

/** Preview assets in docs/static/themes/assets/ */
const preview = (filename: string) => `themes/assets/${filename}`;

export const AMUD_THEMES: AmudTheme[] = [
  {
    id: 'default',
    name: 'AMUD Default',
    description:
      'The built-in orange glass cockpit look. No custom CSS required — reset the Custom CSS field or leave it empty.',
    tags: ['default', 'orange', 'glass', 'built-in'],
    cssFile: '',
    previewImage: 'img/AMUD-Dashboard.png',
    palette: {
      background: '#0b0e14',
      card: 'rgba(15,20,25,0.45)',
      accent: '#cf6427',
      text: '#f8fafc',
    },
  },
  {
    id: 'dracula',
    name: 'Dracula',
    description:
      'Classic dark purple hacker aesthetic with soft purple accents and high-contrast pastel text.',
    tags: ['dark', 'purple', 'hacker', 'dracula'],
    cssFile: 'themes/dracula.css',
    previewImage: preview('AMUD-Theme-Dracula.png'),
    palette: {
      background: '#282a36',
      card: '#44475a',
      accent: '#bd93f9',
      text: '#f8f8f2',
    },
    inspiration: 'Dracula',
    inspirationUrl: 'https://draculatheme.com',
  },
  {
    id: 'nord',
    name: 'Nord',
    description:
      'Clean Arctic blue palette — calm, professional, and easy on the eyes during long monitoring sessions.',
    tags: ['dark', 'blue', 'arctic', 'nord', 'professional'],
    cssFile: 'themes/nord.css',
    previewImage: preview('AMUD-Theme-Nord.png'),
    palette: {
      background: '#2e3440',
      card: '#3b4252',
      accent: '#88c0d0',
      text: '#eceff4',
    },
    inspiration: 'Nord',
    inspirationUrl: 'https://www.nordtheme.com',
  },
  {
    id: 'cyberpunk-neon',
    name: 'Cyberpunk Neon',
    description:
      'High-contrast neon pink and electric cyan on deep black with glowing card edges and scanline overlay.',
    tags: ['dark', 'neon', 'pink', 'cyberpunk', 'sci-fi'],
    cssFile: 'themes/cyberpunk-neon.css',
    previewImage: preview('AMUD-Theme-Neon.png'),
    palette: {
      background: '#0a0a0f',
      card: '#12121a',
      accent: '#ff2d95',
      text: '#e0e0ff',
    },
  },
  {
    id: 'sunset-warm',
    name: 'Sunset Warm',
    description:
      'Warm earthy tones with amber accents — cozy golden-hour vibes, great for wall-mounted displays.',
    tags: ['dark', 'warm', 'amber', 'orange', 'cozy'],
    cssFile: 'themes/sunset-warm.css',
    previewImage: preview('AMUD-Theme-Sunset-Warm.png'),
    palette: {
      background: '#1a1410',
      card: '#2a2018',
      accent: '#f59e0b',
      text: '#fef3c7',
    },
  },
  {
    id: 'catppuccin-mocha',
    name: 'Catppuccin Mocha',
    description:
      'Soft pastel dark theme with lavender accents — popular among developers for its gentle contrast.',
    tags: ['dark', 'pastel', 'purple', 'catppuccin', 'soft'],
    cssFile: 'themes/catppuccin-mocha.css',
    previewImage: preview('AMUD-Theme-Catppuccin-Mocha.png'),
    palette: {
      background: '#1e1e2e',
      card: '#313244',
      accent: '#cba6f7',
      text: '#cdd6f4',
    },
    inspiration: 'Catppuccin',
    inspirationUrl: 'https://catppuccin.com',
  },
  {
    id: 'gruvbox-dark',
    name: 'Gruvbox Dark',
    description:
      'Warm retro terminal aesthetic with earthy browns and vibrant orange accents.',
    tags: ['dark', 'retro', 'warm', 'gruvbox', 'terminal'],
    cssFile: 'themes/gruvbox-dark.css',
    previewImage: preview('AMUD-Theme-Gruvbox-Dark.png'),
    palette: {
      background: '#282828',
      card: '#3c3836',
      accent: '#fe8019',
      text: '#ebdbb2',
    },
    inspiration: 'Gruvbox',
    inspirationUrl: 'https://github.com/morhetz/gruvbox',
  },
  {
    id: 'tokyo-night',
    name: 'Tokyo Night',
    description:
      'Deep blue city-night palette with crisp blue accents — a favorite for coding dashboards.',
    tags: ['dark', 'blue', 'purple', 'tokyo', 'night'],
    cssFile: 'themes/tokyo-night.css',
    previewImage: preview('AMUD-Theme-Tokyo-Night.png'),
    palette: {
      background: '#1a1b26',
      card: '#24283b',
      accent: '#7aa2f7',
      text: '#c0caf5',
    },
    inspiration: 'Tokyo Night',
    inspirationUrl: 'https://github.com/enkia/tokyo-night-vscode-theme',
  },
  {
    id: 'one-dark',
    name: 'One Dark',
    description:
      'The classic Atom editor palette — balanced blue accents on neutral dark gray.',
    tags: ['dark', 'blue', 'atom', 'one-dark', 'developer'],
    cssFile: 'themes/one-dark.css',
    previewImage: preview('AMUD-Theme-One-Dark.png'),
    palette: {
      background: '#282c34',
      card: '#2c313a',
      accent: '#61afef',
      text: '#abb2bf',
    },
    inspiration: 'One Dark',
    inspirationUrl: 'https://atom.io/themes/one-dark-ui',
  },
  {
    id: 'everforest',
    name: 'Everforest',
    description:
      'Calm muted greens and warm beige text — a forest-inspired palette for relaxed monitoring.',
    tags: ['dark', 'green', 'forest', 'everforest', 'calm'],
    cssFile: 'themes/everforest.css',
    previewImage: preview('AMUD-Theme-Everforest.png'),
    palette: {
      background: '#2d353b',
      card: '#343f44',
      accent: '#a7c080',
      text: '#d3c6aa',
    },
    inspiration: 'Everforest',
    inspirationUrl: 'https://github.com/sainnhe/everforest',
  },
  {
    id: 'monokai',
    name: 'Monokai',
    description:
      'High-contrast developer classic — neon green accents with hot-pink header highlights.',
    tags: ['dark', 'green', 'monokai', 'developer', 'high-contrast'],
    cssFile: 'themes/monokai.css',
    previewImage: preview('AMUD-Theme-Monokai.png'),
    palette: {
      background: '#272822',
      card: '#3e3d32',
      accent: '#a6e22e',
      text: '#f8f8f2',
    },
    inspiration: 'Monokai',
    inspirationUrl: 'https://monokai.pro',
  },
  {
    id: 'rose-pine',
    name: 'Rose Pine',
    description:
      'Elegant muted rose and pine tones with soft lavender secondary text.',
    tags: ['dark', 'rose', 'pine', 'elegant', 'muted'],
    cssFile: 'themes/rose-pine.css',
    previewImage: preview('AMUD-Theme-Rose-Pine.png'),
    palette: {
      background: '#191724',
      card: '#1f1d2e',
      accent: '#ebbcba',
      text: '#e0def4',
    },
    inspiration: 'Rosé Pine',
    inspirationUrl: 'https://rosepinetheme.com',
  },
  {
    id: 'solarized-dark',
    name: 'Solarized Dark',
    description:
      'Scientific low-contrast dark theme with cyan-blue accents — easy on the eyes for long sessions.',
    tags: ['dark', 'blue', 'solarized', 'low-contrast', 'scientific'],
    cssFile: 'themes/solarized-dark.css',
    previewImage: preview('AMUD-Theme-Solarized-Dark.png'),
    palette: {
      background: '#002b36',
      card: '#073642',
      accent: '#268bd2',
      text: '#93a1a1',
    },
    inspiration: 'Solarized',
    inspirationUrl: 'https://ethanschoonover.com/solarized/',
  },
];

export function themeCssUrl(baseUrl: string, theme: AmudTheme): string | null {
  if (!theme.cssFile) return null;
  return `${baseUrl}${theme.cssFile}`;
}

export function themeSearchText(theme: AmudTheme): string {
  return [
    theme.name,
    theme.description,
    ...theme.tags,
    theme.inspiration ?? '',
  ]
    .join(' ')
    .toLowerCase();
}
