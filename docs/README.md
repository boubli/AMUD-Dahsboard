# AMUD Dashboard — Docs site

Built with [Docusaurus](https://docusaurus.io/). Published via GitHub Actions on release tags.

**Live site:** https://boubli.github.io/AMUD-Dashboard/

## Local development

```bash
cd docs
npm ci
npm run start
```

## Production build

```bash
cd docs
npm ci
npm run build
```

Output: `docs/build/`

## Theme gallery (GitHub Pages)

- Route: `/themes` — bundled themes, category filters, copy CSS/wallpaper
- Static assets: `docs/static/themes/` (CSS, wallpapers, previews)

## Release docs

When tagging a release, update:

- `.github/release-notes/vX.Y.Z.md`
- `CHANGELOG.md` and `docs/docs/changelog.md`
- `README.md` and `readmes/README.*.md`
