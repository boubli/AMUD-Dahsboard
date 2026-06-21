import THEME_DEFINITIONS from './definitions';
import {createTheme, themeSearchText} from './helpers';

export type {AmudTheme, ThemeDefinition, ThemePalette} from './types';
export {createTheme, themeSearchText};

export const AMUD_THEMES = THEME_DEFINITIONS.map(createTheme);

export const BUNDLED_THEME_IDS = AMUD_THEMES.filter((t) => t.bundled && t.cssFile).map(
  (t) => t.id,
);
