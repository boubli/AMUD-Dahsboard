---
sidebar_position: 3
title: Roadmap
---

# Roadmap

This is where AMUD is headed. No dates — homelab project, ship when it's ready.

Items move between **Now**, **Next**, and **Later** as reality hits. Recently shipped work lives in the [Changelog](./changelog).

---

## Recently shipped (v1.8.2)

- **Cross-device Taghawsa** — adaptive WebGL on phones and Windows; CSS fallback when WebGL is off
- **Mobile responsive fixes** — card wrap, metrics grid breakpoints, no horizontal scroll
- **Performance settings tab** — preset cards, live activity badge, polling moved out of Privacy
- **Audit + update history on LXC** — boot version tracking, Last Updated in System tab, legacy audit schema rebuild
- **Card drag reorder polish** — full ID persist, bigger handle, pointer drag, Appearance toggle
- **Settings cleanup** — removed Media streams block; Support icon fix on dark themes

## Recently shipped (v1.8.1)

- **Jellyfin posters + playback controls** — real artwork on the stream card, admin pause / resume / stop while streaming
- **Per-app media integration** — Jellyfin/Emby and Plex configured in Add App; automatic migration from legacy Settings
- **Instant status on reload** — SSR-embedded last-known statuses and metrics, localStorage hydration
- **Taghawsa theme** — WebGL animated gradient with mouse ripple (38 themes)
- **Container action fix** — retry + actionable errors for `client error (SendRequest)`

## Recently shipped (v1.8.0)

- **Smart idle / active runtime** — pollers and caches pause when nobody is on the dashboard; configurable grace period
- **Performance presets** — Light, Balanced, Active, or Custom in Settings → Performance & resources
- **Multi-node homelabs** — `agent_node_tag` and per-card `node_tag` for multi-host telemetry
- **Batch integrations API** — up to 50 cards per request; paginated `GET /api/apps?page=`
- **API token scopes** — selectable scopes when creating tokens
- **Backup reminders** — overdue export banner; streaming database export
- **Idle host alerts** — CPU / RAM / disk webhook thresholds in deep idle
- **Agent RAM diet** — idle agents skip unlinked containers; concurrent Docker stats

## Recently shipped (v1.7.7)

- **Performance** — configurable poll intervals; feeds toggle; per-theme light mode
- **Server** — idle poller gating, integration cache hot-reload

## Recently shipped (v1.7.5)

- **App cards** — container RAM in MB/GB; settings reorganized (Weather → Appearance, Proxmox → Integrations)
- **Web search** — Google, Bing, DuckDuckGo, YouTube, GitHub; °C/°F weather unit
- **Server memory** — WebSocket-gated telemetry, shared HTTP pollers, mimalloc

## Recently shipped (v1.7.4)

- **Integration picker** — custom dropdown with logos; fixes unreadable optgroup bars on Windows
- **Ollama + Open WebUI** — new AI & LLM integrations (model counts, health)

## Recently shipped (v1.7.3)

- **Integration dropdown fix** — CSP nonce on manifest loader; Add/Edit App catalog restored
- **Unraid Docker follow-up** — UID 99 in image, agent `--user 0`; `su-exec: setgroups` loop resolved ([#16](https://github.com/boubli/AMUD-Dashboard/issues/16))

## Recently shipped (v1.7.2)

- **Unraid Docker fix** — PUID 99 entrypoint, `.amud-secrets-key` permission denied resolved ([#16](https://github.com/boubli/AMUD-Dashboard/issues/16))

## Recently shipped (v1.7.1)

- **Mobile/PWA hotfix** — Settings hamburger fix; desktop guest cards restored; compact admin cards on phones (`sw.js` v29)

## Recently shipped (v1.7.0)

- **Integration catalog parity** — 130+ types in manifest; Homepage import aliases; Watchtower, Ombi, FileBrowser; manifest-driven Add App dropdown (`sw.js` v28)

## Recently shipped (v1.6.5)

- **Mobile PWA follow-ups** — guest 2-col grid, weather card sizing, admin header row, Settings hamburger menu (`sw.js` v27)

## Recently shipped (v1.6.4.1)

- **Default logo hotfix** — span-based `{{if app_logo}}` template fix when no custom logo uploaded

## Recently shipped (v1.6.4)

- **Branding logo fix** — custom logos on login, guest dashboard, favicon, and PWA (public `/uploads/` read)
- **PWA mobile polish** — hero/menu overlap, centered topbar, telemetry grid, expandable search
- **Custom logo → manifest** — Settings Dashboard Logo drives favicon and Apple touch icon

## Recently shipped (v1.6.3)

- **ARM64 release binaries** — `amud-server-arm64` / `amud-agent-arm64` alongside amd64
- **Docker CI** — tag builds publish `linux/amd64` only (~3 min); ARM64 via native release binaries + `update-amud.sh`
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
- **Net-new Homepage widgets** — watchtower/ombi/filebrowser batch shipped in v1.7.0; continue small batches (ombi-adjacent, file tools) or Custom API templates

---
## Next

- **Docusaurus locale packs** — translate docs site (READMEs already in 11 languages)
- **Backup/restore UX** — export scheduling reminders

---

## Later (ideas, not commitments)

- **Multi-node agent UI** — per-app `node_tag` shipped in v1.6.0; later = aggregate telemetry from several agents in one dashboard view
- **API tokens** — `read:apps` + `read:status` shipped; expand scopes for telemetry/feeds/webhooks
- **Per-integration setup wizards** — guided Pi-hole, *arr, and DNS blocker configuration in Settings
- **PWA offline polish** — richer offline shell and cache strategy beyond static asset precache

---

## Won't do (on purpose)

- **YAML as primary config** — SQLite + UI is the whole point.
- **Node.js/PHP runtime dependency** — compiled Rust only.
- **SaaS / cloud-hosted AMUD** — self-hosted homelab tool, stays that way.

---

## Request a feature

Open a [GitHub Issue](https://github.com/boubli/AMUD-Dashboard/issues) with the `enhancement` label for **bugs and feature requests** (preferred — tracked per release). Use [Discussions](https://github.com/boubli/AMUD-Dashboard/discussions) for questions and screenshots. If it fits the homelab/self-hosted scope, it lands here.

**Shipped something?** It's in the [Changelog](./changelog).
