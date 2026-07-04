---
sidebar_position: 2
title: Changelog
---

# Changelog

Every stable release is tagged on GitHub with binaries, checksums, and install scripts.

**Download latest:** [GitHub Releases](https://github.com/boubli/AMUD-Dashboard/releases/latest)

---

## v1.7.4 — Integration picker + Ollama / Open WebUI

**2026-07-04** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.7.4)

### Added
- **Ollama** and **Open WebUI** integrations (AI & LLM group)
- **Custom integration picker** — logos, grouped headers, search; replaces native select

### Fixed
- **Integration dropdown** — white optgroup bars on Windows dark theme

### Improved
- Manifest `icon` field; PWA cache v31

---

## v1.7.3 — Integration dropdown + Unraid follow-up

**2026-07-01** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.7.3)

### Fixed
- **Integration dropdown** — CSP nonce on manifest loader; Add/Edit App shows full catalog again
- **Unraid Docker** — image runs as UID 99 (no runtime `su-exec`); fixes `setgroups` loop ([#16](https://github.com/boubli/AMUD-Dashboard/issues/16))
- **Agent** — `--user 0` in Unraid template and Compose for Docker socket

### Improved
- PWA cache v30; troubleshooting docs for empty Integration list

---

## v1.7.2 — Unraid Docker first-boot fix

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.7.2)

### Fixed
- **Unraid Community Apps** — dashboard runs as PUID 99 by default; fixes `.amud-secrets-key` permission denied ([#16](https://github.com/boubli/AMUD-Dashboard/issues/16))
- **Docker agent IPC** — `AMUD_SOCKET_MODE=666` in compose/Unraid templates

### Improved
- Permission-denied startup hints with links to [troubleshooting](https://boubli.github.io/AMUD-Dashboard/docs/troubleshooting#unraid-secrets-key-permission-denied)

---

## v1.7.1 — Mobile/PWA hotfix

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.7.1)

### Fixed
- **Settings hamburger** — outside-click scoped to nav wrapper so menu stays open on tap
- **Desktop guest cards** — horizontal icon+name | ONLINE layout restored; mobile keeps 2-col vertical tiles

### Improved
- **Admin cards (mobile)** — compact metrics grid, no description line, touch-friendly edit button
- **Service worker** — `sw.js` v29

---

## v1.7.0 — Integration catalog parity

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.7.0)

### Added
- **Catalog parity** — 130+ integration types in manifest with correct poll TTL
- **Homepage import** — 20+ new widget aliases (`diskstation`, `cloudflared`, `firefly`, …)
- **Watchtower, Ombi, FileBrowser** — new app card integrations
- **Manifest-driven Add App UI** — integration dropdown from `/api/integrations/manifest`

### Improved
- **Service worker** — `sw.js` v28

---

## v1.6.5 — Mobile PWA follow-ups

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.5)

### Improved
- **Weather card** — responsive temperature sizing on narrow hero widgets
- **Guest grid** — 2-column compact cards on mobile (icon, name, status)
- **Admin cards** — single-row header on mobile
- **Settings** — hamburger overflow menu on phones/tablets
- **Service worker** — `sw.js` v27

---

## v1.6.4.1 — Default logo hotfix

**2026-06-28** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.4.1)

### Fixed
- **Default logo** — topbar no longer shows `{{if app_logo}}` when no custom logo is set; AMUD PNG shows via CSS again
- **Custom logos** — uploaded logos still render as `<img>` on all pages

---

## v1.6.4 — Branding logo fix, PWA mobile polish

**2026-06-28** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.4)

### Fixed
- **Custom logo for guests / login** — public read for `/uploads/` branding images; `<img>` logo on dashboard, login, settings
- **Mobile overflow menu** — fixed panel below topbar; outside-click close

### Improved
- **PWA mobile UI** — centered topbar, hero layout, weather card, telemetry grid, expandable search
- **Custom logo → favicon / PWA** — Settings Dashboard Logo drives manifest and icons everywhere
- **Service worker** — `sw.js` v26

