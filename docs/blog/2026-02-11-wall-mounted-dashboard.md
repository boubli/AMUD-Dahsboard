---
slug: wall-mounted-dashboard
title: Turning an Old Tablet Into a Homelab Status Board
authors: [boubli]
tags: [homelab, themes]
description: Grid density, guest mode, theme gallery wallpapers, kiosk browser tips. AMUD was basically made for wall mounts.
---

I've got an old iPad on a desk mount running AMUD in guest mode. Shows CPU, which lights are on, if Plex is streaming, whether the *arr stack is actually running. Wife approves because it doesn't look like a terminal.

## Appearance settings that matter

**Settings → Appearance**

- **Grid columns:** 4–5 on a landscape tablet
- **Glass opacity:** lower = cleaner in bright rooms
- **Background:** grab a 2K wallpaper from [/themes](/themes)
- **Custom CSS:** Nord or Everforest for low glare

## What to put on the board

- Home Assistant card (lights + temp)
- Plex/Jellyfin with live stream badge
- Host CPU/RAM bars
- Key services with RUNNING status

Skip the admin controls on a shared display. Guest account.

## Kiosk browsers

- **Android:** Fully Kiosk Browser
- **iPad:** Guided Access
- **Linux SBC:** Chromium `--kiosk`

Point at your HTTPS URL (see the reverse proxy post). WebSockets need to work or the graphs lie.

That's it. Old hardware, new purpose, zero YAML.
