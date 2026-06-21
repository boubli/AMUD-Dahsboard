---
slug: backup-amud-db
title: Back Up Your Entire Dashboard in One File
authors: [boubli]
tags: [homelab, selfhosted]
description: amud.db is your whole config. Copy it. Don't forget .amud-secrets-key if you use encrypted integrations.
image: img/AMUD-Dashboard.png
---

I've seen people maintain elaborate backup scripts for YAML dashboards — git repos, env files, secret sidecars, three different paths.

<!-- truncate -->

AMUD Dashboard backup:

```bash
cp /opt/amud/data/amud.db ~/backups/amud-$(date +%F).db
```

Done. Apps, tabs, settings, users, encrypted creds (encrypted blob in DB).

## Restore

1. Stop `amud-server`
2. Replace `amud.db`
3. Start `amud-server`

Your dashboard is exactly as it was. I've used this to clone a test setup to prod. Worked first try.

## Don't forget the keyfile

Integrations use `.amud-secrets-key` for AES-GCM. Backup it **with** the database. Restore DB without key = encrypted tokens become useless noise.

## Cron it and forget it

```bash
0 3 * * * cp /opt/amud/data/amud.db /mnt/nas/backups/amud-$(date +\%F).db
```

Keep a week of dailies. Homelab disasters always happen on the day you skipped backups.

More: [/docs/troubleshooting](/docs/troubleshooting)
