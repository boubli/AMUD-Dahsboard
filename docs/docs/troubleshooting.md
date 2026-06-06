---
sidebar_position: 3
---

# Troubleshooting

This guide covers the most common issues users encounter when deploying AMUD on Proxmox VE, and how to diagnose and resolve them quickly.

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

1. Create an API token in Proxmox (see [Proxmox Installation → API Token Setup](/docs/installation/proxmox#proxmox-api-token-setup-required-for-lxc-monitoring)).
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

## Agent Keeps Disconnecting (Broken Pipe)

**Symptom:** You see repeated `Broken pipe (os error 32)` errors in the agent logs.

**Cause:** This happens when the AMUD server inside the LXC container restarts (e.g., during an update) and the Unix socket connection is interrupted.

**This is normal behavior.** The agent will automatically reconnect within 5 seconds. If it keeps happening continuously without recovery:

1. Check the server is running inside the LXC:

```bash
pct exec 101 -- systemctl status amud
```

2. Check the socket file exists and is writable:

```bash
ls -la /opt/amud/run/amud.sock
```

3. Restart both services:

```bash
pct exec 101 -- systemctl restart amud
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
grep mp0 /etc/pve/lxc/101.conf
```

You should see: `mp0: /opt/amud/run,mp=/opt/amud/run`

3. If missing, add it manually and restart:

```bash
echo "mp0: /opt/amud/run,mp=/opt/amud/run" >> /etc/pve/lxc/101.conf
pct reboot 101
systemctl restart amud-agent
```

---

## Getting Help

If your issue isn't covered here:

1. Run the diagnostic command and save the output:

```bash
journalctl -u amud-agent --no-pager -n 50 > /tmp/amud-debug.log
pct exec 101 -- journalctl -u amud --no-pager -n 50 >> /tmp/amud-debug.log
cat /tmp/amud-debug.log
```

2. Open an issue on [GitHub](https://github.com/boubli/AMUD-Dashboard/issues) with the log output and your Proxmox version.
