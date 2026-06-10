# AMUD Dashboard — Full Audit Report & Task Backlog

**Generated:** 2026-06-10  
**Scope:** Security, optimization, UI→DB traceability (4 layers)  
**Codebase:** `amud-server` (~3.6k LOC monolith), `amud-agent` (~940 LOC), `ui/templates/*`

---

## Executive Summary

Six parallel audits reviewed the codebase as if a junior developer wrote it and senior engineers are now hardening it.

| Area | Findings | Top risk |
|------|----------|----------|
| **Security** | 3 Critical, 12 High, 11 Medium, 8 Low | Unauthenticated `/api/users` = remote admin takeover |
| **Optimization** | 12 High, 18 Medium, 10 Low ROI items | Monolithic `main.rs`, blocking SQLite in async |
| **Traceability** | 15 UI, 10 API, 18 logic, 12 DB gaps | Proxmox test broken (`token` vs `pve_api_token`) |

### Fix these first (cross-cutting, confirmed in code)

| Priority | ID(s) | Issue | Effort |
|----------|-------|-------|--------|
| P0 | SEC-001, TRACE-* | `/api/users/*` has **zero auth** | Small |
| P0 | TRACE-UI-001, TRACE-DB-007 | Settings Proxmox test sends wrong field name | Trivial |
| P1 | SEC-004, SEC-005, SEC-006 | CSRF, WS auth, upload auth | Medium |
| P1 | TRACE-UI-002, TRACE-DB-002 | Category `sort_order` UI sends but DB ignores | Small |
| P1 | SEC-003, SEC-011 | Empty IPC secret = auth disabled | Small |
| P2 | OPT-003–004, TRACE-UI-003 | ~300 lines dead JS in `index.html` | Small |
| P2 | OPT-008, OPT-019 | Split `main.rs`; fix blocking DB | Large |

---

## Part 1 — Security Findings

### CRITICAL

| ID | Location | Issue | Fix |
|----|----------|-------|-----|
| SEC-001 | `main.rs` user handlers | `/api/users` GET/POST/edit/delete — no session check | Add `require_admin()` to all four handlers |
| SEC-002 | `main.rs:284-297` | Default `admin`/`admin` + `guest`/`guest` seeded | Force change on first login; random install password |
| SEC-003 | `docker-compose.yml` + `agent_authenticated()` | No `AMUD_AGENT_SECRET` in compose; empty secret = auth bypass | Fail closed; inject secret in compose |

### HIGH

| ID | Issue |
|----|-------|
| SEC-004 | No CSRF on any POST form or fetch |
| SEC-005 | `/ws` unauthenticated — full telemetry exposed |
| SEC-006 | `/uploads` public read (ServeDir) |
| SEC-007 | SSRF via app URL health poller |
| SEC-008 | Docker socket = root-equivalent |
| SEC-009 | PVE/Jellyfin/Plex secrets in HTML `value=""` |
| SEC-010 | `app_action_handler` JSON via `format!` — injection risk |
| SEC-011 | IPC auth disabled when secret empty |
| SEC-012 | CSS/template injection via settings (`custom_bg_url`, etc.) |
| SEC-013 | Username reflected unescaped (XSS) |
| SEC-014 | Container stop/start with no audit log |
| SEC-015 | PVE token pushed plaintext over IPC to agent |

### MEDIUM (selected)

SEC-016 weak agent secret fallback · SEC-017 SVG uploads · SEC-018 CSP unsafe-inline/eval · SEC-019 Secure cookie opt-in · SEC-020 binds 0.0.0.0 · SEC-021 public GET categories · SEC-022 webhook full URL leaked · SEC-023 innerHTML XSS in settings · SEC-024 sessions not revoked on password change · SEC-025 rate limit gaps · SEC-026 settings accepts arbitrary keys

### LOW (selected)

SEC-027 Jellyfin key in query string · SEC-028 GET logout · SEC-029 `/tmp/amud.sock` fallback · SEC-030 docs chmod 666/777 · SEC-031 curl\|bash · SEC-032 legacy SHA-256 · SEC-033 default guest · SEC-034 no HSTS

