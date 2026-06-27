---
title: Migrate from Homarr
---

# Migrate from Homarr

Homarr stores boards in its own database. AMUD provides a compatible **feature set** via UI configuration.

## Recommended approach

1. Export your Homarr app list (screenshot or manual CSV) — there is no universal export format yet
2. Use **Homepage import** if you also have a `services.yaml` from Homepage
3. Use **Discover Docker** on the AMUD host to pull labelled containers
4. Re-create integrations in **Add App** — credentials are encrypted at rest in AMUD

## Feature mapping

| Homarr | AMUD |
|--------|------|
| Boards | Dashboard boards (`/api/boards`) |
| Calendar widget | Widget type `calendar_ics` |
| OIDC / LDAP | Settings → Security |
| Integration widgets | Per-app integration dropdown |
| Custom widgets | `custom_api` integration or HTML widget |

## Homarr-only integrations

AMUD adds long-tail types over time (ntfy, Coolify, Aria2, Speedtest Tracker, etc.). Use **Custom API** until a native card exists.
