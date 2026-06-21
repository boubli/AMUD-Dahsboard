---
slug: sqlite-wal-not-postgres
title: Why AMUD Uses SQLite Instead of Postgres (Yes, Really)
authors: [boubli]
tags: [rust, homelab]
description: Embedded DB for embedded dashboard. WAL mode, single-file backup, no extra container for config storage.
---

"Why not Postgres?"

Because then I'd need to run Postgres.

AMUD is a dashboard. Config storage is a few hundred KB of apps, settings, and encrypted tokens. Running a whole separate database server for that is like buying a warehouse to store a shoebox.

## SQLite with WAL

Write-Ahead Logging lets readers (the web UI loading settings) coexist with writers (you saving a new app card) without the lock contention that makes people hate SQLite in web apps from 2010.

For a single-tenant homelab dashboard with one admin clicking settings occasionally? Perfect fit.

## Operational wins

- **Zero extra container** — `amud.db` sits next to the binary
- **Backup** — `cp amud.db backup.db`
- **Restore** — swap file, restart
- **Inspect** — `sqlite3 amud.db` when debugging

I've done all of these at 1am. Postgres would add connection strings, another service to monitor, another thing to backup separately.

## When I'd reach for Postgres

Multi-tenant SaaS. Hundreds of concurrent writers. Horizontal scaling. AMUD is explicitly none of those — see roadmap "won't do SaaS."

Homelab tool. Homelab database. Match the complexity to the problem.

[/docs/intro](/docs/intro)
