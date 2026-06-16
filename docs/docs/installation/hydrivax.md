---
sidebar_position: 2
---

# HydrivaX LXC Installation

AMUD supports deployment inside the custom **HydrivaX OS LXC** container template. The installation script automatically provisions a container using the lightweight, custom Debian-based HydrivaX LXC template, and deploys the compiled, native AMUD Telemetry Agent directly onto your Proxmox VE host.

---

## 1. Architecture Overview

AMUD runs as a decoupled system of two lightweight Rust daemons:

```
+---------------------------------------------------------------------------------+
|                                 PROXMOX HOST                                    |
|                                                                                 |
|  +-----------------------+                    +------------------------------+  |
|  |  Proxmox VE REST API  |                    |      amud-agent (Host)       |  |
|  |  (LXC Container List) |                    |  - Polls local system info   |  |
|  +-----------+-----------+                    |  - Communicates with PVE API |  |
|              ^                                +--------------+---------------+  |
|              | (localhost HTTPS:8006 API)                    |                  |
|              v                                               v                  |
|  +---------------------------+                +------------------------------+  |
|  |     pveproxy service      |                |      /opt/amud/run/          |  |
|  +---------------------------+                |        amud.sock             |  |
|                                               +--------------+---------------+  |
|                                                              ^                  |
|  +-----------------------------------------------------------+---------------+  |
|  |  HYDRIVAX LXC CONTAINER (hydrivax-amud)                   | Bind Mount    |  |
|  |                                                           v                  |
|  |  +--------------------------------------------------------+-----------+  |  |
|  |  |  amud-server (Rust Server)                                         |  |  |
|  |  |  - Serves HTTP UI on Port 8000                                     |  |  |
|  |  |  - Streams metrics to Browser via WebSockets                       |  |  |
|  |  +--------------------------------------------------------------------+  |  |
|  +--------------------------------------------------------------------------+  |
+---------------------------------------------------------------------------------+
```

1. **`amud-agent` (runs on host)**: A high-frequency telemetry collector. It reads host hardware statistics directly from `/proc` and `/sys` interfaces, and queries the local Proxmox REST API (via port 8006) to fetch container states. It writes this metrics data to a Unix Domain socket.
2. **`amud-server` (runs in HydrivaX LXC container)**: A web backend built with Tokio and Axum. It reads telemetry data from the shared Unix socket, manages configuration settings in an embedded SQLite database, and broadcasts updates to connected browsers using persistent WebSockets.

---

## 2. Quick Install Command

Run the following command in your Proxmox Host shell as the `root` user:

```bash
curl -sSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/setup-hydrivax.sh | bash
```

### What the Script Installs and Configures
1. **Agent Setup**:
   - Downloads the pre-compiled native `amud-agent` binary and installs it to `/usr/local/bin/amud-agent`.
   - Creates a dedicated `systemd` service (`amud-agent.service`) configured to auto-restart.
   - Initializes `/opt/amud/run` for runtime IPC socket files.
2. **LXC Container Provisioning**:
   - Checks if the custom `HydrivaX-v2.5-LXC.tar.zst` template is present in your local Proxmox storage (`/var/lib/vz/template/cache`). If not, it downloads it directly from the public GitHub Releases page of the `boubli/HydrivaX` repository.
   - Spins up an unprivileged LXC named `hydrivax-amud` (1 CPU Core, 512MB RAM, 4GB Disk, Nesting & Keyctl enabled).
   - Establishes a secure bind-mount mapping `/opt/amud/run` on the host to `/opt/amud/run` in the container.
3. **Server Deployment**:
   - Sets up `/opt/amud/data` inside the container for the SQLite database.
   - Installs the `amud-server` binary and extracts UI assets to `/opt/amud/ui`.
   - Registers and starts the `amud.service` daemon.

---

## 3. LXC Console Login & Password Setup

If you want to log directly into the LXC container's terminal (via the Proxmox Web Console or SSH), you need to set a password for the `root` user. By default, custom templates do not have a pre-configured root password.

You can set or reset the password directly from your **Proxmox Host shell** without needing the old password:

```bash
# Set the password for the root user of your container
pct set <CONTAINER_ID> -password

# Alternatively, enter a direct shell inside the container to configure it
pct enter <CONTAINER_ID>
```

---

## 4. Initial Dashboard Access

Once the installation completes, the script displays the container's IP address.

1. Open your browser and navigate to:
   ```
   http://<YOUR_LXC_IP>:8000/
   ```
2. Log in using the default administrator credentials:
   - **Username**: `admin`
   - **Password**: `password`

:::warning Change Default Password Immediately
Log in, navigate to **Settings → Admin Profile**, and change the administrator password immediately to secure your installation.
:::

---

## 5. Proxmox API Token Configuration

To display live status badges (**RUNNING**, **STOPPED**) for other containers on your dashboard, the `amud-agent` needs a Proxmox API token. Without a token, the agent will report host metrics (CPU/RAM/Disk), but your app cards will remain stuck on **"CHECKING..."**.

You can read the configuration steps for generating a restricted, read-only PVE user and token in the main [Proxmox VE Installation Guide](./proxmox.md#5-proxmox-api-token-configuration).

---

## 6. Saving and Testing Token in AMUD

Once you have your API token, configure it using AMUD's built-in settings panel:

1. Log into your **AMUD Dashboard** as an administrator.
2. Click the **Settings** gear icon in the top right.
3. Select the **Proxmox** tab.
4. Paste the complete API token value into the input field:
   ```
   PVEAPIToken=amud@pve!amud-token=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
   ```
5. Click **Test Connection** to confirm connectivity.
6. Click **Save Settings**.

---

## 7. Upgrading AMUD

To upgrade the AMUD binaries and assets running inside your HydrivaX LXC container and host, run the updater on the **Proxmox host** as root:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/update-amud.sh)
```