---

## v1.6.3 — ARM64, Pages, PWA

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.3)

### Added
- **ARM64 release binaries** — `amud-server-arm64` and `amud-agent-arm64` on GitHub Releases.
- **Multi-arch Docker** — `linux/amd64` and `linux/arm64` images.
- **GitHub Pages deploy** restored — rotating homepage hero + GIF demos.
- **PWA install UX** — manifest shortcuts, install banner, offline shell (`sw.js` v23).

### Improved
- Mobile topbar overflow menu and touch targets.
- Architecture-aware `update-amud.sh` and in-app updater.

---

## v1.6.2 — Local bundled themes

**2026-06-27** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.2)

### Added
- **Manifest v5** — icons, wallpapers, and gallery previews bundled in `ui.tar.gz` (WebP + SVG), fully offline.
- **Per-profile icon libraries** and **`theme-layouts/`** for distinct UI per theme.
- **`theme-picker.js`** — settings gallery uses local preview images.

### Fixed
- Theme gallery black boxes when CDN unavailable.
- v1.6.1 withdrawn (broken CDN assets).

---

## v1.6.1 — Withdrawn

**2026-06-27** — This release was **withdrawn** (GitHub release and tag removed). Theme gallery previews and CDN assets did not work reliably offline. Use **v1.6.0** until **v1.6.2** ships with bundled local theme assets.

---

## v1.6.0 — Universal dashboard parity

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.0)

### Added
- Integration cache + poll coordinator
- Homepage YAML and Homarr JSON importers
- Homepage Docker label discovery
- Custom API, LDAP, OIDC admin groups, per-user boards
- Plex/Jellyfin per-app cards, *arr calendar widget, release trackers
- 40+ long-tail integrations and server-driven integration manifest
- Multi-node agent `node_tag` support
- Comparison and migration documentation

### Improved
- Generic tier-2 integration card template
- Promoted health-only integrations to full cards

---

## v1.5.6.4 — Theme fix & 13 integrations

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.6.4)

### Added
- Lidarr, Readarr, Whisparr, SABnzbd, NZBGet, Transmission, Jackett, Tautulli, Audiobookshelf, Immich, Tdarr, Maintainerr, Frigate integrations
- Wallpaper overlay strength slider in Glass & Layout Parameters

### Fixed
- All bundled themes respect glass/blur/radius/overlay sliders
- Integration cards always show 8 cells; card hover removed
- Guest dashboard compact grid row spacing (no large gaps between cards)

---

## v1.5.6.3 — Theme gallery & 37 themes

**2026-06-26** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.6.3)

**Added**
- Visual Theme Gallery in Settings → Appearance (click to preview CSS + wallpaper)
- 37 bundled themes including 18 new packs; offline wallpapers and preview thumbnails
- `active_theme_id` setting

**Fixed**
- Guest compact app cards
- RSS add-feed modal and category table layout
- CI clippy + docs build job

**Improved**
- GitHub Pages theme gallery category filters; wallpaper fetch script

---

## v1.5.6.2 — Filled integration cards

**2026-06-26** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.6.2)

**Fixed**
- Integration cards fill tall 1x2 layout with a unified 6-cell metrics grid (no empty middle)
- Expanded API stats per integration (queue, missing, library size, speeds, indexers, etc.)
- Integration data refreshes every 30 seconds on visible cards

---

## v1.5.6.1 — Tall integration cards, RSS table, restore fix (withdrawn)

**2026-06-25** — Pulled from Docker Hub; use v1.5.6.2 instead.


## v1.5.6.0 — Fix blank app card bodies (#15)

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.6.0) · [Compare v1.5.5.9…v1.5.6.0](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.5.9...v1.5.6.0)

