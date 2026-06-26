# Release prep — v1.5.6.4

## Pre-tag checklist

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace --lib`
- [ ] `cd docs && npm ci && npm run build`
- [ ] Smoke: default theme overlay=0 (clear wallpaper)
- [ ] Smoke: Nord + Cyberpunk — glass sliders + overlay slider
- [ ] Smoke: integration card 8 cells, no hover on cards
- [ ] Re-pick bundled theme in Settings if upgrading from pre-1.5.6.4 paste

## Tag

```bash
git tag v1.5.6.4
git push origin main
git push origin v1.5.6.4
```

## Post-release

- [ ] GitHub Release assets (release.yml)
- [ ] Docker Hub `tradmss/amud-dashboard:latest`
- [ ] GitHub Pages docs deploy
