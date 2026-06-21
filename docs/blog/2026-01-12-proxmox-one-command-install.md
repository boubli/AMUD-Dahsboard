---
slug: proxmox-one-command-install
title: Install AMUD Dashboard on Proxmox in One Command (No Docker)
authors: [boubli]
tags: [proxmox, homelab, selfhosted]
description: One curl on your Proxmox host provisions an LXC, installs the agent on the hypervisor, and gives you a working dashboard on port 8000.
image: img/amud-architecture.svg
---

Docker is fine. I use it. But wrapping a *dashboard* — a thing that mostly displays links and polls metrics — inside another network stack on a Proxmox box always felt like wearing two coats indoors.

<!-- truncate -->

AMUD's happy path on Proxmox is **native**: agent on the host, server in a tiny Debian 12 LXC, Unix socket between them.

## The install

SSH to your Proxmox host as root:

```bash
curl -sSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/setup-amud.sh | bash
```

The script:

1. Drops `amud-agent` on the host as a systemd unit
2. Spins up an LXC called `amud-dashboard` (1 vCPU, 256MB RAM, 4GB disk)
3. Bind-mounts `/opt/amud/run` so both sides share the IPC socket
4. Installs `amud-server` inside the LXC and prints the IP

No registry pull. No compose file. Just binaries.

## First login

```
http://<lxc-ip>:8000/
```

Default creds are `admin` / `password`. Change that immediately under **Settings → Admin Profile**. Please. I'm begging you.

## The token thing (don't skip this)

Host CPU/RAM bars will work out of the box. App card badges won't — they'll sit on **CHECKING...** until you give the agent a Proxmox API token.

Quick restricted user setup:

```bash
pveum role add AMUDAgentRole -privs "VM.Audit Sys.Audit VM.PowerMgmt"
pveum group add amud-group
pveum user add amud@pve -group amud-group
pveum aclmod / -group amud-group -role AMUDAgentRole
pveum user token add amud@pve amud-token --privsep 0
```

**Uncheck Privilege Separation** when creating the token. I learned that one the hard way.

Paste the full `PVEAPIToken=...` string into **Settings → Proxmox VE**, link app cards to CTIDs, watch badges flip to RUNNING/STOPPED.

Full walkthrough: [/docs/installation/proxmox](/docs/installation/proxmox)

Prefer Docker anyway? [/docs/installation/docker](/docs/installation/docker) — works, just a bit heavier on RAM.
