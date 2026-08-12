import type {AmudTheme, ThemeDefinition} from './types';

const PREVIEW_PREFIX = 'themes/previews/';
const WALLPAPER_PREFIX = 'themes/wallpapers/';

function resolveCategory(definition: ThemeDefinition): string {
  if (definition.category) {
    return definition.category;
  }
  if (definition.id === 'default') {
    return 'default';
  }
  if (definition.tags.includes('advanced')) {
    return 'advanced';
  }
  if (definition.id.startsWith('terminal-')) {
    return 'terminal';
  }
  return 'classic';
}

export function createTheme(definition: ThemeDefinition): AmudTheme {
  const cssFile = definition.cssFile ?? `themes/${definition.id}.css`;
  const previewImage =
    definition.previewImage ?? `${PREVIEW_PREFIX}${definition.id}.webp`;
  const wallpaper =
    definition.wallpaper === ''
      ? undefined
      : definition.wallpaper ?? `${WALLPAPER_PREFIX}${definition.id}.webp`;
  const bundled = definition.bundled ?? definition.id !== 'default';
  const category = resolveCategory(definition);

  return {
    id: definition.id,
    name: definition.name,
    description: definition.description,
    tags: definition.tags,
    category,
    palette: definition.palette,
    cssFile,
    previewImage,
    wallpaper: definition.id === 'default' ? undefined : wallpaper,
    inspiration: definition.inspiration,
    inspirationUrl: definition.inspirationUrl,
    bundled,
  };
}

export function themeSearchText(theme: AmudTheme): string {
  return [theme.name, theme.description, ...theme.tags, theme.inspiration ?? '']
    .join(' ')
    .toLowerCase();
}
