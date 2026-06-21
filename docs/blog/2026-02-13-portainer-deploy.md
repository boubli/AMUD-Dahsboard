---
slug: portainer-deploy
title: Deploying AMUD Dashboard Through Portainer (For Clickers Not SSHers)
authors: [boubli]
tags: [docker, homelab]
description: Stack template, matching AMUD_AGENT_SECRET, Docker socket mount. GUI homelab crowd deserves dashboards too.
image: img/AMUD-Dashboard.png
---

I live in SSH. Some of you live in Portainer. Both are valid.

<!-- truncate -->

AMUD Dashboard deploys as a stack: `amud_app` + `amud_agent` + `amud_run` volume.

## Portainer steps

1. **Stacks → Add stack** → name it `amud`
2. Paste the compose from [/docs/installation/docker](/docs/installation/docker)
3. Set `AMUD_AGENT_SECRET` to something long and random
4. **Same secret in both services** — write it down
5. Deploy

Agent needs `/var/run/docker.sock:ro` and `AMUD_DOCKER=1`.

## After deploy

`http://your-host:8000` → login → change password → add cards in UI.

Bind `./data` to a host path you snapshot. That's your backup.

Portainer-specific notes: [/docs/installation/portainer](/docs/installation/portainer)

Docker path uses a bit more RAM than native Proxmox LXC. Trade-off for convenience.
