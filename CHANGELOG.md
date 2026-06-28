# Changelog

All notable changes to AMUD Dashboard are documented here and on the docs site.

**Full history (readable):** https://boubli.github.io/AMUD-Dashboard/docs/changelog  
**Latest release + binaries:** https://github.com/boubli/AMUD-Dashboard/releases/latest  
**Roadmap:** https://boubli.github.io/AMUD-Dashboard/docs/roadmap

---

## [v1.6.4.1](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.4.1) — 2026-06-28

Hotfix for v1.6.4 default logo showing raw template syntax instead of AMUD PNG.

### Fixed
- **Default logo** — `apply_app_logo_template` uses span-based `{{if app_logo}}…{{end}}` replacement so empty logo shows CSS default `/static/AMUD-logo.png`
- **Custom logos** — still inject `<img>` when Settings Dashboard Logo is set

---

## [v1.6.4](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.4) — 2026-06-28

Branding logo fix for guests/login, PWA mobile polish, custom logo favicon/manifest.

### Fixed
- **Custom logo empty for guests / login** — public `GET /uploads/` for branding images; upload stays admin-only
- **Logo markup** — `<img>` on dashboard, login, settings; favicon and PWA manifest stay in sync
- **Mobile overflow menu** — fixed panel below topbar; outside-click close; no hero overlap

### Improved
- **PWA mobile UI** — centered logo/title, hero layout, weather card, telemetry grid, expandable search
- **Custom logo → PWA** — Settings Dashboard Logo drives favicon, Apple touch icon, manifest
- **Service worker** — `sw.js` v26

### Maintainer
- CI hardening, Docker Hub overview sync, README release list trim

---

## [v1.6.3](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.3) — 2026-06-25

ARM64 binaries, multi-arch Docker, GitHub Pages hero + GIFs, PWA/mobile polish.

### Added
- **ARM64 release binaries** — `amud-server-arm64` and `amud-agent-arm64` alongside amd64 builds
- **Multi-arch Docker** — `linux/amd64` and `linux/arm64` images on Docker Hub
- **GitHub Pages deploy** — restored workflow; rotating homepage hero carousel + GIF demo section
- **PWA install UX** — manifest shortcuts, install banner, offline shell (`sw.js` v23)

### Improved
- **Mobile topbar** — overflow actions collapse into a touch-friendly menu at ≤768px
- **`update-amud.sh` + in-app updater** — architecture-aware release asset selection

---

## [v1.6.2](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.2) — 2026-06-27

Offline bundled themes — replaces withdrawn v1.6.1.

### Added
- **Manifest v5** — icons, wallpapers, and gallery previews in `ui.tar.gz` (`/static/themes/…`)
- **WebP** wallpapers and preview thumbnails
- **Per-profile icon libraries** (8 art styles across 35 themes)
- **`theme-layouts/`** and extracted **`theme-picker.js`**

### Fixed
- Theme gallery black boxes when CDN unreachable
- Theme save + local wallpaper/icon apply via `theme-engine.js`

### Note
- **v1.6.1 withdrawn** — do not use; upgrade to v1.6.2

---

## [v1.6.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.0) — 2026-06-25

Universal dashboard parity — Homepage/Homarr import, integration cache, LDAP, Custom API, long-tail integrations, Plex/Jellyfin cards, *arr calendar, release trackers, multi-node agent.

### Added
- **Integration cache** + poll coordinator for scalable card refresh
- **Homepage YAML** and **Homarr JSON** importers (Settings → Integrations)
- **Homepage Docker labels** in agent discovery
- **Custom API**, **LDAP**, **OIDC admin groups**, **per-user boards**
- **Plex/Jellyfin** per-app cards, **\*arr calendar** widget, **release trackers**
- **40+ integrations** (Autobrr, Gotify, Prometheus, OMV, Kubernetes, …)
- **Server-driven integration manifest** for Add/Edit app UI
- **Multi-node agent** `node_tag` support
- Docs: comparison matrix, migration guides, architecture updates

### Improved
- Generic tier-2 integration card template
- Promoted health-only services to full cards (Gitea, GitLab, Jenkins, MinIO, Kopia, Headscale, Stash, …)

---

## [v1.5.6.4](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.6.4) — 2026-06-25

Theme system fix, 13 new integrations, filled integration cards, no card hover.

### Added
- **Servarr expansion** — Lidarr, Readarr, Whisparr app card integrations
- **Top 10 roadmap integrations** — SABnzbd, NZBGet, Transmission, Jackett, Tautulli, Audiobookshelf, Immich, Tdarr, Maintainerr, Frigate
- **Wallpaper overlay strength** — new slider in Glass & Layout Parameters (0 = clear wallpaper)

