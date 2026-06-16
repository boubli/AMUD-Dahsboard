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

1. Open **Settings → Customization**.
2. Enter any valid CSS. 
3. Click Save. The CSS will be injected into the `<head>` of the dashboard for all users immediately.

*(Note: If you write invalid CSS that breaks the layout entirely, refer to [Troubleshooting](./troubleshooting.md) for recovery).*

---

## Proxmox and container control

Live **RUNNING** / **STOPPED** badges and start/stop controls require:

1. A valid **Proxmox API token** under **Settings → Proxmox VE**
2. Matching **container IDs** or Docker names on app cards
3. A working **amud-agent** on the hypervisor host

See [Proxmox VE Installation](./installation/proxmox.md#5-proxmox-api-token-configuration) for token setup.

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

In **Settings → Donation**, the **Public telemetry** toggle stores `telemetry_public` in SQLite. When enabled, anonymous visitors and Guest-role users see host metrics on the dashboard (container names and app health remain hidden).

---

## Troubleshooting configuration issues

| Symptom | See |
|---------|-----|
| Streams show NOT CONFIGURED | [Media Integrations](./troubleshooting.md#media-integrations-not-showing-streams) |
| Apps stuck on CHECKING... | [Troubleshooting](./troubleshooting.md) — Proxmox token and agent IPC |
| Old UI after upgrade | [PWA / Browser Cache](./troubleshooting.md#pwa--browser-cache-issues) |
| Dashboard UI is broken | [Custom CSS Recovery](./troubleshooting.md#recovering-from-broken-custom-css) |
