---
slug: home-assistant-dashboard-card
title: Home Assistant Stats on Your Dashboard (Not Just a Link to :8123)
authors: [boubli]
tags: [integrations, homelab]
description: Name your app card Home Assistant, drop in a long-lived token, get lights/switches/temp on the card.
image: img/AMUD-Dashboard.png
---

I had a Home Assistant card that was literally just a URL. Useful, but my wall tablet deserved better.

<!-- truncate -->

AMUD Dashboard can show on that card:

- How many **lights** are on
- How many **switches** are on
- Average home **temperature**

## The one naming rule that trips people up

Your app card has to be named exactly **`Home Assistant`**. Not "HA" or "homeassistant" or "the smart house thing." Exactly that string. Yes, I should make that more flexible someday. For now, that's the deal.

## Setup

1. HA → Profile → Long-Lived Access Tokens → create one
2. AMUD → **Settings → Smart Home** → URL + token → save

## How it polls without being rude to HA

Prefers the **Template API** (`POST /api/template`) so HA computes counts server-side. Falls back to `/api/states` if templates aren't available. Heavier, but works.

Runs on the same Tokio runtime as everything else — won't block your CPU graphs.

Pair with LXC status: see the HA container is up *and* you've got four lights on at 2am. Investigate that later.

[/docs/configuration](/docs/configuration)