### Fixed
- **All 36 bundled themes** — glass opacity, blur, radius, and wallpaper tint respect user sliders (no hardcoded overlays)
- **Integration cards** — 8 metric cells always filled (CPU/RAM show `—` when agent off)
- **App card hover** — removed lift/glow and metric layer swap
- **Guest dashboard** — compact cards no longer leave large empty row gaps (grid rows use `auto` height)

### Improved
- `theme-guards.css` cascade enforcement; theme audit matrix for v1.5.6.4

---

## [v1.5.6.3](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.6.3) — 2026-06-26

Theme gallery overhaul, 37 offline themes, guest compact cards, and docs/CI polish.

### Added
- **37 bundled themes** — visual Theme Gallery in Settings → Appearance; manifest v3 with previews and wallpapers
- **18 new theme packs** — Nature, Terminal, Feminine, Variety
- **Vendored wallpapers** — unique Unsplash/Pexels photos per theme at `/static/themes/wallpapers/`
- **`active_theme_id`** setting for theme picker persistence

### Fixed
- **Guest dashboard** — compact 1×1 cards (icon, name, status only)
- **RSS settings** — add-feed modal; category table column layout
- **CI** — clippy fixes; Docusaurus build in GitHub Actions

### Improved
- Filled integration cards (6-cell grid, richer stats, 30s refresh) — see v1.5.6.2
- GitHub Pages Theme Gallery category filters; `fetch-theme-wallpapers.py`

---

## [v1.5.6.2](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.6.2) — 2026-06-26

Filled integration cards and expanded per-service API stats.

### Fixed
- Integration cards fill tall 1×2 layout with unified 6-cell metrics grid
- Expanded API stats (Radarr, Sonarr, Prowlarr, qBittorrent, Pi-hole, etc.)
- Integration data refreshes every 30s on visible cards

---

## [v1.5.5.6](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.6) — 2026-06-25

Dashboard UX and telemetry mapping polish release.

### Fixed
- **App card navigation** — clicking anywhere on a card now opens it, while preserving embedded controls and buttons
- **Integration metrics visibility** — integration stats are now visible by default (no hover dependency)
- **Telemetry mapping normalization** — network interface and disk mount lists are canonicalized and deduplicated
- **Settings guidance** — telemetry field help text clarifies normalization and duplicate-handling behavior

---

## [v1.5.5.5](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.5) — 2026-06-24

Hotfix: restore agent telemetry broken in v1.5.5.4.

### Fixed
- **Agent telemetry** — Docker discover handler no longer blocks GPU, host stats, container controls, or container status
- **Status badges** — ONLINE/OFFLINE/RUNNING text restored (latency in tooltip)
- **Search toggle UI**, **accent preview**, **GPU row layout**, service worker cache bump

---

## [v1.5.5.4](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.4) — 2026-06-24

Dashboard parity batch: search, layout, widgets, Docker import, OIDC, API tokens, kiosk/share, iframe embeds.

### Added
- **Global search** — app filter + web search toggle; keyboard shortcuts (`Ctrl+K`, `/`, `1`–`9`, `?`)
- **Dashboard layout** — tabs vs collapsible category sections (Settings → Appearance)
- **Status page** — `/status` and `/api/status`
- **Dashboard widgets** — Settings → Widgets CRUD
- **Per-app guest visibility** — hide individual apps from guest sessions
- **Docker discovery** — agent label scan + Settings import UI
- **OIDC SSO** — Security settings + login SSO button
- **API tokens** — Bearer auth on read-only API routes
- **Kiosk mode** + **share links** (`/s/:token`)
- **Iframe embeds** — per-app embed mode, `/embed/:id`, CSP frame-src allowlist

### Fixed
- CI: clippy, cargo audit (oauth2 dep), SonarCloud gate on parity UI code

---

## [v1.5.5.3](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.3) — 2026-06-24

Fix broken integration hover from v1.5.5.2; admin settings redirect.

### Fixed
- **Integration hover** — fixed metrics slot swap inside the card (no overlap on neighbors)
- **Admin settings** — redirect to `/login` when not signed in as admin

### Improved (carried from v1.5.5.2)
- Docker `:latest` version display, Docker auto-enable controls, RSS auto-favicons

---

## [v1.5.5.2](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.2) — 2026-06-24

**Superseded by v1.5.5.3** — integration hover drawer overlapped other cards. Do not use.


## [v1.5.5.1](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.1) — 2026-06-24

qBittorrent and Bazarr integrations plus compact hover stats on app cards.

### Added
- **qBittorrent** and **Bazarr** app card integrations

