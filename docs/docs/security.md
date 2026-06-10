---
sidebar_position: 3
---

# Security

AMUD ships with several built-in protections for authentication, sessions, and HTTP hardening. This page describes what is enabled by default and how to configure it for production deployments behind HTTPS.

For password recovery and IPC troubleshooting, see [Troubleshooting](./troubleshooting.md).

---

## Password storage (Argon2id)

New passwords and password changes are stored as **Argon2id** hashes in the SQLite `users` table.

If you upgraded from an older release that used SHA-256 password hashes, AMUD still accepts the legacy hash **once** on login and then **transparently re-hashes** the password to Argon2id. No manual migration is required.

After an emergency CLI password reset using a SHA-256 hash, sign in once and change the password under **Settings → Security** so a fresh Argon2id hash is stored immediately.

---

## Sessions and cookies

| Setting | Behavior |
|---------|----------|
| Session token | 32-byte cryptographically random value (URL-safe base64) |
| Session lifetime | 24 hours (`Max-Age=86400`) |
| Server-side expiry | Sessions are removed when expired; a background task runs hourly |
| `HttpOnly` | JavaScript cannot read the session cookie |
| `SameSite=Strict` | Cookie is not sent on cross-site requests |
| `Secure` | Optional — see below |

### HTTPS / reverse proxy: `AMUD_SECURE_COOKIES`

When AMUD is served over HTTPS (directly or through a reverse proxy), set the **`Secure`** cookie flag so the session cookie is never sent over plain HTTP.

Add to the `amud-server` / `amud` systemd environment (or Docker `environment` block):

```bash
AMUD_SECURE_COOKIES=1
```

Then restart the dashboard service.

:::tip
If you terminate TLS at Nginx, Caddy, or Traefik, see [Reverse Proxy](./installation/reverse-proxy.md) for TLS configuration. Set `AMUD_SECURE_COOKIES=1` whenever the browser reaches AMUD over `https://`.
:::

---

## Login rate limiting

Failed login attempts are tracked **per username** (case-insensitive):

- **Maximum:** 5 failed attempts
- **Window:** 5 minutes
- **Scope:** In-memory (resets on service restart)

When the limit is reached, login is temporarily blocked for that username until the window expires. Use the CLI password reset procedure in [Troubleshooting](./troubleshooting.md#reset-or-change-the-admin-password-from-cli) if you are locked out after repeated mistakes.

---

## Agent IPC authentication (`AMUD_AGENT_SECRET`)

The host **amud-agent** and the LXC/container **amud-server** share a secret (`AMUD_AGENT_SECRET`) used to authenticate telemetry over the Unix socket. This is **not** your Proxmox API token.

The secret must match on both sides. Mismatch causes `invalid IPC authentication` in agent logs and **0% host metrics** on the dashboard.

See [Troubleshooting → AMUD Agent Secret](./troubleshooting.md#amud-agent-secret--ipc-authentication) for verification commands.

---

## HTTP security headers

AMUD adds the following headers on every HTTP response:

| Header | Value |
|--------|-------|
| `X-Frame-Options` | `DENY` |
| `X-Content-Type-Options` | `nosniff` |
| `Referrer-Policy` | `same-origin` |
| `Content-Security-Policy` | Restricts scripts, styles, connections, and framing to trusted sources |

When placing AMUD behind a reverse proxy, avoid stripping these headers unless you re-apply equivalent policy at the proxy layer.

---

## Docker Compose hardening

The bundled `docker-compose.yml` applies:

- `cap_drop: [ALL]` — drop Linux capabilities in the container
- `security_opt: no-new-privileges:true` — block privilege escalation

Treat the Docker socket mount as a **trust boundary**: read-only (`:ro`) protects the socket file on disk, but the Docker API can still accept lifecycle requests over the socket. Use a Docker socket proxy if you need method-level filtering.

---

## Production checklist

1. Change the default `admin` password under **Settings → Security**.
2. Use a restricted Proxmox API token (read-only audit), not root credentials.
3. Terminate TLS at a reverse proxy; set `AMUD_SECURE_COOKIES=1`.
4. Do not expose port `8000` to the public internet without authentication and TLS.
5. Rotate `AMUD_AGENT_SECRET` and Proxmox tokens if they were ever exposed in logs or chat.
