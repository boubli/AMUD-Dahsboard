---
slug: plex-jellyfin-live-badges
title: See What's Playing on Plex and Jellyfin Without Opening the App
authors: [boubli]
tags: [integrations, homelab]
description: AMUD Dashboard polls /Sessions and /status/sessions in the background. Your media cards show live stream titles.
image: img/blog/plex.svg
---

"Is anyone watching something right now or can I restart the container?"

<!-- truncate -->

That's the question I wanted answered from the dashboard, not from opening Plex on my phone.

## Setup (boring but fast)

**Settings → Integrations**

**Jellyfin:** base URL + API key from Dashboard → Advanced → API Keys.

**Plex:** base URL + `X-Plex-Token`. If you've never extracted a Plex token before, welcome to the club — there's a few documented ways and they're all mildly annoying.

## What the badge does

Starts as **CHECKING...**, then shows the active stream title. Multiple clients? You'll see something like `Dune (+2 more)`.

Missing creds → **NOT CONFIGURED**. Fixable in thirty seconds once you stop procrastinating.

## Why this pairs well with LXC status

Card shows Jellyfin is **RUNNING** *and* someone's watching *Blade Runner*. Now you know restarting would make someone mad. That's operational awareness, not just bookmarks.

API keys get **AES-GCM encrypted** in SQLite at rest. Not sitting in plain text in a yaml file on disk.

Details: [/docs/configuration](/docs/configuration)
