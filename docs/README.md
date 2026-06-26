# AMUD Dashboard — Docs site

Built with [Docusaurus](https://docusaurus.io/). Published to **GitHub Pages** via `.github/workflows/deploy-docs.yml` when `docs/**` changes on `main`.

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

Output: `docs/build/` (uploaded by GitHub Actions — not the legacy `gh-pages` branch).

## Theme gallery (GitHub Pages)

- Route: `/themes` — 37 themes, category filters, copy CSS/wallpaper
- Static assets: `docs/static/themes/` (CSS, wallpapers, previews)
- Blog: `docs/blog/2026-06-25-eighteen-new-themes.md`

## Release docs (v1.5.6.3)

When tagging a release, update:

- `.github/release-notes/v1.5.6.3.md`
- `CHANGELOG.md` and `docs/docs/changelog.md`
- `README.md` and `readmes/README.*.md` (or run `python scripts/update-readme-release.py`)

See `.github/RELEASE_PREP_v1.5.6.3.md` for the full publish checklist.
