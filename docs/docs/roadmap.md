---
sidebar_position: 3
title: Roadmap
---

# Roadmap

This is where AMUD is headed. No dates — homelab project, ship when it's ready.

Items move between **Now**, **Next**, and **Later** as reality hits.

---

## Now (active focus)

- **Audit log expansion** — more admin actions tracked, export/filter in Settings.
- **Docs site** — changelog, roadmap, and install guides kept in sync with releases (you're reading this).
- **Quality bar** — keep SonarCloud green, `cargo audit` clean, pinned CI actions.

---

## Next

- **Guest vs Admin UX (In Progress)** — expanding clearer permission boundaries in the UI for read-only dashboard users (RSS feeds are already guest-friendly, continuing with more).
- **Backup/restore UX** — one-click export reminder and post-import validation feedback.
- **Custom themes** — document and polish the CSS variable system ([Themes](./themes)).
- **ARM64 release binaries** — official builds alongside x86_64 (manual builds exist today; CI support planned).
- **Webhook templates** — preset payloads for common notification services.

---

## Later (ideas, not commitments)

- **Multi-node agent** — one dashboard, several Proxmox hosts reporting in.
- **API tokens** — scoped read-only tokens for external dashboards/scripts.
- **Plugin-style integrations** — community-contributed status providers without recompiling core.
- **Mobile PWA polish** — offline shell, install prompts, tighter touch targets.

---

## Won't do (on purpose)

- **YAML as primary config** — SQLite + UI is the whole point.
- **Node.js/PHP runtime dependency** — compiled Rust only.
- **SaaS / cloud-hosted AMUD** — self-hosted homelab tool, stays that way.

---

## Request a feature

Open a [GitHub Discussion](https://github.com/boubli/AMUD-Dashboard/discussions) or an [Issue](https://github.com/boubli/AMUD-Dashboard/issues) with the `enhancement` label. If it fits the homelab/self-hosted scope, it lands here.

**Shipped something?** It's in the [Changelog](./changelog).
