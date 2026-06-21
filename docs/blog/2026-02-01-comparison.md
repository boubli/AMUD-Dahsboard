---
slug: amud-vs-heimdall-homepage-homarr
title: AMUD Dashboard vs Heimdall vs Homepage vs Homarr (I'll Be Honest)
authors: [boubli]
tags: [homelab, selfhosted]
description: I built AMUD Dashboard so I'm biased. Here's when you'd still pick something else.
image: img/AMUD-Dashboard.png
---

I built AMUD. I'm biased. Here's the honest comparison anyway.

<!-- truncate -->

| | AMUD | Heimdall | Homepage | Homarr |
|---|:---:|:---:|:---:|:---:|
| Runtime | Rust binary | PHP/Laravel | Node/React | Next.js |
| Config | SQLite + UI | DB + UI | YAML | UI + files |
| Idle RAM | ~26–35MB (PVE native) | ~150MB | varies | 200MB+ |
| Proxmox LXC control | native | no | widgets | limited |
| YAML required | never | no | yes | partial |

## Pick Heimdall if

You want simple bookmarks and PHP on your server doesn't bother you. No live telemetry needed.

## Pick Homepage if

You live in Git, love YAML, want the widget ecosystem. You're good at debugging indent errors. Respect.

## Pick Homarr if

You want the polished drag-drop UI and don't care about RAM. *arr integration out of the box matters to you.

## Pick AMUD Dashboard if

- Proxmox is your hypervisor and you want native LXC status + power controls
- You refuse to maintain YAML for a link portal
- You want Plex/Jellyfin/HA/host metrics without gluing shell scripts
- ~26MB idle on Proxmox sounds nice

## What AMUD Dashboard won't become

SaaS. YAML-first config. Node/PHP dependency. It's a homelab tool. Stays that way.

```bash
curl -sSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/setup-amud.sh | bash
```

[GitHub](https://github.com/boubli/AMUD-Dashboard) · [Themes](/themes)
