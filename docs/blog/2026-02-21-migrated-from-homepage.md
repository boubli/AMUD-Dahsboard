---
slug: migrated-from-homepage
title: I Migrated From Homepage to AMUD Dashboard — What I Lost and Gained
authors: [boubli]
tags: [homelab, selfhosted]
description: Honest post-Homepage review. GitOps YAML vs browser config. No sugarcoating.
image: img/AMUD-Dashboard.png
---

I ran Homepage for a year. Liked it. Still recommend it for the right person.

<!-- truncate -->

Switching to AMUD Dashboard wasn't "Homepage bad, AMUD Dashboard good." It was "I'm tired of this specific workflow."

## What I lost

- **Git-tracked config** — every service change was a commit. That's gone. AMUD Dashboard lives in SQLite, not `services.yaml` in a repo.
- **Community widget YAML** — Homepage's widget library is huge. AMUD Dashboard has built-in Plex/Jellyfin/HA/Proxmox stuff but not the long tail of community widgets.
- **Docker-only mental model** — Homepage fits the "everything in compose" crowd perfectly.

## What I gained

- **UI edits at midnight** — add a card from my phone browser. No SSH, no git pull on the host.
- **Native Proxmox LXC control** — not a widget bolted on. First-class CTID binding and power actions.
- **RAM** — meaningful on a crowded Proxmox node.
- **One-file backup** — `amud.db` instead of yaml + env + secrets scattered around.

## Who should stay on Homepage

You like GitOps. You generate configs from Ansible. You want fifty community widgets. YAML doesn't make you twitch.

## Who should try AMUD Dashboard

Proxmox primary. YAML fatigue. Want live LXC status without shell scripts. Okay with "config in a database" as the trade.

Migration tip: screenshot your Homepage layout, recreate cards in AMUD Dashboard while watching TV. Took me about forty minutes for ~25 services. Your mileage varies.

```bash
curl -sSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/setup-amud.sh | bash
```
