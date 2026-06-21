/**
 * Sync image: frontmatter in all blog posts from blog-cover-map.json
 * Run from docs/: node scripts/sync-blog-cover-frontmatter.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const blogDir = path.join(__dirname, '..', 'blog');
const COVERS = JSON.parse(
  fs.readFileSync(
    path.join(__dirname, '..', 'src', 'data', 'blog-cover-map.json'),
    'utf8',
  ),
);

for (const file of fs.readdirSync(blogDir).filter((f) => f.endsWith('.md'))) {
  const filePath = path.join(blogDir, file);
  const content = fs.readFileSync(filePath, 'utf8');
  if (!content.startsWith('---')) continue;

  const slugMatch = content.match(/^slug:\s*(\S+)\s*$/m);
  const slug =
    slugMatch?.[1] ??
    file.replace(/^\d{4}-\d{2}-\d{2}-/, '').replace(/\.md$/, '');
  const cover = COVERS[slug];
  if (!cover) continue;

  const end = content.indexOf('---', 3);
  const fm = content.slice(0, end + 3);
  const body = content.slice(end + 3);

  const updatedFm = fm.includes('image:')
    ? fm.replace(/^image:.*$/m, `image: ${cover}`)
    : fm.replace(/^(---\n)/, `$1image: ${cover}\n`);

  fs.writeFileSync(filePath, updatedFm + body);
  console.log(`${slug} -> ${cover}`);
}
