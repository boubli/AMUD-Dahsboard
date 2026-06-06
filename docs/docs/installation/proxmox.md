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
http://<YOUR_LXC_IP>:8000/
```

:::tip Default Admin Login
Username: `admin`  
Password: `password`
:::

---

## Proxmox API Token Setup (Required for LXC Monitoring)

AMUD communicates **directly with the Proxmox VE REST API** for live container telemetry. Without an API token, the agent will still stream host-level CPU, RAM, and disk metrics — but your app cards will remain stuck on **"CHECKING..."** because the agent cannot query your LXC containers.

:::warning This step is mandatory for LXC status monitoring
If you skip this section, your dashboard will work but all application cards will show **"CHECKING..."** instead of live **RUNNING** / **STOPPED** badges.
:::

### Step 1 — Create an API Token in Proxmox

1. Open the **Proxmox Web UI** (typically `https://YOUR_IP:8006`).
2. Navigate to **Datacenter → Permissions → API Tokens**.
3. Click **Add**.
4. Fill in the fields:
   - **User:** `root@pam` (or any user with at least `VM.Audit` + `Sys.Audit` privileges)
   - **Token ID:** `amud`
5. **⚠️ UNCHECK "Privilege Separation"** — this is the most critical step.

:::danger Privilege Separation Must Be Unchecked
By default, Proxmox enables **Privilege Separation** when creating API tokens. When this is ON, your token starts with **zero permissions** — even if it belongs to `root@pam`. The Proxmox API will return an empty container list, and your app cards will remain stuck on "CHECKING...".

**You MUST uncheck this checkbox** to allow the token to inherit the user's permissions.
:::

6. Click **Add**, then **immediately copy the Secret value** — Proxmox displays it **only once**.

The full token credential will look like this:

```
PVEAPIToken=root@pam!amud=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

### Step 2 — Configure the Agent Service

Edit the `amud-agent` systemd service file on your **Proxmox host**:

```bash
nano /etc/systemd/system/amud-agent.service
```

Add the `PVE_API_TOKEN` environment variable under the `[Service]` section. Your file should look like this:

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
Environment="PVE_API_TOKEN=PVEAPIToken=root@pam!amud=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

[Install]
WantedBy=multi-user.target
```

:::caution Make sure to include the closing quote
The `Environment` line must have **both** an opening `"` and a closing `"` around the value. Missing the closing quote will cause systemd to fail silently.
:::

### Step 3 — Restart the Agent

Save the file (`Ctrl+O`, `Enter`, `Ctrl+X` in nano) and restart the agent:

```bash
systemctl daemon-reload
systemctl restart amud-agent
```

### Step 4 — Verify It Works

Check the agent logs to confirm it's fetching your containers:

```bash
journalctl -u amud-agent --no-pager -n 15
```

You should see output like this:

```
[LXC] Fetching containers from: https://localhost:8006/api2/json/nodes/YOUR_NODE/lxc
[LXC] Successfully fetched 20 containers from PVE.
```

If you see `Successfully fetched 0 containers`, refer to the [Troubleshooting Guide](/docs/troubleshooting) below.

---

## Minimal Permissions Token (Advanced)

If you prefer not to use `root@pam` or want tighter security, you can create a dedicated user with minimal permissions:

1. **Create a new user** in Proxmox (e.g., `amud@pve`).
2. **Assign permissions** — the agent only needs read access:
   - `VM.Audit` on `/vms` (to list and read LXC container status)
   - `Sys.Audit` on `/nodes` (to read node information)
3. **Create an API token** for `amud@pve` with **Privilege Separation unchecked**.
4. Use the new token in the agent service file.

```bash
# Example: Create a PVE user and assign audit-only permissions
pveum user add amud@pve
pveum aclmod / -user amud@pve -role PVEAuditor
pveum user token add amud@pve amud --privsep 0
```

---

## Updating

To update AMUD to the latest release, run the updater script on your Proxmox Host:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/update-amud.sh)
```

:::note
The update script preserves your database, settings, and API token configuration. It only replaces the server binary, UI assets, and agent binary.
:::
