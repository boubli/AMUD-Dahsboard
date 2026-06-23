---
sidebar_position: 2
---

# Dashboard Configuration

Most AMUD settings are managed in the web UI under **Settings** (admin login required). This page covers appearance layout and media integrations.

---

## Appearance

Open **Settings → Appearance** to customize the dashboard look and layout.

### Grid columns

The **Grid columns** dropdown controls how many app cards appear per row on desktop screens:

| Value | Use case |
|-------|----------|
| 2 | Large cards, fewer columns |
| 3 | Default balanced layout |
| 4 | Dense homelab with many services |
| 5 | Maximum density on wide monitors |

The setting is stored as `grid_columns` in the SQLite settings table (default: `3`). On smaller viewports, the layout automatically reflows to fewer columns for mobile and tablet screens.

Other appearance options on the same tab include accent color, glass blur/opacity, background image, logo, and bento card radius.

---

## Media integrations (Jellyfin & Plex)

Open **Settings → Integrations** to connect live stream detection for Jellyfin and Plex cards on the dashboard.

When credentials are missing, stream badges show **NOT CONFIGURED**. When configured, badges start as **CHECKING...** and update from live session polling plus LXC/Docker telemetry.

### Jellyfin

| Field | Description |
|-------|-------------|
| **Jellyfin URL** | Base URL of your server, e.g. `http://jellyfin.local:8096` |
| **Jellyfin API Key** | API key from Jellyfin admin |

**Create an API key in Jellyfin:**

1. Open Jellyfin as an administrator.
2. Go to **Dashboard → Advanced → API Keys**.
3. Click **+** to create a key (e.g. name it `AMUD`).
4. Paste the key into AMUD **Settings → Integrations**.

AMUD polls:

```http
GET /Sessions
X-Emby-Token: <your-api-key>
```

The API key is sent in the `X-Emby-Token` header (Emby/Jellyfin convention), not as a query parameter.

### Plex

| Field | Description |
|-------|-------------|
| **Plex URL** | Base URL of your Plex server, e.g. `http://plex.local:32400` |
| **Plex Token** | `X-Plex-Token` for your account |

**Find your Plex token:**