### Improved
- **Integration stats on hover** — cards stay short; hover to see Prowlarr/*arr/qBit/Bazarr metrics
- **Telemetry mapping docs** — how to find interface names and disk mounts (linked from Settings)

---

## [v1.5.5.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.5.0) — 2026-06-23

Community integrations, telemetry mapping, and Unraid polish (Inch feedback batch).

### Added
- **Prowlarr, Uptime Kuma, Cloudflare Tunnel, Peanut (UPS)** app card integrations
- **Per-app CPU/RAM toggle** on Add/Edit App
- **Host telemetry mapping** — external/internal network interfaces and disk mount points (Settings → Privacy & Access)
- **Guest ONLINE/OFFLINE** container status; **login page branding**

### Improved
- **AdGuard Home** — Basic auth clarity, auto Base64 for raw credentials, correct stats field, safer widget fetch
- **Unraid docs** — permission fix and Docker password reset
- **GitHub Issues** guidance for bugs and features

---

## [v1.5.4.3](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.4.3) — 2026-06-23

Hardening fix when live telemetry still failed after v1.5.4.2 (cached HTML / service worker).

### Fixed
- **Live dashboard scripts** — clock, WebSocket, GPU card, telemetry bars, and status badges moved to versioned `/static/dashboard-live.js`
- **Service worker** — do not cache HTML navigations; cache bump clears stale dashboard JS

---

## [v1.5.4.2](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.4.2) — 2026-06-23

Critical fix: live telemetry and status badges after feeds UI refactor.

### Fixed
- **Dashboard JavaScript crash** in `updateClock()` — WebSocket, GPU card, CPU/RAM bars, and CHECKING badges work again

---

## [v1.5.4.1](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.4.1) — 2026-06-23

Audit log migration fix for upgraded databases.

### Fixed
- **Audit log schema** — auto-adds missing `username` column on startup; backfills from legacy `user` column when present

---

## [v1.5.4.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.4.0) — 2026-06-23

Feeds page polish: hero headline, category colors, and drag reorder.

### Added
- **Featured headline hero** on `/feeds` — top story from the first feed in sort order
- **RSS feed reorder** — drag rows in Settings → RSS Feeds; order drives `/feeds` layout

### Improved
- **Feed category tabs** — guest-visible accent colors per category on `/feeds`
- **Feeds UX** — news-style cards, preset icons, and category empty states (phases 1–3)

---

## [v1.5.3.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.3.0) — 2026-06-23

RSS management UI, homelab polish, and audit expansion.

### Added
- **RSS Feeds settings tab** — add, edit, and delete RSS/Atom feeds from Settings
- **Dedicated `/feeds` page** — RSS-only view with nav link for all visitors
- **RSS in Add/Edit App modal** — create feed cards from the dashboard
- **Webhook private LAN toggle** — allow homelab webhooks to `192.168.x.x` (Settings → Privacy)
- **Backup validate endpoint** — preview app/user/webhook counts before restore
- **Audit log filter** — search and filter by action in Settings
- **Webhook quick presets** — Discord, Telegram, and generic JSON shortcuts

### Improved
- **Audit logging** — wake-on-LAN, card reorder, and category CRUD now recorded
- **Backup tab** — last export timestamp, secrets key reminder, import confirmation with counts
- **Home Assistant TLS** — respects “Accept invalid TLS certificates” setting
- **Docs** — full features page, updated roadmap and homepage

### Fixed
- Guest users can discover the Feeds page from the topbar
- Feeds page shows a clear empty state when no RSS apps exist

---

## [v1.5.2.1](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.2.1) — 2026-06-23

Overseerr/Jellyseerr integrations, multi-language READMEs, and Docker build optimizations.

### Added
- **Overseerr & Jellyseerr Integrations** — Native API support to display live pending media request counts on dashboard cards.
- **Multi-Language Support** — README documentation in Arabic, Hindi, Spanish, French, German, Italian, Portuguese, Russian, Chinese, Japanese, and Korean.

### Improved
- **Docker Build Optimizations** — Overhauled `.dockerignore` to reduce image size and build times.

---

## [v1.5.2.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.2.0) — 2026-06-23

Native RSS and Atom feed integration for dashboard app cards.

### Added
- **RSS Integration**: Add any valid RSS or Atom feed URL to an app to display the top 3 latest headlines on its card.
- **Guest Visibility**: RSS integration data is explicitly permitted for unauthenticated guest users, keeping your dashboard lively while keeping sensitive integrations (like Pi-hole or Proxmox) locked down to admins.
- **Feed Parsing**: Powered by `feed-rs` for robust cross-format feed compatibility.

---

## [v1.5.1.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.1.0) — 2026-06-21

Appearance cleanup, offline bundled themes, audit log fixes, and in-app updater repair.

### Added
- Bundled theme picker in Settings (18 CSS files + manifest, works offline)
- Six new advanced themes: Terminal Phosphor, Vaporwave Grid, Blueprint Tech, Luxury Gold, Holographic Prism, Brutalist Mono
- Theme gallery **Download CSS** button; per-file theme definitions in docs
- Pre-commit `cargo fmt` hook, `check-rust` scripts, and `rustfmt.toml`

### Improved
- Appearance tab simplified — overlay tint presets removed; quick colors + sliders + Custom CSS
- Live preview scoped to mini scene; Custom CSS updates in real time
- Audit log schema on startup, 503 on read failure, settings-save audit entry
- Troubleshooting docs for audit log

### Fixed
- Proxmox in-app updater SHA256 lookup (basename matching in `SHA256SUMS`)
- Release workflow checksum layout aligned with updater
- CI rustfmt drift; Sonar theme re-export

See [release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.1.0).

---

## [v1.5.0.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.0.0) — 2026-06-20

Major visual upgrade, drag-and-drop layout, light mode, and docs expansion.

### Added
- Cinematic dashboard animations (staggered cards, hover effects, ambient orbs, status pulse, greeting shimmer)
- Admin drag-and-drop card reorder with CSRF-protected API and SQLite `sort_order`
- Bento card spans (`1x1`, `2x1`, `1x2`) with mobile collapse
- Light mode theme (`data-theme="light"`) across dashboard and settings
- Video wallpaper support (`.mp4`, `.webm`, `.ogg`)
- Live settings preview (accent, glass, wallpaper, overlay, grid columns, accent glow)
- Docusaurus blog, FAQ, `llms.txt`, theme gallery, and JSON-LD SEO

### Improved
- Handle-only drag with category-filter guard, error toasts, rollback, and touch support
- Settings wallpaper layers and theme mode on page load (no flash)
- Light mode background polish and contrast fixes
- Docker CI resilience; Sonar maintainability fixes

### Fixed
- New apps get `MAX(sort_order)+1`; reorder validates full app set
- Add App modal preserves `card_span: '1x1'`
- Admin-only `drag.js` loading

See [release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.5.0.0).

---

## [v1.4.2.2](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.4.2.2) — 2026-06-20

Host telemetry and status polish on the main dashboard only.

### Removed
- Dedicated `/telemetry` page and **System** topbar button

### Fixed
- Host-based apps (e.g. Beszel, Filebrowser) now map correctly even when they are not LXC containers
- Proxmox card status now follows host agent connectivity (no false OFFLINE from URL check)
- Dashboard app matching improved via normalized aliases and URL-derived tokens

### Improved
- Dashboard top telemetry row shows CPU, RAM, GPU (when available), Disk, and Bandwidth
- Guest (not logged in) app cards render in compact mode to avoid oversized empty cards

---

## [v1.4.1.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.4.1.0) — 2026-06-20

Proxmox card, host telemetry (CPU model + GPU), and settings fixes. See [release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.4.1.0).

### Added
- Host CPU model, cores, temperature, and live CPU/Memory sparklines
- GPU telemetry via `nvidia-smi` on the Proxmox host agent
- **Privacy & Access** settings tab (guest telemetry + TLS)

### Fixed
- Proxmox false OFFLINE when agent is connected
- Per-app container CPU/RAM on dashboard cards
- WebSocket telemetry updates stopping on null DOM nodes
- Settings logo field showing `{{app_logo}}` placeholder

### Changed
- Proxmox Jellyfin-style stream card in app category section
- Guest/TLS settings moved out of Support / Donation

---

## [v1.4.0.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.4.0.0) — 2026-06-20

Security, audit log, and UI overhaul. See [release notes](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.4.0.0) for full details.

### Added
- Audit log (SQLite + Settings tab)
- `.env.example` for security-related env vars

### Changed
- SQLite WAL + foreign keys
- Dashboard status badges and Proxmox-style chips
- Settings DOM rendering via `admin.js` helpers
- SonarCloud quality gate passing; Actions pinned to SHAs

### Security
- Webhook SSRF filtering and URL masking
- HTML escaping on branding fields in templates

---

## [v1.3.7.3](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.3.7.3) — 2026-06-20

Settings sidebar scroll fix on smaller screens.

## [v1.3.7.2](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.3.7.2) — 2026-06-20

Auto-updater fix; CI runner ubuntu-22.04 for older LXC GLIBC.

## [v1.3.7.1](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.3.7.1) — 2026-06-20

Auto-update system and version notifications.

## [v1.3.7](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.3.7) — 2026-06-20

Wake-on-LAN decoupling; Proxmox UDS permission fixes.

## [v1.3.6](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.3.6) — 2026-06-18

Stability and deployment improvements.

## [v1.3.1.5](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.3.1.5) — 2026-06-14

Maintenance release.

## [v1.3.0.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.3.0.0) — 2026-06-10

Feature release.

## [v1.2.0.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.2.0.0) — 2026-06-09

Architecture and telemetry updates.

## [v1.1.0.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.1.0.0) — 2026-06-06

Fully static architecture and security hardening.

## [v1.0.0](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.0.0) — 2026-06-03

Initial public release.
