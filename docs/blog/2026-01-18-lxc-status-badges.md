---
slug: lxc-status-badges
title: Green and Red LXC Badges (Plus Start/Stop From the Dashboard)
authors: [boubli]
tags: [proxmox, homelab]
description: Link an app card to a CTID, get live RUNNING/STOPPED status, and optionally power-cycle containers without opening Proxmox.
image: img/AMUD-Dashboard.png
---

A link list tells you where Jellyfin *should* be. It doesn't tell you Jellyfin is stopped because you updated the LXC last night and forgot to bring it back up.

<!-- truncate -->

AMUD Dashboard app cards can bind to a Proxmox **CTID**. Badge flips between **RUNNING**, **STOPPED**, and the eternal **CHECKING...** when something's misconfigured.

## Setup checklist

1. `amud-agent` running on the Proxmox host
2. API token in **Settings → Proxmox VE**
3. App card has the correct LXC/VM ID field filled in

That's it. If you're stuck on CHECKING..., 90% of the time it's the token. I wrote a whole troubleshooting post for that misery.

## Power controls

If your API role includes `VM.PowerMgmt`, admins get start/stop/restart on linked containers. Handy when you're on your phone and don't want the Proxmox mobile experience.

Guest users see status only. No accidental `pct stop` from the kitchen tablet.

## Don't use root@pam

Create a restricted `amud@pve` user with `VM.Audit`, `Sys.Audit`, and optionally `VM.PowerMgmt`. Paste that token into AMUD. Your dashboard doesn't need god-mode PVE access.

Config details: [/docs/configuration](/docs/configuration)
