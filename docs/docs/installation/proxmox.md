---
sidebar_position: 1
---

# Proxmox Installation

AMUD features an **Autopilot Installer** specifically built for Proxmox VE. The script will automatically spin up an ultra-lean Debian 12 LXC container to host the AMUD Server, and it will install the AMUD Telemetry Agent directly onto your Proxmox host.

## Quick Install Command

Run the following command in your Proxmox Host shell (as root):

```bash
curl -sSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/setup-amud.sh | bash
```

## What the Script Does

1. **Host Agent Installation:**
   - Downloads the compiled `amud-agent` binary.
   - Creates a secure `systemd` service for the agent.
   - Starts the agent to begin collecting host metrics and LXC states via `pvesh`.
2. **LXC Server Creation:**
   - Automatically downloads the official Debian 12 standard LXC template if not present.
   - Spins up a highly optimized container (ID: 101, Name: `amud-dashboard`).
   - Configures a bind-mount to share the secure Unix socket between the Host Agent and the Server.
   - Downloads the `amud-server` and `ui.tar.gz` templates inside the container.
   - Creates and starts the server `systemd` service.

## Accessing the Dashboard

Once the script completes, it will output the IP address of your new dashboard.

Open your browser and navigate to:
```
http://<YOUR_PROXMOX_IP>:8000/
```

> [!TIP]
> **Default Admin Login:**
> Username: `admin`
> Password: `password`

## Updating

To update AMUD to the latest release, run the updater script on your Proxmox Host:

```bash
curl -sSL https://github.com/boubli/AMUD-Dashboard/releases/latest/download/update-amud.sh | bash
```