### Implemented well ✓

Argon2id + legacy migration · constant-time compare · secure session tokens · HttpOnly/SameSite cookies · login rate limit · parameterized SQL · `escape_html` on app cards · security headers · upload extension whitelist · SHA256SUMS in updater · admin gate on most destructive routes

---

## Part 2 — Optimization Findings

### High priority

| ID | Category | Issue |
|----|----------|-------|
| OPT-008 | structure | `main.rs` monolith — 66 functions, ~3,620 lines |
| OPT-019 | performance | `Mutex<Connection>` held inside async handlers |
| OPT-028 | performance | Agent creates new tokio runtime per PVE/Docker call |
| OPT-042–043 | other | Same as SEC-001 — unauthenticated user API |
| OPT-040–041 | other | Hardcoded fake latencies/metrics in SSR until WS |

### Medium priority (selected)

OPT-009 duplicate UDS/TCP listeners · OPT-010–011 duplicate webhook/settings queries · OPT-012–013 duplicate theme/CSS builders · OPT-016 duplicate admin JS in index+settings · OPT-017–018 duplicate admin guard · OPT-020 session write lock on read · OPT-021–027 dashboard/WS/webhook perf · OPT-029 agent Docker N+1 · OPT-030 agent refresh_all every 5s

### Quick wins

| ID | Action |
|----|--------|
| OPT-003–004 | Delete dead Alpine state + ~300 lines orphaned JS from `index.html` |
| OPT-005 | Remove no-op template `.replace()` calls |
| OPT-040 | SSR badges → `CHECKING...` instead of fake ms values |
| OPT-048 | Rename media key `emby` → `jellyfin` end-to-end |

---

## Part 3 — Traceability (UI → API → Logic → DB)

### Layer flow (healthy paths)

```
Browser form/fetch/ws
  → Axum route (main.rs:334-376)
    → get_session() / handler logic
      → SQLite (apps, users, settings, categories, webhooks)
      → OR agent_command_tx → amud-agent → PVE API / Docker socket
      → OR background pollers → in-memory state → WebSocket push
```

### Confirmed broken paths

| ID | Layer | UI action | Break point |
|----|-------|-----------|-------------|
| TRACE-UI-001 | UI→API | Settings → Test Proxmox | JS sends `token`, handler reads `pve_api_token` |
| TRACE-UI-002 | UI→DB | Category sort order | UI sends `sort_order`, handlers ignore column |
| TRACE-UI-010 | API | Users CRUD | No auth on backend despite admin-only UI |
| TRACE-LOGIC-003 | Logic | Container start/stop | Returns success when queued, not executed |
| TRACE-LOGIC-012 | Logic→UI | Stream card badges | `updateMediaStream()` skips RUNNING/ERROR badge updates |

### Full route inventory (27 handlers + static)

See TRACE-API layer: all routes mapped. Every active UI action has a matching route **except** external weather APIs. Dead callers exist in `index.html` legacy JS only.

### Settings key matrix (17 UI fields)

All 17 `#mainSettingsForm` fields map correctly to `settings` table **except** credentials (separate `/admin/credentials` → `users` table). Legacy `app_grid_columns` read but never written.

---

## Task Backlog (for sub-agents)

Each task is self-contained. Pick by `priority` field. Reference IDs link to audit sections above.

---

### TASK-001 — Hotfix: Auth on `/api/users`
- **priority:** P0
- **status:** done
- **refs:** SEC-001, TRACE-UI-010, TRACE-API-001, OPT-042
- **files:** `amud-server/src/main.rs`
- **work:** Add `get_session` + `role == "Admin"` to `list_users_handler`, `add_user_handler`, `edit_user_handler`, `delete_user_handler`. Return 403 JSON on failure.
- **acceptance:** Unauthenticated `curl /api/users` returns 403; settings Users tab still works when logged in as admin.

---

