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
GET /Sessions?api_key=<your-key>
```

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

## Proxmox and container control

Live **RUNNING** / **STOPPED** badges and start/stop controls require:

1. A valid **Proxmox API token** under **Settings → Proxmox VE**
2. Matching **container IDs** or Docker names on app cards
3. A working **amud-agent** on the hypervisor host

See [Proxmox VE Installation](./installation/proxmox.md#4-proxmox-api-token-configuration) for token setup.

---

## Troubleshooting configuration issues

| Symptom | See |
|---------|-----|
| Streams show NOT CONFIGURED | [Media Integrations](./troubleshooting.md#media-integrations-not-showing-streams) |
| Apps stuck on CHECKING... | [Troubleshooting](./troubleshooting.md) — Proxmox token and agent IPC |
| Old UI after upgrade | [PWA / Browser Cache](./troubleshooting.md#pwa--browser-cache-issues) |
| Grid setting not visible | Hard-refresh the browser; clear PWA cache after upgrades |
