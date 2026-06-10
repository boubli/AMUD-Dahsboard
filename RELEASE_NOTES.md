# AMUD Dashboard v2.0.0 - Security & Architecture Overhaul

This is a major release representing the completion of the AMUD Dashboard audit backlog (TASK-001 to TASK-031). It includes substantial architectural improvements, security hardening, and UI enhancements. All 7 core server tests are passing.

## 🚨 Breaking Changes & Mandatory Updates
- **Mandatory Agent Secret**: The `AMUD_AGENT_SECRET` environment variable is now strictly mandatory. The server will exit if this is missing. Docker Compose setups must include this.
- **Docker API Option**: Docker API access can now be disabled entirely via `AMUD_DOCKER=0`.
- **Proxmox IPC Security**: The PVE API token no longer sends over IPC when using the Settings UI without an agent env token. `PVE_API_TOKEN` environment variable on the agent takes precedence.
- **Removed Fallbacks**: The legacy `/tmp/amud.sock` fallback has been completely removed. Auth bypass when the secret is empty has also been removed.

## 🏗️ Architecture & Code Organization
- **Monolith Split**: `main.rs` has been modularized into domain-specific files (handlers, auth, db, apps, media, agent, webhooks, settings, templates, models, security, audit).
- **Async & SQLite Handling**: Integrated `spawn_blocking` for SQLite operations (`with_db()`) and added a settings cache to prevent async handlers from blocking.
- **Agent Efficiency**: The agent now reuses a shared `OnceLock<Runtime>` rather than spawning new runtimes per PVE/Docker call.
- **Unified Templates & IPC**: Shared `BrandingVars` and unified UDS/TCP handler in `agent.rs`. Introduced `ui/static/admin.js` for shared CSRF/upload JS logic.

## 🔒 Security Enhancements
- **Auth Everywhere**: Admin authentication is now enforced on all `/api/users/*` routes and GET `/api/categories`.
- **CSRF Protection**: Added CSRF protection (X-CSRF-Token + meta tag) on POST forms and fetches. Logout is now primarily POST + CSRF.
- **Session Strictness**: WebSockets (`/ws`) and `/uploads/*` require a valid session. Sessions are immediately revoked upon password or username changes.
- **Bootstrap Admin**: Implemented a random bootstrap admin password (printed once on startup) with forced `admin_must_change_password`. The guest account has been removed.
- **Rate Limiting**: Added per-IP rate limits on login, settings, credentials, uploads, container actions, webhooks, and user management to prevent brute-force attacks.
- **XSS & Injection Hardening**: Implemented `serde_json` for app actions (removing `format!` injection risks), `escape_html` on usernames, and per-request CSP nonces on inline scripts (removing `'unsafe-inline'`).
- **SSRF Guard**: Health poller blocks localhost/metadata requests while allowing RFC1918 (Private IP) targets.
- **Data Exposure Prevention**: Secrets are masked in settings HTML (placeholders instead of full values), Jellyfin API keys are sent via `X-Emby-Token` headers instead of query strings, and webhook URLs are masked in the API.

## 📝 Audit & Traceability
- **Audit Logging**: Introduced an `audit_log` table and `GET /api/audit` route to track container actions, settings, credentials, uploads, webhooks, and users.

## 🎨 UI & Stream Cards
- **Jellyfin/Plex Gating**: Stream cards for Jellyfin/Plex now only show on the top row when a matching app exists in the database.
- **Traceability**: Implemented a container action acknowledgment protocol. The UI now shows real success/error statuses, and WS properly reports `agent_connected`.
- **SSR & Cleanup**: Removed ~280 lines of dead JS from `index.html` and replaced fake metrics with SSR placeholders (`CHECKING...` / `—`).
