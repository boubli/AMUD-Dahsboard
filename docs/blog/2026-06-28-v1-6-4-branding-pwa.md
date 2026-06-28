---
slug: v1-6-4-branding-pwa
title: "v1.6.4 — Branding logo fix & PWA mobile polish"
authors: [boubli]
tags: [release, pwa]
---

Custom logos uploaded in **Settings → Dashboard Logo** now show for **everyone** — guests, the login page, browser favicon, and installed PWA icons. This release also tightens mobile layout across all themes.

<!--truncate-->

## Branding logo fix

Before v1.6.4, `/uploads/…` required an admin session. Logged-in users saw your custom gamepad (or PNG); guests and the login page got an empty box and a broken favicon.

**Now:**

- `GET /uploads/{file}` is publicly readable (upload stays admin-only)
- Dashboard, login, and settings render `<img>` when a logo is set
- Favicon, Apple touch icon, and `/manifest.webmanifest` use the same URL

## PWA mobile polish

- Hamburger menu uses a **fixed panel** below the topbar — no more covering the greeting
- Tap **outside** the menu to close it
- Centered logo/title, telemetry 2-per-row, expandable search icon
- Service worker cache **v26** — hard-refresh or clear PWA cache after upgrade

## Upgrade

```bash
curl -sSL https://github.com/boubli/AMUD-Dashboard/releases/download/v1.6.4/update-amud.sh | bash
```

Docker: `docker compose pull && docker compose up -d --force-recreate`

Full notes: [GitHub Release v1.6.4](https://github.com/boubli/AMUD-Dashboard/releases/tag/v1.6.4) · [Changelog](/docs/changelog)
