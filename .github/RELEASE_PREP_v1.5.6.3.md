# Release prep checklist — v1.5.6.3 (do not publish until validated)

**Target tag:** `v1.5.6.3`  
**Prepared:** 2026-06-26  
**Status:** Documentation ready — **not tagged, not pushed, not on Docker Hub**

## What ships

| Area | Summary |
|------|---------|
| Themes | 37 total; visual picker in Appearance; 18 new packs; vendored wallpapers |
| Guest UI | Compact app cards |
| Integrations | Filled 6-cell cards + 30s refresh (carried from v1.5.6.2) |
| RSS | Add modal + category table fixes |
| CI | Clippy fix + `docs` build job |
| Pages | Theme gallery filters, blog, changelog |

## Before you tag

1. [ ] `cargo fmt --all --check`
2. [ ] `cargo clippy --workspace --all-targets -- -D warnings`
3. [ ] `cargo test --workspace --lib`
4. [ ] `cd docs && npm ci && npm run build`
5. [ ] Manual smoke: Settings → Appearance → Theme Gallery, guest dashboard, admin filled cards
6. [ ] Proxmox test container (if you require audit baseline)
7. [x] Cargo `version` stays `1.5.6` (semver 3-part); release tag is `v1.5.6.3` (4-part product version)

## Publish steps (when ready)

```bash
git add -A
git commit -m "release v1.5.6.3 — theme gallery, 37 themes, guest compact cards"
git push origin main
git tag v1.5.6.3
git push origin v1.5.6.3
```

- GitHub Actions **CI** runs on push
- **deploy-docs.yml** runs when `docs/**` changes → https://boubli.github.io/AMUD-Dashboard/
- **release.yml** / **docker-publish.yml** run on tag (per your workflow setup)

## Docker Hub

Publish manually after tag if you do not auto-publish broken builds. Withdrawn tags: `v1.5.6.1`.

## Post-release

- [ ] Verify https://boubli.github.io/AMUD-Dashboard/themes shows 37 themes + categories
- [ ] Verify GitHub Release assets + release body (paste from `.github/release-notes/v1.5.6.3.md`)
- [ ] Update Docker `:latest` only after container smoke test