**Fixed**
- **Dual metrics row** — integration stats and CPU/RAM no longer stack as two rows; they share one slot (integration default, hover for container metrics)
- App card bodies empty while **ONLINE** still showed — common on Unraid/Docker after the v1.5.5.9 bento grid change ([#15](https://github.com/boubli/AMUD-Dashboard/issues/15))
- Integration cards show a loading placeholder before Alpine fetch completes
- Drag reorder targets the app grid only (not dashboard widgets)

**Docs**
- Dashboard Widgets guide simplified; troubleshooting entry for empty card bodies

---

## v1.5.5.9 — Uniform bento grid layout

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.9) · [Compare v1.5.5.8…v1.5.5.9](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.5.8...v1.5.5.9)

**Fixed**
- Mixed card sizes (1x1, 2x1, 1x2) pack cleanly without vertical holes under shorter cards
- Settings Appearance preview reflects bento row layout

---

## v1.5.5.8 — Test host visibility HTTP 415

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.8) · [Compare v1.5.5.7…v1.5.5.8](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.5.7...v1.5.5.8)

**Fixed**
- **Test host visibility** in Settings sends correct form encoding (fixes HTTP 415)

---

## v1.5.5.7 — Unraid host telemetry + per-mount disk tiles

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.7) · [Compare v1.5.5.6…v1.5.5.7](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.5.6...v1.5.5.7)

Thanks to [@Inch-high](https://github.com/Inch-high) for [#12](https://github.com/boubli/AMUD-Dashboard/issues/12).

**Added**
- Unraid agent template: **host network** + optional array/cache bind-mounts
- Per-mount disk tiles when multiple paths are configured
- **Test host visibility** in Settings (admin diagnostics)
- `telemetry_scope` and auto-detect hints on dashboard

**Fixed**
- Bandwidth/disk mapping on Docker/Unraid when agent could not see host NICs or mounts
- Bond/eth0 WAN heuristic and mapping fallbacks

---

## v1.5.5.6 — Dashboard click UX + telemetry normalization

**2026-06-25** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.6) · [Compare v1.5.5.5…v1.5.5.6](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.5.5...v1.5.5.6)

**Fixed**
- Full-card click opens apps while preserving in-card interactive controls
- Integration metrics are visible by default without hover
- Telemetry interface/disk mapping lists are normalized and deduplicated
- Settings help text explains canonicalization behavior

---

## v1.5.5.3 — Fix integration hover + admin redirect

**2026-06-24** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.3) · [Compare v1.5.5.2…v1.5.5.3](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.5.2...v1.5.5.3)

**Fixed**
- Integration stats swap inside a **fixed metrics slot** (replaces broken v1.5.5.2 drawer)
- `/admin/settings` redirects to **login** for non-admins

**Upgrade:** Proxmox — `curl -sSL …/update-amud.sh | bash` · Docker — `docker compose pull && docker compose up -d` — verify **Settings → System** shows `v1.5.5.3`.

---

## v1.5.5.2 — superseded (do not use)

Broken integration hover drawer overlapped cards below. Use **v1.5.5.3** instead.

---

## v1.5.5.1 — qBittorrent, Bazarr, hover integration stats

**2026-06-24** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.1) · [Compare v1.5.5.0…v1.5.5.1](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.5.0...v1.5.5.1)

**Added**
- **qBittorrent** and **Bazarr** app card integrations

**Improved**
- Integration stats on **card hover** (compact cards)
- Telemetry mapping discovery docs + Settings link

**Upgrade:** `curl -sSL …/update-amud.sh | bash` — verify **Settings → System** shows `v1.5.5.1`.

---

## v1.5.5.0 — Integrations, telemetry mapping, Unraid polish

**2026-06-23** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.0) · [Compare v1.5.4.3…v1.5.5.0](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.4.3...v1.5.5.0)

**Added**
- **Prowlarr, Uptime Kuma, Cloudflare Tunnel, Peanut (UPS)** app card integrations
- **Per-app CPU/RAM toggle**; **host telemetry mapping** (network interfaces + disk mounts)
- Guest **ONLINE/OFFLINE** status; login page branding

