# Maintainer tooling (local only)

This folder is a **template**. It is not used by homelab installs.

## Setup (once per machine)

```bash
cp -r maintainer-local.example maintainer-local
# Windows: Copy-Item -Recurse maintainer-local.example maintainer-local
```

`maintainer-local/` is gitignored — **never push it**. Use it for everything that is not homelab-facing: theme regeneration, `test_local.sh`, dashboard test scripts (`test-*.mjs`), Sonar, and git hooks.

Enable git hooks from repo root:

```bash
bash maintainer-local/scripts/setup-githooks.sh
```

## Theme regeneration (before a release)

Run from **repo root**:

```bash
python maintainer-local/scripts/fetch-theme-wallpapers.py
python maintainer-local/scripts/compress-theme-images.py
python maintainer-local/scripts/generate-theme-overhaul.py
python maintainer-local/scripts/validate-theme-assets.py
bash maintainer-local/scripts/sync-themes.sh
```

Commit only the outputs under `ui/static/themes/` and `docs/static/themes/`.

## Pre-push Rust checks

```bash
bash maintainer-local/scripts/check-rust.sh
# or: maintainer-local/scripts/check-rust.ps1
```

## Local tests (maintainer-local only)

```bash
bash maintainer-local/scripts/test_local.sh
node maintainer-local/scripts/test-dashboard-rates.mjs
node maintainer-local/scripts/test-disk-volumes.mjs
```

## SonarCloud

Copy `sonar-project.properties` to repo root when scanning, or pass `-Dproject.settings=maintainer-local/sonar-project.properties`.
