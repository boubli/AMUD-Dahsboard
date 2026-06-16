---
sidebar_position: 4
---

# Troubleshooting

This guide covers the most common issues users encounter when deploying AMUD on Proxmox VE, and how to diagnose and resolve them quickly.

---

## Find Your AMUD Container ID

The autopilot installer creates an LXC named `amud-dashboard`, but the numeric ID depends on your cluster (it is **not always `101`**). Before running any `pct exec` command below, look up your container ID on the Proxmox host:

```bash
pct list | awk '$3 == "amud-dashboard" {print $1}' | head -n1
```

Or list all containers and pick the ID for `amud-dashboard`:

```bash
pct list
```

In the examples below, replace `<CT_ID>` with that number (for example `102`, `111`, etc.).

---

## Apps Stuck on "CHECKING..."

**Symptom:** Your dashboard loads correctly and shows live CPU, RAM, and Disk metrics in the top bar, but individual application cards display a grey **"CHECKING..."** badge instead of **RUNNING** or **STOPPED**.

**Root Cause:** The AMUD agent cannot retrieve the list of LXC containers from the Proxmox API. This is almost always caused by a missing or misconfigured API token.

### Diagnostic Steps

**Step 1 — Check the agent logs:**

```bash
journalctl -u amud-agent --no-pager -n 20
```

Look for lines starting with `[LXC]`. The log output will tell you exactly what's wrong:

