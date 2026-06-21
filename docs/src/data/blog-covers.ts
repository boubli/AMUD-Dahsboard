import coverMap from './blog-cover-map.json';

const DEFAULT = 'img/AMUD-Dashboard.png';
const COVERS = coverMap as Record<string, string>;

export function blogCoverForSlug(slug: string): string {
  return COVERS[slug] ?? DEFAULT;
}
