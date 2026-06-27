<p align="center">
  <img src="https://boubli.github.io/AMUD-Dashboard/img/amud-logo-github.png" alt="AMUD Logo" width="280" />
</p>

# AMUD Dashboard

<p align="center">
  <img src="https://boubli.github.io/AMUD-Dashboard/img/AMUD-Dashboard.png" alt="AMUD Dashboard UI" width="720" />
</p>

AMUD (Advanced Modern Unified Dashboard) is a high-performance, intelligent homelab cockpit built for resource-constrained environments. While legacy dashboards demand heavy runtimes and YAML files, AMUD is a compiled Rust stack that idles around **30–50 MB RAM** (server + agent) with sub-millisecond routing.

**📚 Documentation:** [https://boubli.github.io/AMUD-Dashboard/](https://boubli.github.io/AMUD-Dashboard/)

**Deploy:** [Docker guide](https://boubli.github.io/AMUD-Dashboard/docs/installation/docker) · [Deployment scripts](https://github.com/boubli/AMUD-Dashboard/blob/main/DEPLOY.md)

```bash
docker pull tradmss/amud-dashboard:latest
```

---

## Quick start (Docker Compose)

```yaml
services:
  amud_app:
    image: tradmss/amud-dashboard:latest
    ports:
      - "8000:8000"
    volumes:
      - ./data:/app/data
    restart: unless-stopped

  amud_agent:
    image: tradmss/amud-dashboard:latest
    command: ["/app/amud-agent"]
    network_mode: host
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./run:/opt/amud/run
    environment:
      - AMUD_DOCKER=1
    restart: unless-stopped
```

Open `http://<host>:8000` — default login is in the [docs](https://boubli.github.io/AMUD-Dashboard/docs/installation/docker).

---

## Proxmox telemetry (agent)

The agent talks to the Proxmox VE REST API directly. Set:

```bash
PVE_API_TOKEN=PVEAPIToken=root@pam!amud=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

Create the token under **Datacenter → Permissions → API Tokens**. If unset, host CPU/RAM/disk metrics still work; LXC polling is skipped.

---

## Why AMUD vs Heimdall / Homepage / Homarr

| | Legacy dashboards | AMUD |
| :--- | :--- | :--- |
| **Runtime** | PHP / Node / YAML | Compiled Rust + SQLite |
| **Config** | YAML files | Web UI |
| **Telemetry** | Often static links | Live agent + WebSockets |
| **Idle RAM** | 80–150 MB+ | ~30–50 MB combined |

---

## Support

- **GitHub:** [boubli/AMUD-Dashboard](https://github.com/boubli/AMUD-Dashboard)
- **Sponsor:** [GitHub Sponsors](https://github.com/sponsors/boubli)
