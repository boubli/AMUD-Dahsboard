# AMUD Deployment Guide

Production deployment instructions for AMUD Server and Telemetry Agent.

---

## 1. Native Proxmox VE Installation (LXC & Host Agent)

This method deploys the server natively inside a minimal Linux Container (LXC) and installs the agent on the Proxmox host. This avoids virtualization overhead and Docker nesting.

### How the Installer Works:
1. **LXC Provisioning**: Creates a minimal Debian 12 container on Proxmox allocating 256MB RAM (256MB swap).
2. **Directory Bind-Mounting**: Configures a directory (`/opt/amud/run`) on the host and bind-mounts it to the LXC container at `/opt/amud/run` for UDS communication.
3. **Server Deployment**: Downloads precompiled `amud-server` and assets, setting up a systemd unit inside the container.
4. **Agent Installation**: Downloads and registers `amud-agent` as a systemd daemon on the Proxmox host to stream CPU, RAM, and disk metrics via the UNIX domain socket.

### Execution
SSH into the Proxmox VE host as `root` and run:
```bash
curl -sSL https://github.com/boubli/AMUD-Dashboard/releases/latest/download/setup-amud.sh | bash
```

---

## 1.1 Proxmox API Token Setup (For LXC Telemetry)

To enable live LXC status polling in the dashboard, the agent must be authenticated to the Proxmox VE REST API.

### 1. Generate API Token
1. In the Proxmox Web UI, navigate to **Datacenter → Permissions → API Tokens**.
2. Click **Add**. Select user (e.g. `root@pam`) and Token ID (e.g. `amud`).
3. **Uncheck** *Privilege Separation* (required for the token to inherit the user's VM/System audit permissions).
4. Copy the returned Secret key.

### 2. Configure Agent Environment
Edit the agent systemd unit file on the **Proxmox host**:
```bash
nano /etc/systemd/system/amud-agent.service
```

Add your token under the `[Service]` section:
```ini
Environment="PVE_API_TOKEN=PVEAPIToken=root@pam!amud=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
```

Reload and restart the daemon:
```bash
systemctl daemon-reload
systemctl restart amud-agent
```

Verify via logs:
```bash
journalctl -u amud-agent -n 10
```
It should report successful container extraction.

---

## 2. In-Place Updates

Update the LXC server and host agent binaries without modifying existing databases or configurations.

SSH into the Proxmox VE host as `root` and execute:
```bash
curl -sSL https://github.com/boubli/AMUD-Dashboard/releases/latest/download/update-amud.sh | bash
```

---

## 3. Uninstallation

To destroy the LXC container, stop host-side telemetry agents, and clean up directories:

SSH into the Proxmox VE host as `root` and execute:
```bash
curl -sSL https://github.com/boubli/AMUD-Dashboard/releases/latest/download/uninstall-amud.sh | bash
```

---

## 4. Docker Compose Deployment

For containerized hosts (running server and agent on standard Linux distros):

### 1. Create Environment Config (`.env`)
```bash
AMUD_AGENT_SECRET=change-me-to-a-long-random-string
```

### 2. Compose Definition (`docker-compose.yml`)
```yaml
version: '3.8'

services:
  amud-server:
    image: tradmss/amud-dashboard:latest
    container_name: amud_server
    restart: unless-stopped
    ports:
      - "8000:8000"
    environment:
      - DB_PATH=/app/data/amud.db
      - PORT=8000
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
      - AMUD_AGENT_SECRET=${AMUD_AGENT_SECRET}
    volumes:
      - ./data:/app/data
      - amud_run:/var/run/amud

  amud-agent:
    image: tradmss/amud-dashboard:latest
    container_name: amud_agent
    entrypoint: ["/app/amud-agent"]
    environment:
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
      - AMUD_AGENT_SECRET=${AMUD_AGENT_SECRET}
    volumes:
      - amud_run:/var/run/amud
      - /var/run/docker.sock:/var/run/docker.sock:ro
    restart: unless-stopped

volumes:
  amud_run:
    name: amud_run
```

Deploy the stack:
```bash
docker compose up -d
```
