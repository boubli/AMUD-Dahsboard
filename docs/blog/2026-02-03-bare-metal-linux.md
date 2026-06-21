---
slug: bare-metal-linux-install
title: Running AMUD Dashboard on Bare Metal (No Docker, No LXC)
authors: [boubli]
tags: [homelab, selfhosted]
description: Dedicated amud system user, /opt/amud layout, systemd units. Lowest overhead if you have a spare NUC.
image: img/AMUD-Dashboard.png
---

Sometimes you have a spare NUC or mini PC that does one job: show the dashboard on a wall and nothing else.

<!-- truncate -->

Docker on that box is overhead you don't need.

## Directory layout

```
/opt/amud/run   → Unix socket
/opt/amud/data  → amud.db
/opt/amud/ui    → templates
```

## Don't run as root

```bash
sudo groupadd --system amud
sudo useradd --system -g amud -s /sbin/nologin amud
sudo mkdir -p /opt/amud/run /opt/amud/data /opt/amud/ui
sudo chown -R amud:amud /opt/amud
```

Server runs as `amud`. Agent typically needs more privileges for host metrics — see the full guide for the permission model.

## Binaries

Grab latest from [GitHub Releases](https://github.com/boubli/AMUD-Dashboard/releases). Drop `amud-server` and `amud-agent` in `/usr/local/bin/`, set up systemd units, go.

Works on Debian, Ubuntu, Fedora, Arch — anything with systemd.

Full walkthrough: [/docs/installation/linux](/docs/installation/linux)

When I'd pick this over Proxmox LXC: dedicated dashboard hardware, no hypervisor, absolute minimum RAM.
