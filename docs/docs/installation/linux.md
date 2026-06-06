---
sidebar_position: 4
---

# Bare-Metal Linux Installation

For users who want to run AMUD directly on a Linux server (Debian, Ubuntu, Fedora, Arch Linux, Rocky Linux, etc.) without Proxmox VE or Docker, you can install the pre-compiled binaries and run them as native `systemd` background services.

Running bare-metal gives you the absolute lowest memory footprint and direct access to host system hardware metrics.

---

## 1. Prerequisites

- A systemd-compatible Linux distribution.
- `wget` or `curl` installed.
- Root or `sudo` privileges.

---

## 2. Directory Structure Setup

AMUD uses `/opt/amud/` as its default directory for database files, UI assets, and runtime socket communications.

Create these directories on your server:

```bash
sudo mkdir -p /opt/amud/run /opt/amud/data /opt/amud/ui
sudo chmod 755 /opt/amud/run /opt/amud/data
```

---

## 3. Download Release Assets

We provide pre-compiled x86_64 binaries for every stable release.

Download the server, agent, and UI templates directly to your system:

```bash
# 1. Download and install amud-server
wget https://github.com/boubli/AMUD-Dashboard/releases/latest/download/amud-server
chmod +x amud-server
sudo mv amud-server /usr/local/bin/

# 2. Download and install amud-agent
wget https://github.com/boubli/AMUD-Dashboard/releases/latest/download/amud-agent
chmod +x amud-agent
sudo mv amud-agent /usr/local/bin/

# 3. Download and extract the frontend UI assets
wget https://github.com/boubli/AMUD-Dashboard/releases/latest/download/ui.tar.gz
sudo tar -xzf ui.tar.gz -C /opt/amud/ui/
```

---

## 4. Create Systemd Services

To make sure AMUD starts automatically on boot and auto-restarts if a crash occurs, configure systemd unit files.

### A. AMUD Server Service

Create `/etc/systemd/system/amud-server.service`:

```bash
sudo nano /etc/systemd/system/amud-server.service
```

Paste the following configuration:

```ini title="/etc/systemd/system/amud-server.service"
[Unit]
Description=AMUD Dashboard Server
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/amud
ExecStart=/usr/local/bin/amud-server
Restart=always
RestartSec=5
Environment=PORT=8000
Environment=DB_PATH=/opt/amud/data/amud.db
Environment=AMUD_SOCKET_PATH=/opt/amud/run/amud.sock
Environment=UI_DIR=/opt/amud/ui

[Install]
WantedBy=multi-user.target
```

### B. AMUD Agent Service

Create `/etc/systemd/system/amud-agent.service`:

```bash
sudo nano /etc/systemd/system/amud-agent.service
```

Paste the following configuration:

```ini title="/etc/systemd/system/amud-agent.service"
[Unit]
Description=AMUD Host Telemetry Agent
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/amud-agent
Restart=always
RestartSec=5
Environment=AMUD_SOCKET_PATH=/opt/amud/run/amud.sock

[Install]
WantedBy=multi-user.target
```

---

## 5. Enable and Start Services

Reload the systemd manager configuration, enable the services to start at boot, and start them now:

```bash
# Reload systemd configuration
sudo systemctl daemon-reload

# Enable and start amud-server
sudo systemctl enable --now amud-server

# Enable and start amud-agent
sudo systemctl enable --now amud-agent
```

---

## 6. Verification

### Check service statuses:
```bash
sudo systemctl status amud-server
sudo systemctl status amud-agent
```

Both services should report `active (running)`.

### Verify Unix Socket creation:
Check that the IPC socket is present in the runtime directory:
```bash
ls -la /opt/amud/run/
```
You should see `amud.sock` listed.

---

## 7. Accessing the Dashboard

Navigate to your server's IP address on port 8000:
```
http://<YOUR_SERVER_IP>:8000/
```

:::tip Default Admin Credentials
- **Username:** `admin`
- **Password:** `admin` (or `password` depending on version config)
:::