### TASK-002 — Hotfix: Proxmox test field name
- **priority:** P0
- **status:** done
- **refs:** TRACE-UI-001, TRACE-API-004, TRACE-DB-007
- **files:** `ui/templates/settings.html` (~line 1121)
- **work:** Change `formData.append('token', ...)` → `formData.append('pve_api_token', ...)`
- **acceptance:** Test Connection with valid token returns success when PVE reachable.

---

### TASK-003 — Persist category sort_order
- **priority:** P1
- **status:** done
- **refs:** TRACE-UI-002, TRACE-DB-002, TRACE-API-005
- **files:** `amud-server/src/main.rs` (`add_category_handler`, `edit_category_handler`)
- **work:** Accept `sort_order` in form; INSERT/UPDATE include column.
- **acceptance:** Sort order saved in DB; list returns correct order.

---

### TASK-004 — CSRF protection
- **priority:** P1
- **status:** done
- **refs:** SEC-004, TRACE-API-007
- **files:** `main.rs`, `index.html`, `settings.html`, `login.html`
- **work:** Issue CSRF token per session; hidden field on forms; validate header/field on POST.
- **acceptance:** POST without token rejected; all forms include token.

---

### TASK-005 — Authenticate WebSocket `/ws`
- **priority:** P1
- **status:** done
- **refs:** SEC-005, TRACE-UI-012, TRACE-LOGIC-002
- **files:** `main.rs`, optionally `index.html`
- **work:** Validate `amud_session` on upgrade; optional `telemetry_public` setting for guests.
- **acceptance:** Unauthenticated WS rejected (or reduced payload per setting).

---

### TASK-006 — Protect `/uploads`
- **priority:** P1
- **status:** done
- **refs:** SEC-006, SEC-017
- **files:** `main.rs`
- **work:** Replace public ServeDir with authenticated handler; disallow SVG or sanitize.
- **acceptance:** `/uploads/foo.png` requires session or signed URL.

---

### TASK-007 — Fix app_action JSON encoding
- **priority:** P1
- **status:** done
- **refs:** SEC-010, TRACE-LOGIC-004, OPT-037
- **files:** `main.rs` (`app_action_handler`)
- **work:** Use `serde_json::json!`; allowlist `provider` and `action`.
- **acceptance:** IDs with quotes cannot break agent command stream.

---

### TASK-008 — Mandatory IPC secret
- **priority:** P1
- **status:** done
- **refs:** SEC-003, SEC-011, SEC-016
- **files:** `main.rs`, `amud-agent/src/main.rs`, `docker-compose.yml`, `setup-amud.sh`
- **work:** Refuse startup if secret empty; compose injects matching secrets; use OsRng everywhere.
- **acceptance:** Empty secret → process exits with clear error.

---

### TASK-009 — Stop echoing secrets in HTML
- **priority:** P1
- **status:** done
- **refs:** SEC-009
- **files:** `settings.html`, `settings_page_handler`
- **work:** Mask tokens in UI; placeholder "••••••"; separate set-only fields.
- **acceptance:** View-source shows no full API tokens.

---

### TASK-010 — Delete dead JS from index.html
- **priority:** P2
- **status:** done
- **refs:** OPT-003, OPT-004, TRACE-UI-003, TRACE-UI-004
- **files:** `ui/templates/index.html`
- **work:** Remove orphaned functions and unused Alpine state; extract shared admin.js if needed.
- **acceptance:** No references to missing DOM IDs; file ~300 lines shorter.

---

### TASK-011 — Fix SSR placeholder metrics
- **priority:** P2
- **status:** done
- **refs:** OPT-040, OPT-041, OPT-044
- **files:** `main.rs` (`dashboard_handler`), `index.html`
- **work:** Render `CHECKING...` / `—` instead of hardcoded ms and VM counts.
- **acceptance:** First paint honest; WS updates all badges.

---

### TASK-012 — Container action feedback loop
- **priority:** P2
- **status:** done
- **refs:** TRACE-LOGIC-003, TRACE-LOGIC-005
- **files:** `main.rs`, `amud-agent`, `index.html`
- **work:** Agent ack protocol; include `agent_connected` in WS payload; UI shows real result.
- **acceptance:** Failed stop shows error; offline agent shows disconnected state.

