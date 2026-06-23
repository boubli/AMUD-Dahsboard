---
sidebar_position: 3
title: Roadmap
---

# Roadmap

This is where AMUD is headed. No dates — homelab project, ship when it's ready.

Items move between **Now**, **Next**, and **Later** as reality hits. Recently shipped work lives in the [Changelog](./changelog).

---

## Recently shipped (v1.5.x)

- **Feeds redesign** — news cards, categories, hero headline, drag reorder (v1.5.4.0)
- **RSS management UI** — Settings CRUD and `/feeds` page (v1.5.3.0)
- **Webhook LAN toggle** + audit expansion + backup validate (v1.5.3.0)
- **RSS / Atom feeds** — top 3 headlines on app cards; guest-readable (v1.5.2.0)
- **Overseerr & Jellyseerr** — pending media request counts (v1.5.2.1)
- **Prowlarr, Uptime Kuma, Cloudflare Tunnel, Peanut** — app card integrations (v1.5.5.0)
- **Per-app CPU/RAM toggle** + guest ONLINE/OFFLINE + login branding (v1.5.5.0)
- **Host telemetry mapping** — network interfaces and disk mounts in Settings (v1.5.5.0)
- **Unraid permissions docs** + AdGuard credential clarity (v1.5.5.0)
- **18 bundled offline themes** + theme gallery (v1.5.1.0)
- **Drag-and-drop layout**, bento spans, light mode, video wallpapers (v1.5.0.0)
- **Audit log** with settings tab and startup schema checks (v1.4.0.0+)
- **Guest compact cards** and optional public host telemetry toggle (v1.4.x)
- **In-app updater** for native/Proxmox installs (v1.3.7+)

---

## Now (active focus)

- **Quality bar** — keep SonarCloud green, `cargo audit` clean, pinned CI actions.
- **ARM64 release binaries** — official builds alongside x86_64.
- **Webhook templates** — richer preset payloads beyond URL shortcuts.

---

## Next

- **Backup/restore UX** — export scheduling reminders (basic validate/counts shipped in v1.5.3.0).

---

## Later (ideas, not commitments)

- **Multi-node agent** — one dashboard, several Proxmox hosts reporting in.
- **API tokens** — scoped read-only tokens for external dashboards/scripts.
- **Plugin-style integrations** — community-contributed status providers without recompiling core.
- **Mobile PWA polish** — offline shell, install prompts, tighter touch targets.
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
