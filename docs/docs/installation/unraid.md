---
sidebar_position: 6
title: Unraid
description: Install AMUD Dashboard on Unraid via Community Applications — dashboard + agent templates.
---

# Unraid Installation

AMUD ships as **two** Community Applications templates (dashboard + telemetry agent). This matches the Docker Compose architecture and keeps the agent isolated with read-only Docker socket access.

---

## Prerequisites

- Unraid 6.9+ with the **Community Applications** plugin installed
- A free TCP port (default **8000**) for the web UI
- ~100MB free RAM for both containers combined

---

## Step 1 — Install from Community Applications

After the templates are published on CA (or while testing from the maintainer repository):

1. Open the Unraid **Apps** tab.
2. Search for **AMUD Dashboard** and click **Install**.
3. Set a strong **Agent Secret** (long random string). **Copy it** — you need the same value on the agent.
4. Leave default paths unless you use a custom appdata layout:
   - App Data: `/mnt/user/appdata/amud-dashboard/data`
   - Agent Socket Dir: `/mnt/user/appdata/amud-dashboard/run`
5. Click **Apply** and wait for the container to start.

6. Search for **AMUD Agent** and click **Install**.
7. Paste the **same Agent Secret** as step 3.
8. Confirm **Agent Socket Dir** matches the dashboard (`/mnt/user/appdata/amud-dashboard/run`).
9. Set **Docker Monitoring** to `1` (recommended on Unraid).
10. Click **Apply**.

---

## Step 2 — First login

1. Open `http://YOUR_UNRAID_IP:8000` (or your reverse-proxy URL).
2. On first boot, the server prints a one-time **admin password** in the container log:
   - Unraid → **Docker** → **AMUD-Dashboard** → **Log**
3. Log in as `admin` with that password and change it under **Settings**.

---

## Step 3 — Verify telemetry

Within a few seconds of both containers running:

- Host CPU/RAM/disk widgets should populate.
- With `AMUD_DOCKER=1` on the agent, Docker container cards can show live status.

If telemetry is missing:

- Confirm both containers are **running**.
- Confirm **Agent Secret** matches on both (re-create if unsure).
- Confirm **Agent Socket Dir** is identical on both.
- Check **AMUD-Agent** logs for connection errors.

---

## Reverse proxy

AMUD uses WebSockets for live telemetry. If you put it behind Nginx Proxy Manager or Traefik, enable WebSocket upgrade on the backend. See [Reverse Proxy](./reverse-proxy.md).

---

## Updating

1. Stop **AMUD-Agent**, then **AMUD-Dashboard** (optional but avoids a brief socket race).
2. In **Apps**, click the container icon → **Force Update** (or set tag to `latest` and recreate).
3. Update **both** containers to the same image tag.
4. Start dashboard, then agent.

Your config lives in `/mnt/user/appdata/amud-dashboard/data` — back up that folder before major upgrades.

---

## Support

- [GitHub Discussions](https://github.com/boubli/AMUD-Dashboard/discussions) — questions, screenshots, stack reports
- [GitHub Issues](https://github.com/boubli/AMUD-Dashboard/issues) — bugs with logs and steps to reproduce
- [Unraid forum thread](https://forums.unraid.net/) — search for "AMUD Dashboard" after the support topic is posted