---

### TASK-013 — Split main.rs into modules
- **priority:** P3
- **status:** done
- **refs:** OPT-008, OPT-017
- **files:** new `auth.rs`, `handlers/`, `db.rs`, etc.
- **work:** Extract auth, routes, media, agent, templates without behavior change.
- **acceptance:** `cargo test` passes; `main.rs` < 500 lines.

---

### TASK-014 — Non-blocking database access
- **priority:** P3
- **status:** done
- **refs:** OPT-019, OPT-020, OPT-021
- **files:** `main.rs`
- **work:** `spawn_blocking` for rusqlite or connection pool; settings cache `Arc<RwLock<HashMap>>`.
- **acceptance:** Concurrent requests don't block tokio workers on DB lock.

---

### TASK-015 — Agent runtime reuse
- **priority:** P3
- **status:** done
- **refs:** OPT-028, OPT-015, OPT-029
- **files:** `amud-agent/src/main.rs`
- **work:** Single `OnceLock<Runtime>`; shared TLS client; parallel Docker stats with cap.
- **acceptance:** No new runtime per PVE/Docker call.

---

### TASK-016 — Rename emby → jellyfin in media pipeline
- **priority:** P3
- **status:** done
- **refs:** OPT-048, TRACE-LOGIC-011
- **files:** `main.rs`, `index.html`
- **work:** Key `jellyfin` in `media_streams` and WS; alias `emby` for compat.
- **acceptance:** Jellyfin stream card updates consistently.

---

### TASK-017 — Settings handler allowlist
- **priority:** P2
- **status:** done
- **refs:** SEC-026, TRACE-DB-010
- **files:** `main.rs` (`settings_handler`)
- **work:** Only upsert known keys from `get_default_settings()` + overlay/weather keys.
- **acceptance:** Arbitrary POST keys ignored.

---

### TASK-018 — XSS hardening pass
- **priority:** P2
- **status:** done
- **refs:** SEC-012, SEC-013, SEC-023, SEC-018
- **files:** `main.rs`, `settings.html`, `index.html`
- **work:** Escape username everywhere; `textContent` not `innerHTML`; tighten CSP where possible.
- **acceptance:** XSS payloads in username/category name don't execute.

---

### TASK-019 — Session invalidation on password change
- **priority:** P2
- **status:** done
- **refs:** SEC-024
- **files:** `main.rs` (`credentials_handler`)
- **work:** Revoke all sessions for user on password/username change.
- **acceptance:** Old cookies invalid after credential update.

---

### TASK-020 — Default credentials / first-login flow
- **priority:** P1
- **refs:** SEC-002, SEC-033
- **files:** `main.rs`, `setup-amud.sh`, docs
- **work:** Random admin password at install OR forced change flag in DB.
- **acceptance:** Fresh install cannot login with known `admin`/`admin` without reset step.

---

## Sub-agent assignment guide

| Agent type | Suggested tasks |
|------------|-----------------|
| Security-focused | TASK-001, 004, 005, 006, 007, 008, 009, 017, 018, 019, 020 |
| Bugfix / traceability | TASK-002, 003, 011, 012, 016 |
| Cleanup | TASK-010 |
| Architecture / perf | TASK-013, 014, 015 |

---

## Appendix — Route auth matrix

| Route | Auth today | Should be |
|-------|------------|-----------|
| `GET/POST /api/users/*` | **None** | Admin |
| `GET /ws` | None | Session or public-reduced |
| `GET /uploads/*` | None | Session or signed URL |
| `GET /api/categories` | None | Session (optional) |
| `POST /admin/settings` | Admin (silent fail) | Admin + CSRF + error feedback |
| All other admin POST | Admin | Admin + CSRF |

---

*This file is the canonical task backlog from the 2026-06-10 audit. Sub-agents should reference task IDs in commit messages (e.g. `fix: TASK-002 proxmox test field name`).*