1. Sign in to [plex.tv](https://app.plex.tv) and open your server, **or**
2. Use Plex's documented token claim flow for your account, **or**
3. Inspect an authenticated Plex web request in browser DevTools for the `X-Plex-Token` header.

AMUD polls:

```http
GET /status/sessions
X-Plex-Token: <your-token>
```

When multiple clients are streaming, the badge may show the primary title plus `(+N more)`.

---

## App card integrations

Per-app integrations are set when you **Add** or **Edit** an app (Integration dropdown). AMUD fetches data when the card loads.

| Integration | App URL field | Credential field | Notes |
|-------------|---------------|------------------|-------|
| **Pi-hole** | Pi-hole web UI base URL | Web password / API token | Shows ads blocked today; admin can disable 5 min |
| **AdGuard Home** | AdGuard UI base URL | Basic auth credential | **Not an API key.** Base64-encoded `username:password` for the AdGuard UI login (see below) |
| **Radarr** | Radarr base URL | API key (`X-Api-Key`) | Queue size |
| **Sonarr** | Sonarr base URL | API key | Queue size |
| **Overseerr** | Overseerr base URL | API key | Pending media requests |
| **Jellyseerr** | Jellyseerr base URL | API key | Pending media requests |
| **Prowlarr** | Prowlarr base URL | API key (`X-Api-Key`) | Enabled/total indexers + queue size |
| **Uptime Kuma** | Uptime Kuma base URL | Status page slug **or** API key | Monitors up/down (status page JSON or `/api/monitors`) |
| **Cloudflare Tunnel** | Tunnel hostname or dashboard URL | `account_id\|tunnel_id\|api_token` | Tunnel status + active connections |
| **Peanut (UPS)** | Peanut/NUT base URL | API token (optional) | Battery % and UPS status |
| **RSS / Atom** | — | — | Manage under **Settings → RSS Feeds**; top 3 headlines on `/feeds`; **visible to guests** |

RSS feeds are not added via the dashboard **Add App** modal — use **Settings → RSS Feeds** (stored as `integration_type=rss` apps).

### AdGuard Home credential (not an API key)

AdGuard Home uses **HTTP Basic authentication**, not a Pi-hole-style API token. In the Add/Edit app form:

1. Set **Integration** to **AdGuard Home**.
2. Set **URL** to your AdGuard UI base (e.g. `http://192.168.1.10:3000`).
3. In **Basic auth credential**, paste a **Base64-encoded** `username:password` string (the same credentials you use to log into AdGuard).

Generate the value on Linux/macOS:

```bash
echo -n 'admin:your-adguard-password' | base64
```

Paste the output (e.g. `YWRtaW46eW91ci1hZGd1YXJkLXBhc3N3b3Jk`) into the credential field.

On Windows PowerShell:

```powershell
[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('admin:your-adguard-password'))
```

The card shows blocked queries today and whether protection is enabled.

Enable **Accept invalid TLS certificates** under **Settings → Privacy & Access** if your homelab services use self-signed HTTPS (applies to Jellyfin, Plex, app integrations, and Home Assistant).

---

## Host telemetry mapping

Under **Settings → Privacy & Access → Host telemetry mapping**, you can override how the AMUD agent reports network and disk stats:

| Setting | Example | Behavior |
|---------|---------|----------|
| **External network interfaces** | `eth0,enp3s0` | Count only these interfaces toward external bandwidth |
| **Internal network interfaces** | `vmbr0,br-0` | Count only these toward internal bandwidth |
| **Disk mount points** | `/,/mnt/user` | Sum storage usage only from these mounts |

Leave fields **blank** for automatic detection (bridges/Docker → internal; other interfaces → external; all eligible disks → storage bar).

Changes push to the connected agent when you save settings.

### How to find your interface names and mount paths

The **amud-agent** runs on the **host** (Proxmox bare metal, Unraid, or the machine that owns the Docker socket). Run the discovery commands **on that host**, not inside the AMUD dashboard container/LXC unless the agent also runs there.

#### Find network interface names

```bash
cat /proc/net/dev
```

The first column is the interface name (ignore `lo`). Common patterns:

| Interface | Typical role | Auto mode |
|-----------|--------------|-----------|
| `eno1`, `enp3s0`, `eth0` | Physical NIC (WAN/LAN) | **External** |
| `vmbr0` | Proxmox bridge | **Internal** |
| `br-*`, `docker0` | Docker / virtual bridges | **Internal** |

**Proxmox:** run on the **hypervisor host**. Physical NIC → **External**; `vmbr0` → **Internal**.

```bash
# On the Proxmox host
cat /proc/net/dev
# Example result: eno1 (external), vmbr0 (internal)
```

**Unraid:** run in the **Unraid terminal** (or the host where **AMUD Agent** runs). Use the NIC that faces your router for **External**; Docker `br-*` bridges for **Internal** if you care about container traffic separately.

**Docker / Portainer:** the agent needs the host network view. Interface names come from the **Docker host**, not from inside the dashboard container.

#### Find disk mount paths

```bash
df -h
```

Use the **Mounted on** column. Pick the paths you want the storage bar to represent.

| Platform | Common mounts | Example setting |
|----------|---------------|-----------------|
| **Proxmox** | OS disk + VM/CT storage | `/,/var/lib/vz` |
| **Unraid** | Array / cache | `/mnt/user` or `/mnt/cache` |
| **Generic Linux** | Root only | `/` |

AMUD skips virtual/temporary filesystems (`tmpfs`, `overlay`, etc.) automatically when summing disks.

#### Fill in Settings

1. Open **Settings → Privacy & Access → Host telemetry mapping**.
2. Paste comma-separated names/paths (no spaces required, but spaces after commas are fine).
3. Click **Save Settings** at the bottom of the page.
4. Wait ~5–10 seconds — the agent picks up the new config on the next sync.

**Example (typical Proxmox):**

```
External network interfaces:  eno1
Internal network interfaces:  vmbr0
Disk mount points:            /,/var/lib/vz
```

#### Rules and tips

- **Blank = auto** — start here; only override if the dashboard disk or network numbers look wrong.
- **If you set either network list**, only interfaces you list are counted. Unlisted interfaces are ignored — fill **both** external and internal lists when overriding.
- **Verify saved values** (optional, admin shell on the AMUD server):

```bash
sqlite3 /opt/amud/data/amud.db "SELECT key, value FROM settings WHERE key LIKE 'telemetry_%';"
```

- **Still wrong?** See [Troubleshooting — Host telemetry mapping](./troubleshooting.md#host-telemetry-mapping).

---

## Smart Home Integration (Home Assistant)

Connect your dashboard to Home Assistant to view live sensor telemetry directly inside the Home Assistant app card.

1. Open **Settings → Smart Home**.
2. Enter your **Home Assistant URL** (e.g. `http://homeassistant.local:8123`).
3. Enter your **Long-Lived Access Token** (created from your user profile in HA).

If you have an application named exactly `Home Assistant` on your dashboard, its telemetry will now include the number of active lights, switches, and average home temperature.

AMUD polls Home Assistant using the lightweight **Template API** (`POST /api/template`) to compute those counts on the HA host, falling back to the full `/api/states` dump only when template rendering is unavailable.

---

## Custom CSS Injection

Make the dashboard truly yours by overriding the default styling.

1. Open **Settings → Appearance → Custom CSS**.
2. Copy CSS from the [Theme Gallery](/themes) (preview screenshots help you pick a theme), or write your own.
3. Paste into the Custom CSS field and click **Save**. Changes apply immediately for all users.

*(Note: If invalid CSS breaks the layout, see [Troubleshooting](./troubleshooting.md) for recovery.)*

Browse themes with preview screenshots on the **[Theme Gallery](/themes)** — click **Copy CSS**, paste, and save. See also the [CSS variable reference](./themes.md).

---

## Proxmox and container control

Live **RUNNING** / **STOPPED** badges and start/stop controls require:

1. A valid **Proxmox API token** under **Settings → Proxmox VE**
2. Matching **container IDs** or Docker names on app cards
3. A working **amud-agent** on the hypervisor host

See [Proxmox VE Installation](./installation/proxmox.md#5-proxmox-api-token-configuration) for token setup.

### Per-app CPU / RAM row

Each app card can show live **CPU** and **RAM** from the host agent when the container name matches. For services that run on **another server** (or cloud), those numbers are misleading.

When adding or editing an app, use **Show CPU / RAM from host agent**:

- **On** (default) — card shows agent metrics when a container match exists
- **Off** — status badge still updates; CPU/RAM row is hidden

Guests never see per-card CPU/RAM — only **ONLINE** / **OFFLINE** availability.

---

## Environment variables

These are set on the **server** or **agent** process (Docker `environment:`, systemd unit, or shell). Most day-to-day options live in the SQLite settings table via the UI.

| Variable | Component | Default | Description |
|----------|-----------|---------|-------------|
| `PORT` | Server | `8000` | HTTP listen port |
| `BIND_ADDR` | Server | `127.0.0.1` | Bind address. Use `0.0.0.0` in Docker so the container accepts external traffic. |
| `DB_PATH` | Server | `data/amud.db` | SQLite database file path |
| `AMUD_SECRETS_KEY` | Server | auto-generated file | 32-byte key (base64url or 64-char hex) for encrypting integration tokens at rest in SQLite. If unset, AMUD writes `data/.amud-secrets-key` on first boot — back it up with `amud.db`. |
| `AMUD_AGENT_SECRET` | Both | *(required)* | Shared secret for agent ↔ server IPC authentication |
| `AMUD_SOCKET_PATH` | Both | `/var/run/amud/amud.sock` | Unix socket path for agent IPC |
| `AMUD_ENABLE_PROXMOX` | Server | `false` | Set `true` on a Proxmox LXC host to show the Proxmox settings tab |
| `AMUD_DOCKER` | Agent | `0` | Set `1` to enable Docker socket monitoring (requires socket mount) |
| `PVE_NODE` | Agent | hostname | Proxmox node name for LXC API calls when it differs from `/etc/hostname` |
| `PVE_API_TOKEN` | Agent | *(none)* | Proxmox API token; prefer setting on the agent host instead of over IPC |

### Public telemetry

In **Settings → Privacy & Access**, the **Guest system telemetry visibility** toggle stores `telemetry_public` in SQLite. When enabled, anonymous visitors and Guest-role users see host CPU model, usage, memory, GPU (when reported by the agent), and network stats on the dashboard. Container names, VMIDs, per-app metrics, streams, and admin controls stay hidden.

---

## Troubleshooting configuration issues

| Symptom | See |
|---------|-----|
| Streams show NOT CONFIGURED | [Media Integrations](./troubleshooting.md#media-integrations-not-showing-streams) |
| Apps stuck on CHECKING... | [Troubleshooting](./troubleshooting.md) — Proxmox token and agent IPC |
| Old UI after upgrade | [PWA / Browser Cache](./troubleshooting.md#pwa--browser-cache-issues) |
| Dashboard UI is broken | [Custom CSS Recovery](./troubleshooting.md#recovering-from-broken-custom-css) |
