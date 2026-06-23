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

See [Permission errors on appdata](#permission-errors-on-appdata) below if the dashboard container exits or the database cannot be created.

---

## Permission errors on appdata

**Symptom:** The **AMUD-Dashboard** container fails to start, restarts in a loop, or logs show **permission denied** when writing to `/data` or the agent cannot use the shared socket under `run/`.

**Cause:** Unraid maps host paths into the container. If `data/` or `run/` on the host are owned by a user the container cannot write as, SQLite and the agent Unix socket will fail.

Default CA paths:

| Path | Purpose |
|------|---------|
| `/mnt/user/appdata/amud-dashboard/data` | SQLite database and settings |
| `/mnt/user/appdata/amud-dashboard/run` | Shared socket between dashboard and agent |

**Fix 1 — Match container user (most common)**

Our images run as **root** inside the container (`UID 0`). Ensure the host appdata folders are writable:

```bash
chown -R 0:0 /mnt/user/appdata/amud-dashboard/data
chown -R 0:0 /mnt/user/appdata/amud-dashboard/run
chmod -R 755 /mnt/user/appdata/amud-dashboard/data
chmod -R 770 /mnt/user/appdata/amud-dashboard/run
```

Then restart **AMUD-Dashboard**, then **AMUD-Agent**.

---

## Reset admin password (Docker)

If you cannot log in, reset the password from the Unraid terminal (legacy SHA-256 hash; AMUD upgrades it to Argon2id on next login):

```bash
# 1. Generate SHA256 hash of your new password
echo -n 'YOUR_NEW_PASSWORD' | sha256sum | awk '{print $1}'

# 2. Update the database (replace HASH with output above)
docker run --rm -v /mnt/user/appdata/amud-dashboard/data:/data alpine sh -c \
  "apk add --no-cache sqlite >/dev/null 2>&1 && sqlite3 /data/amud.db \"UPDATE users SET password_hash='HASH' WHERE username='admin';\""
```

Restart the **AMUD-Dashboard** container, sign in, then change the password under **Settings → Security** so a fresh Argon2id hash is stored.

See also [Troubleshooting — Reset admin password](../troubleshooting.md#reset-or-change-the-admin-password-from-cli).

---

**Fix 2 — Custom PUID/PGID templates**

If your template sets a non-root user (e.g. `PUID=99`, `PGID=100`), ownership must match that user instead of `0:0`:

```bash
chown -R 99:100 /mnt/user/appdata/amud-dashboard/data
chown -R 99:100 /mnt/user/appdata/amud-dashboard/run
```

**Verify**

```bash
ls -la /mnt/user/appdata/amud-dashboard/
```

Both `data` and `run` should be owned by the same UID/GID the container uses. After fixing, open the UI — host telemetry should populate within a few seconds.

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

Please use the right channel so reports are tracked and fixed in releases:

| Channel | Use for |
|---------|---------|
| [**GitHub Issues**](https://github.com/boubli/AMUD-Dashboard/issues) | **Bugs**, install failures, feature requests — include Unraid version, container logs, and steps to reproduce |
| [GitHub Discussions](https://github.com/boubli/AMUD-Dashboard/discussions) | Questions, screenshots, general homelab chat |
| [Documentation](https://boubli.github.io/AMUD-Dashboard/docs/troubleshooting) | Self-service fixes (permissions, WebSockets, agent socket) |

Unraid forum threads are welcome for visibility, but **open a GitHub Issue** for anything that needs a code or docs fix so it is not lost in the thread.
