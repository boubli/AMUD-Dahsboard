# Changelog

All notable changes to AMUD Dashboard are documented here and on the docs site.

**Full history (readable):** https://boubli.github.io/AMUD-Dashboard/docs/changelog  
**Latest release + binaries:** https://github.com/boubli/AMUD-Dashboard/releases/latest  
**Roadmap:** https://boubli.github.io/AMUD-Dashboard/docs/roadmap

---

## [v1.4.2.1](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.4.2.1) — 2026-06-20

Telemetry and host-app status polish with AMUD-native UI behavior.

### Fixed
- Host-based apps (e.g. Beszel, Filebrowser) now map correctly even when they are not LXC containers
- Proxmox card status now follows host agent connectivity (no false OFFLINE from URL check)
- Dashboard app matching improved via normalized aliases and URL-derived tokens

### Improved
- Dashboard top telemetry row now clearly shows CPU, RAM, GPU (when available), Disk, and Bandwidth
- `/telemetry` page now includes richer host details and live services table while keeping AMUD visual style
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
