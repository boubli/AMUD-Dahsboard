---
sidebar_position: 2
title: Changelog
---

# Changelog

Every stable release is tagged on GitHub with binaries, checksums, and install scripts.

**Download latest:** [GitHub Releases](https://github.com/boubli/AMUD-Dashboard/releases/latest)

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
