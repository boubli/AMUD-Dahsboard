---
sidebar_position: 2
---

# Docker Deployment

Deploying AMUD in a Docker environment containerizes the entire dashboard and telemetry ecosystem. We provide pre-built, multi-architecture Docker images (`x86_64` and `arm64`) that enable instant setup.

---

## 1. How the Docker Ecosystem Communicates

AMUD uses a multi-container architecture in Docker to enforce isolation and minimize host resource usage:

```
                  +----------------------------------------------+
                  |                  DOCKER HOST                 |
                  |                                              |
                  |     +----------------------------------+     |
                  |     |       /var/run/docker.sock       |     |
                  |     +-----------------+----------------+     |
                  |                       ^                      |
                  |                       | (Read-Only Mount)    |
                  |                       v                      |
                  |     +-----------------+----------------+     |
                  |     |        amud-agent container      |     |
                  |     | - Telemetry agent daemon         |     |
                  |     +-----------------+----------------+     |
                  |                       |                      |
                  |                       | Writes IPC Telemetry |
                  |                       v                      |
                  |     +-----------------+----------------+     |
                  |     |    Shared Volume: amud_run       |     |
                  |     |    (Socket: /var/run/amud/...)   |     |
                  |     +-----------------+----------------+     |
                  |                       ^                      |
                  |                       | Reads IPC Telemetry  |
                  |                       v                      |
                  |     +-----------------+----------------+     |
                  |     |      amud-dashboard container    |     |
                  |     | - Serving Web UI (Port 8000)     |     |
                  |     +----------------------------------+     |
                  +----------------------------------------------+
```

* **`amud-dashboard`**: The core application server. It runs the Web UI, stores state inside SQLite, and serves dashboards to users over HTTP.
* **`amud-agent`**: The telemetry helper. It mounts the host's Docker socket to discover containers and monitor their active state.
* **`amud_run` Volume**: A high-speed, in-memory or directory volume sharing a Unix Domain Socket between the two containers. This allows them to transfer rich metric payloads locally with sub-millisecond latency.

---

## 2. Recommended: Docker Compose Setup

Using Docker Compose is the standard method for running AMUD. Create a `docker-compose.yml` file in your preferred configuration directory (e.g. `/opt/amud/`):

```yaml title="docker-compose.yml"
version: '3.8'

services:
  amud-dashboard:
    image: tradmss/amud-dashboard:latest
    container_name: amud-dashboard
    restart: unless-stopped
    ports:
      - "8000:8000"
    environment:
      - PORT=8000
      - DB_PATH=/app/data/amud.db
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
      - AMUD_ENABLE_PROXMOX=false # Set to true if running on Proxmox
    volumes:
      - amud_data:/app/data
      - amud_run:/var/run/amud
    deploy:
      resources:
        limits:
          cpus: '0.25'
          memory: 128M
        reservations:
          memory: 32M

  amud-agent:
    image: tradmss/amud-dashboard:latest
    container_name: amud-agent
    entrypoint: ["/app/amud-agent"]
    restart: unless-stopped
    environment:
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
    volumes:
      - amud_run:/var/run/amud
      - /var/run/docker.sock:/var/run/docker.sock:ro
    deploy:
      resources:
        limits:
          cpus: '0.10'
          memory: 64M
        reservations:
          memory: 16M

volumes:
  amud_data:
    name: amud_data
  amud_run:
    name: amud_run
```

### Deploying the Stack
To start the services in detached background mode:
```bash
docker compose up -d
```

To view real-time log aggregates for diagnostic purposes:
```bash
docker compose logs -f
```

---

## 3. Alternative: Docker CLI Run

If you prefer using pure CLI commands without creating files, establish the shared volume and start both containers sequentially:

