---
slug: docker-homelab-35mb
title: Docker Homelab Monitoring in ~35MB RAM
authors: [boubli]
tags: [docker, homelab, selfhosted]
description: Two containers, one shared socket volume, AMUD_DOCKER=1 on the agent. Not Proxmox-native but still lean.
image: img/AMUD-Dashboard.png
---

Not everyone runs Proxmox. Some of you are Docker-all-the-way-down and that's fine.

<!-- truncate -->

AMUD Dashboard in Docker is two containers:

- **`amud_app`** — web server + SQLite
- **`amud_agent`** — telemetry, mounts Docker socket read-only

They talk over a shared volume at `/var/run/amud/amud.sock`. Same IPC model as bare metal, just containerized.

## Minimal compose

```yaml
services:
  app:
    image: tradmss/amud-dashboard:latest
    ports:
      - "8000:8000"
    environment:
      - AMUD_AGENT_SECRET=use-a-long-random-string-here
      - DB_PATH=/app/data/amud.db
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
    volumes:
      - ./data:/app/data
      - amud_run:/var/run/amud

  agent:
    image: tradmss/amud-dashboard:latest
    entrypoint: ["/app/amud-agent"]
    environment:
      - AMUD_AGENT_SECRET=use-a-long-random-string-here
      - AMUD_DOCKER=1
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
    volumes:
      - amud_run:/var/run/amud
      - /var/run/docker.sock:/var/run/docker.sock:ro

volumes:
  amud_run:
```

**The secret must match in both containers.** I cannot count how many issue reports were just mismatched `AMUD_AGENT_SECRET`.

Link app cards to **container names** for running/stopped badges.

Full guide: [/docs/installation/docker](/docs/installation/docker)

RAM lands around 35–100MB depending on host. Still lighter than most PHP/Node dashboards I've run.
