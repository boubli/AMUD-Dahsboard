---
slug: zero-yaml-sqlite
title: I Replaced Homepage YAML with a SQLite UI and I'm Not Going Back
authors: [boubli]
tags: [homelab, selfhosted]
description: AMUD stores your whole dashboard in amud.db. Edit in the browser, backup one file, never debug YAML indentation again.
---

I lost a Saturday to a YAML space.

Homepage is a great project. Seriously. But when your dashboard config is a nested YAML tree living in a git repo, adding "one more service" turns into: SSH, edit, pray, `docker compose restart`, check logs, realize you used tabs instead of spaces, question life choices.

AMUD's whole pitch is **no YAML as primary config**. Everything lives in SQLite (`amud.db`), edited through the UI.

## What's actually in the database

- App cards, URLs, icons, LXC IDs
- Category tabs and layout
- Appearance settings (grid columns, accent, custom CSS)
- Integration credentials (encrypted — more on that later)
- Users and roles

SQLite runs in **WAL mode** so reads don't block writes while you're clicking around in settings and the agent is pushing telemetry.

## Backup is stupidly simple

```bash
cp /opt/amud/data/amud.db ~/backups/amud-$(date +%F).db
```

That's your entire dashboard. Restore = stop server, swap file, start server. I've migrated between LXCs this way. No merge conflicts. No "did I forget the secrets file."

## Honest caveat

If you're deep into GitOps — Ansible generating Homepage configs, PR reviews for every new service — AMUD isn't trying to replace that workflow. We're opposite philosophies.

AMUD is for people who want to tweak the dashboard from a browser at 11pm without opening VS Code.

From our roadmap, under "won't do":

> YAML as primary config. SQLite + UI is the whole point.

I meant it.

Try it: [/docs/installation/proxmox](/docs/installation/proxmox)