```bash
# 1. Create a shared volume for the Unix IPC socket
docker volume create amud_run

# 2. Create a persistent volume for the SQLite database
docker volume create amud_data

# 3. Start the dashboard web server
docker run -d \
  --name amud-dashboard \
  -p 8000:8000 \
  -v amud_data:/app/data \
  -v amud_run:/var/run/amud \
  -e PORT=8000 \
  -e DB_PATH=/app/data/amud.db \
  -e AMUD_SOCKET_PATH=/var/run/amud/amud.sock \
  -e AMUD_ENABLE_PROXMOX=false \
  --restart unless-stopped \
  tradmss/amud-dashboard:latest

# 4. Start the telemetry agent
docker run -d \
  --name amud-agent \
  -v amud_run:/var/run/amud \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  -e AMUD_SOCKET_PATH=/var/run/amud/amud.sock \
  --entrypoint "/app/amud-agent" \
  --restart unless-stopped \
  tradmss/amud-dashboard:latest
```

---

## 4. Configuration Reference

You can pass these environment variables to adjust container configurations:

| Variable | Target Container | Default | Description |
|---|---|---|---|
| `PORT` | `amud-dashboard` | `8000` | Port on which the Axum web server listens. |
| `DB_PATH` | `amud-dashboard` | `/app/data/amud.db` | Directory path pointing to the SQLite database file. |
| `AMUD_SOCKET_PATH` | Both | `/var/run/amud/amud.sock` | File path pointing to the Unix socket for agent-server IPC. |
| `PVE_API_TOKEN` | `amud-agent` | *(None)* | Proxmox API token (if using agent on a PVE host; not needed for Docker monitoring). |
| `AMUD_ENABLE_PROXMOX` | `amud-dashboard` | `false` | Set to `true` if installing on a Proxmox LXC to show the Proxmox settings tab in the UI. |

:::info For Non-Proxmox Users (Other OS)
If you are deploying AMUD via Docker on a standard Linux distribution (like Ubuntu, Fedora, Arch Linux, or Alpine) instead of Proxmox VE, the Proxmox Integration tab is hidden by default. If you do want to enable it, you can pass `-e AMUD_ENABLE_PROXMOX=true` to your dashboard container. Your host's CPU and Memory will still display perfectly fine on the dashboard under the "System" widget!
:::

---

## 5. Security Hardening Recommendations

When running AMUD in production environments, implement these security practices:

### A. Docker Socket Trust Boundary
The agent volume mount is defined as `/var/run/docker.sock:/var/run/docker.sock:ro`. The `:ro` modifier protects the socket file from being modified through the bind mount, but Docker's HTTP API can still process lifecycle requests over that socket. Treat the agent as trusted, and use a Docker socket proxy if you need method-level filtering.

### B. User Permissions (Non-Root Running)
If your host enforces strict daemon security, configure the agent to run under the host's `docker` group ID so it does not require root privileges. 

1. Identify the GID of the `docker` group on your host system:
   ```bash
   getent group docker | cut -d: -f3
   ```
2. In the `amud-agent` service definition inside `docker-compose.yml`, specify the appropriate user grouping:
   ```yaml
   user: "1000:998" # Replace 998 with your host's docker GID
   ```

### C. Restricting Port Exposure
If you use a reverse proxy (e.g. Nginx, Caddy, Traefik), prevent exposing port `8000` to the public internet. Bind the port strictly to the host loopback interface by modifying the dashboard ports entry:
```yaml
ports:
  - "127.0.0.1:8000:8000"
```

---

## 6. Accessing the Dashboard

Navigate to your server's IP address on port 8000:
```
http://<YOUR_SERVER_IP>:8000/
```

:::tip Default Credentials
- **Username**: `admin`
- **Password**: `admin` (or `password` depending on version setup)
:::

---

## 7. Upgrading

Pull the latest image and recreate the stack:

```bash
docker compose pull
docker compose up -d
```

After upgrading:

1. **Hard-refresh the browser** (`Ctrl+Shift+R`) or [clear the PWA cache](../troubleshooting#pwa--browser-cache-issues).
2. If you use HTTPS via a reverse proxy, set `AMUD_SECURE_COOKIES=1` on the dashboard container — see [Security](../security.md).
3. Confirm both `amud-dashboard` and `amud-agent` containers are healthy: `docker compose ps`.
