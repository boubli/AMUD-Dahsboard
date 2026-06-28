---
sidebar_position: 3
title: Roadmap
---

# Roadmap

This is where AMUD is headed. No dates — homelab project, ship when it's ready.

Items move between **Now**, **Next**, and **Later** as reality hits. Recently shipped work lives in the [Changelog](./changelog).

---

## Recently shipped (v1.6.4)

- **Branding logo fix** — custom logos on login, guest dashboard, favicon, and PWA (public `/uploads/` read)
- **PWA mobile polish** — hero/menu overlap, centered topbar, telemetry grid, expandable search
- **Custom logo → manifest** — Settings Dashboard Logo drives favicon and Apple touch icon

## Recently shipped (v1.6.3)

- **ARM64 release binaries** — `amud-server-arm64` / `amud-agent-arm64` alongside amd64
- **Multi-arch Docker** — `linux/amd64` images on Docker Hub (ARM64: use native release binaries)
- **GitHub Pages** — auto-deploy restored; homepage hero carousel + GIF demos
- **PWA / mobile polish** — install banner, offline shell, responsive topbar menu

## Recently shipped (v1.6.0)

- **Universal parity** — integration cache, Homepage/Homarr import, Custom API, LDAP, boards, manifest API
- **Plex/Jellyfin cards** + **\*arr calendar** widget + release trackers
- **40+ long-tail integrations** — Autobrr, Gotify, Prometheus, OMV, Kubernetes, Healthchecks, Gatus, …
- **Multi-node agent** — `node_tag` / `AMUD_NODE_TAG` telemetry labels
- **Comparison & migration docs** — [comparison](./comparison.md), [Homepage](./migration/homepage.md), [Homarr](./migration/homarr.md)

## Recently shipped (v1.5.x)

- **Feeds redesign** — news cards, categories, hero headline, drag reorder (v1.5.4.0)
- **RSS management UI** — Settings CRUD and `/feeds` page (v1.5.3.0)
- **Webhook LAN toggle** + audit expansion + backup validate (v1.5.3.0)
- **RSS / Atom feeds** — top 3 headlines on app cards; guest-readable (v1.5.2.0)
- **Overseerr & Jellyseerr** — pending media request counts (v1.5.2.1)
- **Lidarr, Readarr, Whisparr** — complete Servarr suite app card integrations (queue, library stats, disk, version)
- **13 download & media integrations** — SABnzbd, NZBGet, Transmission, Jackett, Tautulli, Audiobookshelf, Immich, Tdarr, Maintainerr, Frigate (v1.5.6.4)
- **Theme system overhaul** — wallpaper overlay slider; glass params work on all 37 themes; no card hover (v1.5.6.4)
- **Per-app CPU/RAM toggle** + guest ONLINE/OFFLINE + login branding (v1.5.5.0)
- **Host telemetry mapping** — network interfaces and disk mounts in Settings (v1.5.5.0)
- **Unraid permissions docs** + AdGuard credential clarity (v1.5.5.0)
- **37 bundled offline themes** + visual Theme Gallery in Settings (v1.5.6.3); 18 original offline pack (v1.5.1.0)
- **Drag-and-drop layout**, bento spans, light mode, video wallpapers (v1.5.0.0)
- **Audit log** with settings tab and startup schema checks (v1.4.0.0+)
- **Guest compact cards** and optional public host telemetry toggle (v1.4.x)
- **In-app updater** for native/Proxmox installs (v1.3.7+)

---

## Now (active focus)

- **Quality bar** — keep SonarCloud green, `cargo audit` clean, CI integration tests
- **Remaining Homepage widget catalog** — niche services via registry batches + Custom API

---
## Next

- **Docusaurus locale packs** — translate docs site (READMEs already in 11 languages)
- **Backup/restore UX** — export scheduling reminders

---

## Later (ideas, not commitments)

- **Multi-node agent** — one dashboard, several Proxmox hosts reporting in.
- **API tokens** — scoped read-only tokens for external dashboards/scripts (shipped; expand scopes).
- **Per-integration setup wizards** — guided Pi-hole, *arr, and DNS blocker configuration in Settings.

---

## Won't do (on purpose)

- **YAML as primary config** — SQLite + UI is the whole point.
- **Node.js/PHP runtime dependency** — compiled Rust only.
- **SaaS / cloud-hosted AMUD** — self-hosted homelab tool, stays that way.

---

## Request a feature

Open a [GitHub Issue](https://github.com/boubli/AMUD-Dashboard/issues) with the `enhancement` label for **bugs and feature requests** (preferred — tracked per release). Use [Discussions](https://github.com/boubli/AMUD-Dashboard/discussions) for questions and screenshots. If it fits the homelab/self-hosted scope, it lands here.

**Shipped something?** It's in the [Changelog](./changelog).
