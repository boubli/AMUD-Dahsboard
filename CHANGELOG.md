# Changelog

All notable changes to AMUD Dashboard are documented here and on the docs site.

**Full history (readable):** https://boubli.github.io/AMUD-Dashboard/docs/changelog  
**Latest release + binaries:** https://github.com/boubli/AMUD-Dashboard/releases/latest  
**Roadmap:** https://boubli.github.io/AMUD-Dashboard/docs/roadmap

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
