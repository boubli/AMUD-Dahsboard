export type ThemePalette = {
  background: string;
  card: string;
  accent: string;
  text: string;
};

export type ThemeDefinition = {
  id: string;
  name: string;
  description: string;
  tags: string[];
  palette: ThemePalette;
  cssFile?: string;
  previewImage?: string;
  wallpaper?: string;
  inspiration?: string;
  inspirationUrl?: string;
  /** Shipped with AMUD at /static/themes/ for offline use */
  bundled?: boolean;
};

export type AmudTheme = {
  id: string;
  name: string;
  description: string;
  tags: string[];
  cssFile: string;
  previewImage: string;
  wallpaper?: string;
  palette: ThemePalette;
  inspiration?: string;
  inspirationUrl?: string;
  bundled: boolean;
};
