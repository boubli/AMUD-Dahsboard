---
sidebar_position: 2
title: Features
---

# Features

Complete inventory of what AMUD Dashboard ships today (v1.5.2.1). Everything below is implemented in the compiled Rust binaries — no YAML files, no Node.js runtime.

---

## Core dashboard

| Feature | Details |
|---------|---------|
| **Bento app grid** | Drag-and-drop reorder (admin), card spans (`1x1`, `2x1`, `1x2`), category filter tabs |
| **~2000 bundled logos** | Offline SVG library plus custom URL icons |
| **Light & dark mode** | System-wide theme toggle with 18 bundled CSS themes |
| **Video wallpapers** | `.mp4`, `.webm`, `.ogg` background support |
| **Weather widget** | Open-Meteo via latitude/longitude in Settings |
| **Live settings preview** | Accent, glass blur/opacity, grid columns, wallpaper |
| **Dedicated `/feeds` page** | RSS-only view for guest-friendly news cards |
| **In-app update banner** | Native/Proxmox installs check GitHub Releases automatically |
| **PWA service worker** | Basic static asset caching (offline shell polish is on the roadmap) |

---

## Telemetry & agent (`amud-agent`)

| Feature | Details |
|---------|---------|
| **Host metrics** | CPU (model, cores, %, temperature), RAM, disk, GPU via `nvidia-smi` |
| **Network bandwidth** | Internal and external interface throughput (Mbit/s) |
| **Proxmox LXC** | Native HTTPS REST API — no `pvesh` subprocesses |
| **Docker monitoring** | Container list + CPU/RAM (`AMUD_DOCKER=1`, Unix only) |
| **LXC power controls** | Start, stop, reboot, shutdown from dashboard cards |
| **Docker power controls** | Start, stop, restart from dashboard cards |
| **Live WebSocket stream** | `/ws` pushes telemetry; role-filtered payloads for Admin vs Guest |
| **App health badges** | ONLINE / OFFLINE / BLOCKED with latency on URL checks |
| **IPC authentication** | Challenge-response with shared `AMUD_AGENT_SECRET` |

---

## App card integrations

Configure per app under **Add/Edit App → Integration**. Data loads when the card renders (on-demand fetch, not background polling).

| Integration | Data shown | Admin actions | Guest visible |
|-------------|-----------|---------------|---------------|
| **Pi-hole** | Ads blocked today, status | Disable 5 min | No |
| **AdGuard Home** | Blocked today, protection state | Disable 5 min | No |
| **Radarr** | Queue size | — | No |
| **Sonarr** | Queue size | — | No |
| **Overseerr** | Pending media requests | — | No |
| **Jellyseerr** | Pending media requests | — | No |
| **RSS / Atom** | Top 3 headlines | — | **Yes** |

RSS feeds are managed under **Settings → RSS Feeds** (not the Add App modal).

---

## Media stream badges

Configured under **Settings → Integrations**. Polls every few seconds when matching app cards exist.

| Service | Badge |
|---------|-------|
| **Jellyfin** (incl. Emby) | Now playing title, progress bar, multi-stream summary |
| **Plex** | Same |

---

## Smart home

| Feature | Details |
|---------|---------|
| **Home Assistant** | Lights on, switches on, average temperature on a card named exactly `Home Assistant` |
| **Template API** | Lightweight `POST /api/template` with `/api/states` fallback |

---

## Auth & users

| Feature | Details |
|---------|---------|
| **Roles** | Admin (full control) and Guest (read-only dashboard) |
| **Argon2id passwords** | Legacy SHA-256 hashes rehashed on login |
| **Bootstrap admin** | Random one-time password printed on first boot |
| **Sessions** | 24h HttpOnly cookies, CSRF tokens, login rate limiting |
| **Encrypted secrets** | Integration API keys stored with ChaCha20-Poly1305 at rest |
| **Security headers** | CSP nonce, X-Frame-Options, HSTS-ready via reverse proxy |

---

## Webhooks & notifications

| Event | Payload formats |
|-------|-----------------|
| `container_started` / `container_stopped` | Discord embed, Telegram HTML, generic JSON |
| `agent_connected` / `agent_disconnected` | Same |

Manage under **Settings → Webhooks**. Test button included. SSRF protection blocks loopback and private IPs (homelab LAN targets may need a public relay).

---

## Wake-on-LAN

Dedicated **Settings → Wake-on-LAN** device list (decoupled from app cards). Send UDP magic packets from dashboard cards or the WOL manager.

---

## Backup & restore

| Action | Endpoint |
|--------|----------|
| Export `amud.db` | Settings → Backup |
| Import `amud.db` | Validates SQLite header, creates `.bak`, restarts process |

Back up `.amud-secrets-key` alongside the database when using encrypted integration tokens.

---

## Audit log

24 action types tracked (login, app CRUD, webhooks, backup, system updates, container actions, etc.). View and filter under **Settings → Audit**.

Not yet audited: card reorder, category CRUD, wake packets.

---

## System & deployment

| Install path | Docs |
|--------------|------|
| Proxmox LXC one-liner | [Proxmox](./installation/proxmox.md) |
| Docker Compose (server + agent) | [Docker](./installation/docker.md) |
| Bare-metal Linux | [Linux](./installation/linux.md) |
| Unraid Community Apps | [Unraid](./installation/unraid.md) |
| Hydrivax | [Hydrivax](./installation/hydrivax.md) |

In-app updater works on native/Proxmox installs only (Docker uses image pulls).

---

## See also

- [Configuration](./configuration.md) — appearance, media, Home Assistant, Proxmox
- [Roadmap](./roadmap.md) — what is next
- [Changelog](./changelog.md) — release history
