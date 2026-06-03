# AMUD Deployment & Maintenance Guide

This document outlines the verified production deployment methodologies for the AMUD Dashboard. AMUD is designed to run with a microscopic resource footprint (less than 10MB RAM combined) by avoiding heavy virtualization layers or Docker nesting within the LXC container.

---

## 1. Proxmox VE Autopilot Deployment (LXC & Host Agent)

The elite deployment method for Proxmox VE clusters uses our native autopilot shell script (`setup-amud.sh`) to automate container provisioning, directory bind-mounting, and host telemetry agent installation directly from GitHub Releases.

### How the Autopilot Script Works:
1. **Container Setup**: Provisions a minimal Debian 12 Linux Container (LXC) on Proxmox, allocating **256MB RAM** (with 256MB swap) for a native production runtime.
2. **Directory Bind-Mounting**: Configures a secure directory `/opt/amud/run` on the Proxmox host and bind-mounts it to the LXC container as `/opt/amud/run`.
3. **App Orchestration**: Retrieves the latest precompiled `amud-server` and `ui.tar.gz` assets directly from the official **GitHub Releases** page inside the LXC container, configuring them to run natively under `systemd`. No Docker, compiler, or Go/Rust toolchain is needed.
4. **Host Agent Installation**: Downloads the latest precompiled `amud-agent` binary directly from GitHub Releases, installs it at `/usr/local/bin/amud-agent` on the Proxmox host, and configures a lightweight `systemd` daemon (`amud-agent.service`). This agent streams real-time CPU, RAM, and disk metrics to the server via the UNIX Domain Socket.

### Execution:
SSH into your Proxmox VE host as `root` and run:
```bash
# Clone the repository and execute the installer
git clone https://github.com/boubli/AMUD-Dahsboard.git
cd AMUD-Dahsboard
chmod +x *.sh
./setup-amud.sh
```

Alternatively, you can run the installer directly via a single-line command:
```bash
curl -sSL https://github.com/boubli/AMUD-Dahsboard/releases/latest/download/setup-amud.sh | bash
```

---

## 2. How to Update AMUD to the Latest Release

When a new version of the AMUD Dashboard or Agent is released on GitHub, you can perform an in-place update of both the LXC server and the Proxmox host agent without destroying your configuration or database.

The update script `update-amud.sh` automatically queries the GitHub API for the latest release, stops the services, replaces the binaries and UI templates with the latest precompiled release assets, and restarts the services.

### Update execution:
SSH into your Proxmox VE host as `root` and run:
```bash
# Navigate to the cloned folder and run the updater
cd AMUD-Dahsboard
./update-amud.sh
```

Or run the updater directly via curl:
```bash
curl -sSL https://github.com/boubli/AMUD-Dahsboard/releases/latest/download/update-amud.sh | bash
```

---

## 3. Complete Uninstallation / Cleanup

If you need to completely remove the AMUD Dashboard LXC and the Proxmox host telemetry agent, we provide an uninstaller script that destroys the container, stops/deletes the host agent, and removes all socket directories.

SSH into your Proxmox VE host as `root` and run:
```bash
# Run the uninstaller script from the cloned repository
cd AMUD-Dahsboard
./uninstall-amud.sh
```

Or run the uninstaller directly via curl:
```bash
curl -sSL https://github.com/boubli/AMUD-Dahsboard/releases/latest/download/uninstall-amud.sh | bash
```

---

## 4. Portainer Stack Deployment (Web Editor)

For standard containerized host management panels (when running as a standalone container, not on Proxmox LXC):
1. Open your Portainer Web UI.
2. Select **Stacks** -> **Add Stack**.
3. Under the Web Editor panel, paste the following single-service definition:
```yaml
version: '3.8'

services:
  app:
    image: boubli/amud:latest
    container_name: amud_app
    restart: always
    ports:
      - "80:8000"
    environment:
      - DB_PATH=/app/data/amud.db
      - PORT=8000
      - AMUD_SOCKET_PATH=/opt/amud/run/amud.sock
    volumes:
      - ./data:/app/data
      - /opt/amud/run:/opt/amud/run
```
4. Click **Deploy the stack**.

---

## 5. Standalone Docker Compose (CLI Native)

To compile and launch the dashboard locally on any Linux server:
```bash
# Clone the repository
git clone https://github.com/boubli/AMUD-Dahsboard.git
cd AMUD-Dahsboard

# Run the compose stack in detached mode
docker compose up -d
```
The dashboard service is now serving at `http://localhost:8000`.
