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
   - Starts the agent to begin collecting host metrics and LXC states natively via the **Proxmox REST API** (no `pvesh`/Python subprocess).
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

## Proxmox Telemetry Configuration

AMUD communicates **directly with the Proxmox VE REST API** for container telemetry. The agent issues native, lightweight HTTPS requests over `hyper` instead of shelling out to the `pvesh` Python CLI, dramatically reducing CPU and memory overhead on the host.

To enable LXC telemetry, provide the agent with a Proxmox API token.

### 1. Create an API Token

In the Proxmox web UI:

1. Navigate to **Datacenter → Permissions → API Tokens**.
2. Click **Add**.
3. Select the **User** the token belongs to (e.g. `root@pam`).
4. Enter a **Token ID** (e.g. `amud`).
5. *(Optional)* Leave **Privilege Separation** unchecked to inherit the user's permissions, or assign explicit `VM.Audit` / `Sys.Audit` rights on the relevant nodes.
6. Click **Add**, then **copy the Secret value immediately** — Proxmox displays it only once.

### 2. Set the Environment Variable

Pass the credential to the agent via `PVE_API_TOKEN`. It must contain the **entire** value, including the `PVEAPIToken=` scheme prefix:

```bash
PVE_API_TOKEN=PVEAPIToken=USER@REALM!TOKENID=SECRET
```

For example:

```bash
PVE_API_TOKEN=PVEAPIToken=root@pam!amud=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

> If this variable is unset, the agent skips Proxmox polling — host CPU/RAM/disk metrics still work normally.

### 3. Add it to `docker-compose.yml`

```yaml
services:
  amud-agent:
    image: boubli/amud-agent:latest
    environment:
      - PVE_API_TOKEN=PVEAPIToken=root@pam!amud=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    volumes:
      - /opt/amud/run:/opt/amud/run
    restart: unless-stopped
```

> 📖 For the complete walkthrough, see the [AMUD Dashboard Installation Guide](http://tradmss.me/AMUD-Dashboard/).

## Updating

To update AMUD to the latest release, run the updater script on your Proxmox Host:

```bash
curl -sSL https://github.com/boubli/AMUD-Dashboard/releases/latest/download/update-amud.sh | bash
```
