<div align="center">
  <img src="https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/assist/amud-logo-github.png" alt="AMUD Logo" width="300" />
</div>

# AMUD Dashboard

[![GitHub Release](https://img.shields.io/github/v/release/boubli/AMUD-Dashboard?style=flat-square)](https://github.com/boubli/AMUD-Dashboard/releases/latest)

**[Changelog](https://boubli.github.io/AMUD-Dashboard/docs/changelog)** · **[Blog](https://boubli.github.io/AMUD-Dashboard/blog)** · **[Theme Gallery](https://boubli.github.io/AMUD-Dashboard/themes)** · **[Roadmap](https://boubli.github.io/AMUD-Dashboard/docs/roadmap)** · **[Docs](https://boubli.github.io/AMUD-Dashboard/)** · **[FAQ](https://boubli.github.io/AMUD-Dashboard/docs/faq)**

### What's new in v1.5.1.0

- **Bundled themes** — apply 18 looks from Settings without copying CSS from the docs
- **Appearance cleanup** — simpler quick colors, live Custom CSS preview, overlay presets removed
- **Audit log** — reliable schema, clear errors, settings changes logged
- **In-app updater** — SHA256 checksum lookup fixed for Proxmox installs
- [Theme Gallery](https://boubli.github.io/AMUD-Dashboard/themes) — six new advanced themes + Download CSS

![AMUD Dashboard UI](https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/assist/AMUD-Dashboard.png)

A compiled, zero-dependency homelab control center and telemetry dashboard.

Unlike legacy dashboards (Heimdall, Homepage, Homarr) that run on heavy runtimes (PHP-FPM, Node.js) and rely on complex nested YAML configuration files, AMUD is written in compiled Rust and persisted entirely in SQLite. Combined, the server and telemetry agent idle at **35MB to 100MB of RAM** with sub-millisecond route execution.

## Architecture & Design Decisions

AMUD Dashboard is split into two native binaries:
1. **`amud-server`**: Axum-based web server serving server-rendered HTML (templated via Alpine.js) and managing state via SQLite.
2. **`amud-agent`**: Standalone daemon installed on the homelab host. It queries host metrics, Proxmox VE containers, and Docker runtimes, streaming raw JSON payloads back to the server via Unix Domain Sockets (UDS) or TCP.

```mermaid
graph TD
    User[Web Browser] -->|HTML / WebSockets| Server[amud-server]
    Server -->|SQLite WAL| DB[(amud.db)]
    Agent[amud-agent] -->|JSON over UNIX Socket| Server
    Agent -->|Direct HTTPS REST API| PVE[Proxmox VE API]
    Agent -->|Unix Domain Socket| Docker[Docker Daemon]
```

### Technical Stack Justifications

#### Rust & Axum
* **No Runtime Overhead**: Compiles directly to native machine code. Eliminates the JVM/V8 startup and heap overhead.
* **Concurrent Event Loop (Tokio)**: Telemetry streams and third-party integrations (AdGuard, Pi-hole, Plex, Home Assistant) poll concurrently on Tokio green threads. Telemetry is serialized once per poll tick and broadcasted to WebSockets using a `tokio::sync::watch` channel.

#### SQLite Persistence (`rusqlite`)
* **Zero YAML**: Configuration is stored in an embedded SQLite database. Layouts, category tabs, and settings are configured directly via the UI, bypassing YAML syntax headaches.
* **Performance**: Configured in WAL (Write-Ahead Logging) mode, enabling concurrent reads and low-latency writes without external network overhead.

#### Direct Telemetry Collection
* **Zero Shell Subprocesses**: Legacy solutions fork system calls like `pvesh` or `curl` every few seconds to grab container stats, resulting in high CPU overhead.
* **Natively Networked**: `amud-agent` utilizes `hyper` and `rustls` to send native HTTPS REST API calls to Proxmox VE and reads the Docker daemon directly over the UNIX socket via `hyperlocal`.

---

## Telemetry Configuration

### Proxmox VE Integration
Host metrics function automatically. For LXC container monitoring, the agent must be authenticated to the Proxmox VE REST API.

#### 1. Generate API Token
In the Proxmox VE Web UI:
1. Navigate to **Datacenter → Permissions → API Tokens**.
2. Click **Add**. Select User (e.g., `root@pam`) and Token ID (e.g., `amud`).
3. **Uncheck** *Privilege Separation* so the token inherits the user's VM/System audit permissions.
4. Copy the returned Secret key.

#### 2. Pass Token to Agent
Set the environment variable on the host running the agent:
```bash
PVE_API_TOKEN=PVEAPIToken=root@pam!amud=XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX
```

---

## Deployment

### Docker Compose

For containerized hosts (combines server and agent communicating over a shared volume for the Unix socket):

```yaml
version: '3.8'

services:
  app:
    image: tradmss/amud-dashboard:latest
    container_name: amud_app
    restart: always
    ports:
      - "8000:8000"
    environment:
      - PORT=8000
      - BIND_ADDR=0.0.0.0
      - DB_PATH=/app/data/amud.db
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
      - AMUD_AGENT_SECRET=change-me-to-a-long-random-string # MUST match the agent secret below
    cap_drop:
      - ALL
    security_opt:
      - no-new-privileges:true
    volumes:
      - ./data:/app/data
      - amud_run:/var/run/amud

  agent:
    image: tradmss/amud-dashboard:latest
    container_name: amud_agent
    entrypoint: ["/app/amud-agent"]
    restart: always
    environment:
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
      - AMUD_AGENT_SECRET=change-me-to-a-long-random-string # MUST match the app secret above
      - AMUD_DOCKER=0 # Set to 1 to enable Docker monitoring
    cap_drop:
      - ALL
    security_opt:
      - no-new-privileges:true
    volumes:
      - amud_run:/var/run/amud
      - /var/run/docker.sock:/var/run/docker.sock:ro

volumes:
  amud_run:
    name: amud_run
```

### Proxmox LXC Autopilot Script
For native installation within a Proxmox VE LXC container (running outside Docker), execute this on your Proxmox VE host:
```bash
curl -sSL https://github.com/boubli/AMUD-Dashboard/releases/latest/download/setup-amud.sh | bash
```

---

## Production Resource Footprint

| Dimension | Heimdall (Legacy PHP) | AMUD Dashboard (Rust) |
| :--- | :--- | :--- |
| **Engine** | PHP 8+ / Laravel | Rust / Axum / Tokio |
| **Execution Overhead** | High (Interpreted PHP-FPM) | Zero (Native Machine Code) |
| **Asset Delivery** | Disk reads per request | Embedded in binary via `include_str!` |
| **Idle RAM Footprint** | ~150MB | **35MB - 100MB** (Combined) |
| **Startup / Boot Time**| ~2 - 5 seconds | **Sub-millisecond** |

---

## Support & Donation

* [GitHub Sponsors](https://github.com/sponsors/boubli)
* [Donate via Stripe](https://buy.stripe.com/cNi14n6b9a7v5Jg4Rq4ko00)
* [Ko-fi](https://ko-fi.com/Youssefboubli)