| Log Message | Meaning | Fix |
|---|---|---|
| `PVE_API_TOKEN not set or empty` | The agent cannot find the token | [Set the API token →](#fix-1--set-the-api-token) |
| `Successfully fetched 0 containers` | Token works but has no permissions | [Disable Privilege Separation →](#fix-2--disable-privilege-separation) |
| `PVE API returned HTTP 401` | Token secret is invalid or expired | [Recreate the token →](#fix-3--recreate-the-token) |
| `PVE API returned HTTP 500/595` | Wrong Proxmox node name | [Check hostname →](#fix-4--check-node-hostname) |
| `HTTP request to PVE API failed` | Cannot reach Proxmox API on port 8006 | [Check firewall →](#fix-5--check-network-connectivity) |
| No `[LXC]` lines at all | Agent binary is outdated | [Update the agent →](#fix-6--update-the-agent) |

---

### Fix 1 — Set the API Token

If you see `PVE_API_TOKEN not set or empty`, the agent's systemd service file doesn't have the token configured.

1. Create an API token in Proxmox (see [Proxmox Installation → API Token Setup](/docs/installation/proxmox#5-proxmox-api-token-configuration)).
2. Edit the agent service file:

```bash
nano /etc/systemd/system/amud-agent.service
```

3. Add this line under `[Service]` (replace with your actual token):

```ini
Environment="PVE_API_TOKEN=PVEAPIToken=root@pam!amud=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
```

4. Restart:

```bash
systemctl daemon-reload
systemctl restart amud-agent
```

---

### Fix 2 — Disable Privilege Separation

If you see `Successfully fetched 0 containers from PVE`, your token is valid but **Privilege Separation** is blocking it from reading any data.

:::danger This is the #1 most common issue
Proxmox enables **Privilege Separation** by default when creating API tokens. When enabled, the token has **zero permissions** — even if it belongs to `root@pam`. The API will authenticate your request successfully (HTTP 200) but return an empty container list.
:::

**To fix this:**

1. Go to **Proxmox Web UI → Datacenter → Permissions → API Tokens**.
2. **Delete** the existing token.
3. Click **Add** to create a new one:
   - User: `root@pam`
   - Token ID: `amud`
   - **⚠️ UNCHECK "Privilege Separation"**
4. Copy the new secret.
5. Update the agent service file with the new token:

```bash
nano /etc/systemd/system/amud-agent.service
```

6. Replace the old `PVE_API_TOKEN` line and restart:

```bash
systemctl daemon-reload
systemctl restart amud-agent
```

7. Verify:

```bash
journalctl -u amud-agent --no-pager -n 10
```

You should now see: `Successfully fetched XX containers from PVE.`

---

### Fix 3 — Recreate the Token

If you see `PVE API returned HTTP 401`, the token secret is invalid. This can happen if:
- You copied the secret incorrectly (missing characters)
- The token was deleted or regenerated in Proxmox
- There is a typo in the service file

**To fix this:**

1. Go to **Proxmox → Datacenter → Permissions → API Tokens**.
2. Delete the old token and create a new one (with **Privilege Separation unchecked**).
3. Carefully copy the entire secret value.
4. Update the service file, making sure:
   - The format is exactly: `PVEAPIToken=user@realm!tokenid=secret`
   - The line has both opening **and closing** double quotes
   - There are no trailing spaces or newlines

```ini
# ✅ Correct
Environment="PVE_API_TOKEN=PVEAPIToken=root@pam!amud=4af82325-36a8-4e24-ab33-0fd71276e31b"

# ❌ Wrong — missing closing quote
Environment="PVE_API_TOKEN=PVEAPIToken=root@pam!amud=4af82325-36a8-4e24-ab33-0fd71276e31b

# ❌ Wrong — missing PVEAPIToken= prefix
Environment="PVE_API_TOKEN=root@pam!amud=4af82325-36a8-4e24-ab33-0fd71276e31b"
```

---

### Fix 4 — Check Node Hostname

If you see `PVE API returned HTTP 500` or `HTTP 595`, the agent may be using the wrong Proxmox node name.

The agent automatically reads the hostname from `/etc/hostname` on the Proxmox host. You can verify it matches your actual node name:

```bash
# Check what the agent will use
cat /etc/hostname

# Compare with your actual Proxmox node name
pvesh get /nodes --output-format json | grep node
```

If they don't match, update `/etc/hostname` to match your Proxmox node name and restart the agent.

---

### Fix 5 — Check Network Connectivity

If you see `HTTP request to PVE API failed`, the agent cannot reach the Proxmox API on `https://localhost:8006`.

Verify the Proxmox API is accessible locally:

```bash
curl -k https://localhost:8006/api2/json/version
```

If this fails, check:
- Is the `pveproxy` service running? (`systemctl status pveproxy`)
- Is a firewall blocking port 8006 on localhost? (`iptables -L -n`)

---

### Fix 6 — Update the Agent

If you see no `[LXC]` log lines at all, your agent binary is outdated and doesn't include the diagnostic logging or the dynamic hostname fix.

Update to the latest version:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/update-amud.sh)
```

---

## App Name Matching

Even when LXC data is flowing correctly, an app card may still show "CHECKING..." if the **Application Name** in the AMUD dashboard doesn't match the **LXC container name** in Proxmox.

AMUD uses fuzzy matching — the app name and LXC name only need to **partially overlap** (case-insensitive). For example:

| Dashboard App Name | Proxmox LXC Name | Match? |
|---|---|---|
| `jellyfin` | `jellyfin` | ✅ Exact match |
| `Nginx Proxy Manager` | `nginx-proxy-manager` | ❌ No match (hyphens vs spaces) |
| `qbittorrent` | `qbittorrent` | ✅ Exact match |
| `immich` | `immich-server` | ✅ Partial match (`immich` is inside `immich-server`) |
| `My Plex Server` | `plex` | ✅ Partial match (`plex` is inside `my plex server`) |

**Best practice:** Name your apps in the dashboard to match (or contain) the exact LXC container name shown in Proxmox.

---

## Dashboard Refuses Connection After Update

**Symptom:** After running `update-amud.sh`, the updater reports success, but opening `http://<LXC_IP>:8000` shows **ERR_CONNECTION_REFUSED** or **This site can't be reached**.

**Most likely cause:** The AMUD server is running inside the LXC, but it is listening only on `127.0.0.1:8000` inside the container instead of the LXC network interface. This can happen after upgrading to a release where the secure default bind address changed to `127.0.0.1`.

### Diagnostic Steps

Run these commands from the Proxmox host:

```bash
pct exec <CT_ID> -- systemctl status amud
pct exec <CT_ID> -- ss -tlnp | grep 8000
```

If the `ss` output shows `127.0.0.1:8000`, the dashboard is only reachable from inside the LXC. Set the LXC service bind address to all interfaces:

```bash
pct exec <CT_ID> -- bash -c "grep -q '^Environment=BIND_ADDR=' /etc/systemd/system/amud.service && sed -i 's|^Environment=BIND_ADDR=.*|Environment=BIND_ADDR=0.0.0.0|' /etc/systemd/system/amud.service || sed -i '/^\[Service\]/a Environment=BIND_ADDR=0.0.0.0' /etc/systemd/system/amud.service"
pct exec <CT_ID> -- systemctl daemon-reload
pct exec <CT_ID> -- systemctl restart amud
pct exec <CT_ID> -- ss -tlnp | grep 8000
```

The final command should show `0.0.0.0:8000`. Then open:

```text
http://<LXC_IP>:8000
```

If nothing is listening on port `8000`, check the server logs instead:

```bash
pct exec <CT_ID> -- journalctl -u amud -n 80 --no-pager
```

:::info Fixed in newer updater scripts
Recent versions of `setup-amud.sh` and `update-amud.sh` set `BIND_ADDR=0.0.0.0` automatically for Proxmox LXC deployments.
:::

---

## Agent Keeps Disconnecting (Broken Pipe)

**Symptom:** You see repeated `Broken pipe (os error 32)` errors in the agent logs.

**Cause:** This happens when the AMUD server inside the LXC container restarts (e.g., during an update) and the Unix socket connection is interrupted.

**This is normal behavior.** The agent will automatically reconnect within 5 seconds. If it keeps happening continuously without recovery:

1. Check the server is running inside the LXC:

```bash
pct exec <CT_ID> -- systemctl status amud
```

2. Check the socket file exists and is writable:

```bash
ls -la /opt/amud/run/amud.sock
```

3. Restart both services:

```bash
pct exec <CT_ID> -- systemctl restart amud
systemctl restart amud-agent
```

---

## Dashboard Loads but Shows No Metrics

**Symptom:** The dashboard loads, you can log in, but the CPU/RAM/Disk bars are all at 0% and no data appears.

**Cause:** The agent is not connected to the server.

1. Check the agent status:

```bash
systemctl status amud-agent
```

2. Check the socket bind-mount is configured:

```bash
grep mp0 /etc/pve/lxc/<CT_ID>.conf
```

You should see: `mp0: /opt/amud/run,mp=/opt/amud/run`

3. If missing, add it manually and restart:

```bash
echo "mp0: /opt/amud/run,mp=/opt/amud/run" >> /etc/pve/lxc/<CT_ID>.conf
pct reboot <CT_ID>
systemctl restart amud-agent
```

---

## Docker / Portainer: Permission Denied on docker.sock

**Symptom:** The agent log reports errors like:
```
[Docker] Error connecting to docker socket: Permission denied (os error 13)
```

**Cause:** The user executing the agent inside the container does not have permission to read/write the mapped `/var/run/docker.sock` on the host system.

**Fixes:**

1. **Run container as root (Recommended):**
   In your `docker-compose.yml`, make sure the `amud-agent` container runs as root. You can do this by omitting user specifications, as default is root, which has access to the socket.
2. **Change host socket permissions:**
   Alternatively, grant read/write access to the docker socket on the host machine:
   ```bash
   sudo chmod 666 /var/run/docker.sock
   ```

---

## Reverse Proxy: WebSockets Disconnect (0% Metrics)

**Symptom:** The dashboard Web UI loads fine, but all host metrics (CPU/RAM/Disk) remain at `0%` and no live statuses stream. In your browser console (F12), you see errors like:
```
WebSocket connection to 'wss://amud.yourdomain.com/ws' failed: Error during WebSocket handshake: Unexpected response code: 400
```

**Cause:** The reverse proxy (Nginx, NPM, Apache) is forwarding HTTP traffic but stripping the headers required to "upgrade" the connection to WebSockets.

**Fixes:**

1. **Verify WebSocket Headers:** Refer to the [Reverse Proxy Configuration](/docs/installation/reverse-proxy) guide and ensure the following headers are set in your proxy block:
   ```nginx
   proxy_http_version 1.1;
   proxy_set_header Upgrade $http_upgrade;
   proxy_set_header Connection "upgrade";
   ```
2. **Nginx Proxy Manager:** Edit your proxy host in the NPM interface, check the **Websockets Support** toggle box, and click **Save**.
3. **Cloudflare Tunnels:** Ensure WebSockets are enabled under your domain's **Network** settings in the Cloudflare dashboard.

---

## Database is Locked or Permission Denied

**Symptom:** You receive errors like `database is locked` or `ReadOnly / Permission Denied` when trying to save settings or add apps in the dashboard UI.

**Cause:**
- Multiple instances of `amud-server` are running and contesting access to the SQLite database.
- The user running the `amud-server` service does not have write access to `/opt/amud/data/` or `/opt/amud/data/amud.db`.

**Fixes:**

1. **Check for duplicate server processes:**
   ```bash
   ps aux | grep amud-server
   ```
   If multiple processes are running, stop the service and terminate all duplicates:
   ```bash
   sudo systemctl stop amud-server
   sudo killall amud-server
   sudo systemctl start amud-server
   ```
2. **Fix file permissions:**
   Ensure the directory and database file are writable:
   ```bash
   sudo chmod -R 777 /opt/amud/data
   ```

---

## Reset or Change the Admin Password from CLI

Use this when you cannot log in to the AMUD UI after an upgrade, browser session reset, or credential mistake.

### See Existing Users

Run on the Proxmox host:

```bash
pct exec <CT_ID> -- sqlite3 /opt/amud/data/amud.db "SELECT id, username, role FROM users;"
```

If `sqlite3` is missing inside the LXC:

```bash
pct exec <CT_ID> -- apt-get update
pct exec <CT_ID> -- apt-get install -y sqlite3
```

### Retrieve Initial Bootstrap Password

If this is a fresh installation and you did not save the random bootstrap password printed by the installer, you can retrieve it directly from the systemd journal logs. 

Run this command on your **Proxmox Host**:

```bash
pct exec <CT_ID> -- journalctl -u amud --no-pager | grep 'Password:'
```

It will print the seeded admin password, for example:
```text
amud-server[123]:    Password: xxxxx-xxxxx-xxxxx
```

### Reset `admin` Password to `admin`

```bash
pct exec <CT_ID> -- sqlite3 /opt/amud/data/amud.db "INSERT INTO users (username,password_hash,role) VALUES ('admin','8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918','Admin') ON CONFLICT(username) DO UPDATE SET password_hash=excluded.password_hash, role='Admin';"
pct exec <CT_ID> -- systemctl restart amud
```

Then sign in with:

```text
username: admin
password: admin
```

Change the password immediately in **Settings → Security**.

### Change Any User Password from CLI

AMUD stores new passwords as Argon2id hashes. For emergency CLI recovery, you can still write a legacy SHA-256 hash; AMUD accepts it once and transparently upgrades it to Argon2id after the next successful login.

```bash
NEW_PASSWORD='your-new-password'
HASH=$(printf '%s' "$NEW_PASSWORD" | sha256sum | awk '{print $1}')
pct exec <CT_ID> -- sqlite3 /opt/amud/data/amud.db "UPDATE users SET password_hash='$HASH' WHERE username='admin';"
pct exec <CT_ID> -- systemctl restart amud
```

After signing in, use **Settings → Security** to set the password again so a fresh Argon2id hash is stored immediately.

---

## AMUD Agent Secret / IPC Authentication

`AMUD_AGENT_SECRET` is not your Proxmox API token. It is an internal shared secret between the dashboard server inside the LXC and the host telemetry agent.

### Check Both Sides Match

```bash
grep AMUD_AGENT_SECRET /etc/systemd/system/amud-agent.service
pct exec <CT_ID> -- grep AMUD_AGENT_SECRET /etc/systemd/system/amud.service
systemctl show amud-agent -p Environment
```

If the values differ, copy the container value to the host service:

```bash
SECRET=$(pct exec <CT_ID> -- grep AMUD_AGENT_SECRET /etc/systemd/system/amud.service | cut -d= -f2)
sed -i "s|^Environment=AMUD_AGENT_SECRET=.*|Environment=AMUD_AGENT_SECRET=${SECRET}|" /etc/systemd/system/amud-agent.service
systemctl daemon-reload
systemctl restart amud-agent
pct exec <CT_ID> -- systemctl restart amud
```

### Verify Recent Logs Only

Old rejected connections can remain in `journalctl`. Check logs after restarting:

```bash
journalctl -u amud-agent --no-pager --since "2 minutes ago"
pct exec <CT_ID> -- journalctl -u amud --no-pager --since "2 minutes ago"
```

---

## Update or Release Recovery

If an update stops halfway through, restart both services first:

```bash
pct exec <CT_ID> -- systemctl restart amud
systemctl restart amud-agent
```

Then rerun the fixed updater from `main`:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/update-amud.sh)
```

Check release assets before publishing a new GitHub release:

```bash
gh release view vX.Y.Z --repo boubli/AMUD-Dashboard
gh release download vX.Y.Z --repo boubli/AMUD-Dashboard --pattern SHA256SUMS --dir /tmp/amud-release-check
```

Required assets:

- `amud-server`
- `amud-agent`
- `ui.tar.gz`
- `SHA256SUMS`
- `setup-amud.sh`
- `update-amud.sh`
- `uninstall-amud.sh`

---

## PWA / Browser Cache Issues

After upgrading AMUD, stale service worker cache can make the UI look old or keep old JavaScript loaded.

Try in this order:

```text
1. Hard refresh: Ctrl+Shift+R
2. Open DevTools → Application → Service Workers → Unregister
3. DevTools → Application → Storage → Clear site data
4. Reload the dashboard
```

If using a reverse proxy, also verify `/static/sw.js`, `/static/manifest.json`, and `/ws` are not blocked or rewritten.

---

## Media Integrations Not Showing Streams

Real Jellyfin and Plex playback requires API credentials in **Settings → Integrations**.

### Jellyfin

Set:

- `Jellyfin URL`, for example `http://jellyfin.local:8096`
- `Jellyfin API Key`

AMUD polls:

```text
GET /Sessions
X-Emby-Token: <api-key>
```

### Plex

Set:

- `Plex URL`, for example `http://plex.local:32400`
- `Plex Token`

AMUD polls:

```text
GET /status/sessions
Header: X-Plex-Token: <token>
```

If no credentials are configured, the stream cards show **NOT CONFIGURED** instead of fake playback.

---

## Useful Service Commands

Run from the Proxmox host. Replace `<CT_ID>` with your `amud-dashboard` container ID (see [Find Your AMUD Container ID](#find-your-amud-container-id)):

```bash
# Dashboard service inside LXC
pct exec <CT_ID> -- systemctl status amud
pct exec <CT_ID> -- systemctl restart amud
pct exec <CT_ID> -- journalctl -u amud --no-pager -n 50

# Host telemetry agent
systemctl status amud-agent
systemctl restart amud-agent
journalctl -u amud-agent --no-pager -n 50

# Confirm socket mount
grep mp0 /etc/pve/lxc/<CT_ID>.conf
ls -la /opt/amud/run
```

---

## Recovering from Broken Custom CSS

**Symptom:** You entered custom CSS into the Settings menu that caused the dashboard to become entirely unusable (e.g., hiding the body, breaking the grid, making buttons unclickable), and now you cannot reach the Settings menu to fix it.

**Fix:** You can reset the custom CSS directly from the SQLite database via the CLI:

```bash
pct exec <CT_ID> -- sqlite3 /opt/amud/data/amud.db "UPDATE settings SET value='' WHERE key='custom_css';"
pct exec <CT_ID> -- systemctl restart amud
```

---

## Database Restore Failure

**Symptom:** You attempted to upload an `amud.db` backup file via the UI, but the UI crashed, or the server failed to restart correctly with the new database.

**Fix:** When uploading a database, AMUD automatically saves your original database as `amud.db.bak` before applying the new one. You can easily revert from the CLI:

```bash
pct exec <CT_ID> -- mv /opt/amud/data/amud.db.bak /opt/amud/data/amud.db
pct exec <CT_ID> -- systemctl restart amud
```

---

## Getting Help

If your issue isn't covered here:

1. Run the diagnostic command and save the output:

```bash
journalctl -u amud-agent --no-pager -n 50 > /tmp/amud-debug.log
pct exec <CT_ID> -- journalctl -u amud --no-pager -n 50 >> /tmp/amud-debug.log
cat /tmp/amud-debug.log
```

2. Open an issue on [GitHub](https://github.com/boubli/AMUD-Dashboard/issues) with the log output and your Proxmox version.