**Improved**
- **AdGuard Home** credential UX and stats parsing; Unraid permission/password docs

**Upgrade:** `curl -sSL …/update-amud.sh | bash` — verify **Settings → System** shows `v1.5.5.0`.

---

## v1.5.4.2 — Fix live telemetry after feeds refactor

**2026-06-23** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.4.2) · [Compare v1.5.4.1…v1.5.4.2](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.4.1...v1.5.4.2)

**Fix**
- **Telemetry + badges** — `updateClock()` JS crash blocked WebSocket; GPU/CPU/RAM and status badges stuck at 0% / CHECKING

**Upgrade:** `curl -sSL …/update-amud.sh | bash` then hard-refresh (`Ctrl+Shift+R`).

---

## v1.5.4.1 — Audit log migration fix

**2026-06-23** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.4.1) · [Compare v1.5.4.0…v1.5.4.1](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.4.0...v1.5.4.1)

**Fix**
- **Audit log** — upgraded databases missing `username` column are migrated automatically on server start

**Upgrade:** `curl -sSL …/update-amud.sh | bash` — verify **Settings → System** shows `v1.5.4.1`.

---

## v1.5.4.0 — Feeds hero, category colors, RSS reorder

**2026-06-23** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.4.0) · [Compare v1.5.3.0…v1.5.4.0](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.3.0...v1.5.4.0)

**Feeds**
- **Hero headline** — featured story at the top of `/feeds`
- **Category tab colors** — accent per feed category for guests and admins
- **Drag reorder** — Settings → RSS Feeds table; order matches `/feeds`

**Upgrade:** `curl -sSL …/update-amud.sh | bash` — verify **Settings → System** shows `v1.5.4.0`.

---

## v1.5.3.0 — RSS management, audit expansion, homelab polish

**2026-06-23** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.3.0) · [Compare v1.5.2.1…v1.5.3.0](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.2.1...v1.5.3.0)

**RSS**
- **Settings tab** — full CRUD for RSS/Atom feeds
- **`/feeds` page** — RSS-only dashboard view; nav link visible to guests

**Security & audit**
- **Webhook LAN toggle** — optional private-IP delivery for ntfy/Gotify on homelab LAN
- **Audit expansion** — wake, reorder, and category actions logged
- **Audit filter** — search and action dropdown in Settings

**Backup**
- **Validate before restore** — preview counts; confirm dialog before overwrite
- **Last export timestamp** on Backup tab

**Fixes**
- Home Assistant polling respects self-signed TLS setting

**Upgrade:** `curl -sSL …/update-amud.sh | bash` — verify **Settings → System** shows `v1.5.3.0`.

---

## v1.5.2.1 — Overseerr integration and Docker optimizations

**2026-06-23** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.2.1) · [Compare v1.5.2.0…v1.5.2.1](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.2.0...v1.5.2.1)

**New Integrations**
- **Overseerr & Jellyseerr** — Added native API endpoints to monitor pending media requests directly on your dashboard widgets.

**Improvements**
- **Multi-Language READMEs** — Added documentation translations for Arabic, Hindi, Spanish, French, German, Italian, Portuguese, Russian, Chinese, Japanese, and Korean.
- **Docker Optimizations** — Overhauled `.dockerignore` to block non-essential documentation and script files from entering the Docker build context, resulting in faster builds and smaller images.

---

## v1.5.2.0 — RSS feed integration and guest telemetry

**2026-06-23** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.2.0) · [Compare v1.5.1.0…v1.5.2.0](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.1.0...v1.5.2.0)

**RSS Feeds**
- **Native Integration** — Stream the top 3 headlines from any valid RSS or Atom feed directly on app cards.
- **Guest Access** — RSS integration data is explicitly permitted for unauthenticated guest users, unlike admin-only integrations.
- **Under the hood** — Rust `feed-rs` implementation for fast, reliable parsing.

