---
slug: admin-vs-guest
title: Guest Mode So Your Family Doesn't Stop the Plex Container
authors: [boubli]
tags: [homelab, security]
description: Admin sees power controls. Guest sees links and optional telemetry. Kitchen tablet stays safe.
image: themes/assets/AMUD-Theme-Neon.png
---

Put a dashboard on a kitchen wall tablet. Someone will eventually tap the wrong button.

<!-- truncate -->

AMUD Dashboard has **Admin** and **Guest** roles. Guests get links. Admins get the keys to the kingdom.

## What guests can do

- Open app URLs
- See the layout you configured
- Optionally see host CPU/RAM if you enable **Guest system telemetry visibility** under **Settings → Privacy & Access**

## What guests can't do

- Edit cards or settings
- Start/stop containers
- See container names/VMIDs on sensitive telemetry
- Mess with integrations

## Wall tablet setup

1. Create a Guest user
2. Enable public telemetry if you want the pretty graphs
3. Log the tablet into Guest
4. Stop worrying

Admins keep full power controls on linked LXCs. Guests just see whether stuff is running without the big red stop button.

[/docs/configuration](/docs/configuration)