**Upgrade:** `curl -sSL …/update-amud.sh | bash` — verify **Settings → System** shows `v1.5.2.0`.

---

## v1.5.1.0 — Appearance cleanup, offline themes, updater fix

**2026-06-21** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.1.0) · [Compare v1.5.0.0…v1.5.1.0](https://github.com/boubli/AMUD-Dashboard/compare/v1.5.0.0...v1.5.1.0)

**Themes**
- **Bundled theme** dropdown in Settings — 18 CSS files ship with the UI, no internet needed.
- Six new advanced themes (Terminal Phosphor, Vaporwave Grid, Blueprint Tech, Luxury Gold, Holographic Prism, Brutalist Mono).
- Gallery adds **Download CSS**; each theme is its own definition file in the docs repo.

**Appearance**
- Removed overlay tint presets; quick accent + light/dark, glass/layout sliders, Custom CSS + gallery link.
- Live preview stays inside the mini scene; Custom CSS applies live.

**Audit & updates**
- Audit log schema ensured on boot; API returns 503 if unreadable; settings save writes audit rows.
- In-app updater checksum lookup fixed for Proxmox (`SHA256SUMS` matched by basename).

**Upgrade:** `curl -sSL …/update-amud.sh | bash` — verify **Settings → System** shows `v1.5.1.0`.

---

## v1.5.0.0 — Visual upgrade, drag-and-drop, light mode

**2026-06-20** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.0.0) · [Compare v1.4.2.2…v1.5.0.0](https://github.com/boubli/AMUD-Dashboard/compare/v1.4.2.2...v1.5.0.0)

**Visual upgrade**
- Staggered card animations, hover micro-interactions, floating orbs, pulsing status badges, greeting shimmer.
- Admin **drag-and-drop** card reorder (handle-only, CSRF + SQLite `sort_order`).
- **Bento spans** per app (`1x1`, `2x1`, `1x2`); collapse on mobile.
- **Light mode** with full token overrides on dashboard and settings.
- **Video wallpaper** (`.mp4`/`.webm`/`.ogg`).
- **Live settings preview** — accent, glass, wallpaper, overlay, grid columns, accent glow.

**Docs & marketing**
- Blog (24 posts), FAQ, `llms.txt`, theme gallery on GitHub Pages, JSON-LD SEO.
- Card-grid blog UI with topic cover art; unified **AMUD Dashboard** branding.

**Backend & polish**
- `sanitize_theme_mode`, `sanitize_card_span`, reorder integration tests.
- Drag: filter guard, error toasts, rollback, touch support; admin-only script load.
- Light mode body/orb fixes; Sonar and CI rustfmt clean.

**Upgrade:** `curl -sSL …/update-amud.sh | bash` — verify **Settings → System** shows `v1.5.0.0`.

---

## v1.4.2.2 — Dashboard telemetry only (no System page)

**2026-06-20** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.4.2.2) · [Compare v1.4.1.0…v1.4.2.2](https://github.com/boubli/AMUD-Dashboard/compare/v1.4.1.0...v1.4.2.2)

**Removed**
- `/telemetry` page and **System** topbar button (dashboard telemetry row stays on the home page).

**Host apps**
- Host-based services like Beszel/Filebrowser now get fallback telemetry even when not running as LXC.
- Proxmox app status is driven by host agent connectivity to avoid false OFFLINE badge states.

**Telemetry UI**
- Dashboard top telemetry row shows CPU, RAM, GPU (when available), Disk, and Bandwidth.

**Dashboard UX**
- Guest (not logged in) app cards are compacted to avoid oversized rows.

---

## v1.4.1.0 — Proxmox card, host telemetry, and privacy settings

**2026-06-20** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.4.1.0) · [Compare v1.4.0.0…v1.4.1.0](https://github.com/boubli/AMUD-Dashboard/compare/v1.4.0.0...v1.4.1.0)

**Proxmox**
- Jellyfin-style Proxmox stream card; status from host agent (no false OFFLINE on self-signed PVE UI).
- Card shown under its app category, not hidden by Media filter.

**Telemetry**
- Per-app CPU/RAM restored on dashboard cards.
- Host card: CPU model, cores, temp, CPU/Memory sparklines.
- GPU card when `nvidia-smi` is on the Proxmox host agent.

**Settings**
- Logo field no longer shows `{{app_logo}}` when empty.
- **Privacy & Access** tab for guest telemetry and TLS (moved from Support / Donation).

**Upgrade:** `curl -sSL …/update-amud.sh | bash` — then `systemctl restart amud-agent` on the PVE host for GPU.

---

## v1.4.0.0 — Security, audit log, and UI overhaul

**2026-06-20** · [Release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.4.0.0) · [Compare v1.3.7.3…v1.4.0.0](https://github.com/boubli/AMUD-Dashboard/compare/v1.3.7.3...v1.4.0.0)

Big one. Mostly about making AMUD safer to run in production, easier to trust, and nicer to look at every day.

**Audit log**
- Admin actions recorded in SQLite (logins, settings, backups, user management).
- New **Audit** tab in Settings.
- `audit_log` table auto-created on upgraded Proxmox databases.

**Security**
- Webhook URL masking and SSRF filtering (no localhost/metadata targets).
- Health checks block loopback while allowing homelab RFC1918 ranges.
- Branding fields HTML-escaped in dashboard templates.
- Settings UI uses DOM APIs instead of `innerHTML` for dynamic rows.
- Documented `.env.example` for proxy trust, secure cookies, and secrets key.

**Database**
- SQLite WAL mode + foreign keys enforced.

**UI**
- Proxmox-style status chips and badge styling.
- Accessibility pass (buttons, labels, contrast).
- `globalThis` in service worker and login page.

**Tooling**
- SonarCloud quality gate green (Security / Reliability / Maintainability A).
- GitHub Actions pinned to commit SHAs.
- Install scripts cleaned up (`[[` bash, shared constants).

**Upgrade:** `./update-amud.sh` — no database wipe required.

---

## v1.3.7.x — Auto-update and Proxmox polish

**2026-06-20**

| Version | Highlights |
|---------|------------|
| **v1.3.7.3** | Settings sidebar scroll fix on smaller screens |
| **v1.3.7.2** | Auto-updater fix; GitHub Actions runner pinned to ubuntu-22.04 (GLIBC mismatch on older LXCs) |
| **v1.3.7.1** | Auto-update system and in-app version notifications |
| **v1.3.7** | Wake-on-LAN decoupled from apps; Proxmox host/container UDS permission fixes |

---

## v1.3.6 — 2026-06-18

Stability and deployment improvements. See [release tag](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.3.6).

---

## v1.3.1.5 — 2026-06-14

Maintenance release. Docs and install script refinements.

---

## v1.3.0.0 — 2026-06-10

Feature release: expanded integrations, settings UX, and homelab workflow improvements.

---

## v1.2.0.0 — 2026-06-09

Architecture and telemetry pipeline updates.

---

## v1.1.0.0 — 2026-06-06

**Fully static architecture and security hardening**

- Musl-static binaries and scratch-based Docker image.
- Argon2id password hashing (legacy SHA-256 migrated on login).
- Encrypted app keys at rest; authenticated integration handlers.

---

## v1.0.0 — 2026-06-03

First public release. Rust server + agent, SQLite config, Proxmox/Docker telemetry, zero-YAML UI.

---

## How to stay updated

1. Watch **Releases** on [GitHub](https://github.com/boubli/AMUD-Dashboard/releases).
2. Enable update notifications in **Settings → System** (v1.3.7.1+).
3. Run `./update-amud.sh` on Proxmox LXCs or pull the latest Docker image.

For planned work, see the [Roadmap](./roadmap).
