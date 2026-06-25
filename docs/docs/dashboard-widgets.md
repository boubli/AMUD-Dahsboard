---
sidebar_position: 3
title: Dashboard Widgets
---

# Dashboard Widgets

Dashboard widgets are custom blocks that appear **above the main app grid** on the home dashboard. Use them for quick links, homelab notes, status strips, or small HTML layouts — without adding another app card.

They are separate from:

- **App cards** — services with URLs, telemetry, and integrations
- **RSS feeds** — managed under Settings → RSS Feeds and shown on `/feeds`

---

## How to add a widget

1. Log in as **Admin**.
2. Open **Settings** (gear icon).
3. Go to **Account → Widgets**.
4. Fill in the form:
   - **Title** — heading shown on the widget card
   - **Type** — `Note`, `Links`, or `HTML`
   - **Content** — text, link lines, or HTML (see below)
   - **Grid span** — `1×1`, `2×1`, or `1×2` (same bento sizes as app cards)
   - **Guest visible** — whether guests can see this widget
5. Click **Add widget**.

The widget appears on the dashboard immediately on the next page load.

To remove a widget, use **Delete** in the **Existing widgets** list on the same settings tab.

---

## Widget types

| Type | Content format | Rendered as |
|------|----------------|-------------|
| **Note** | Plain text (line breaks preserved in source; displayed as one paragraph) | Escaped text in a paragraph |
| **Links** | One link per line: `https://url\|Label` | Clickable link list |
| **HTML** | HTML snippet | Sanitized HTML block |

### Links format

Each line is `URL|Label`. The pipe character separates the address from the display text.

```text
https://radarr.local:7878|Radarr
https://sonarr.local:8989|Sonarr
```

This is **not** a JSON array — use one `url|label` pair per line.

### HTML sanitization

HTML widgets pass through a sanitizer that **removes** dangerous tags: `script`, `style`, `iframe`, `object`, `html`, and `body`. Stick to simple markup: `div`, `p`, `a`, `span`, `strong`, `ul`, `li`, `table`, etc. Inline `style` attributes on allowed tags are generally fine; entire `<style>` blocks are stripped.

---

## Grid span and visibility

| Grid span | Layout |
|-----------|--------|
| **1×1** | Standard single cell |
| **2×1** | Two columns wide |
| **1×2** | Two rows tall |

**Guest visible = Yes** shows the widget to guest-role users. Set **No** for admin-only notes (internal IPs, maintenance codes, etc.).

---

## 20 ready-to-use examples

Copy each block into **Settings → Account → Widgets**. Adjust URLs and text for your network.

### Notes (8)

#### 1. Welcome message

| Field | Value |
|-------|-------|
| Title | Welcome |
| Type | Note |
| Grid span | 2×1 |
| Guest visible | Yes |

```text
Welcome to the homelab dashboard. Apps below are live services on this network. Guest access is read-only — ask the admin if you need something added.
```

#### 2. Maintenance window

| Field | Value |
|-------|-------|
| Title | Maintenance |
| Type | Note |
| Grid span | 1×1 |
| Guest visible | No |

```text
Planned maintenance: Sunday 2:00–4:00 AM EST. NAS and media services may be offline. Downloads will resume automatically.
```

#### 3. Backup reminder

| Field | Value |
|-------|-------|
| Title | Backups |
| Type | Note |
| Grid span | 1×1 |
| Guest visible | No |

```text
Last verified backup: update this note after each test restore. Critical paths: /mnt/user/appdata, Proxmox vzdump, and off-site sync bucket.
```

#### 4. NAS mount paths

| Field | Value |
|-------|-------|
| Title | Storage paths |
| Type | Note |
| Grid span | 1×2 |
| Guest visible | No |

```text
Media: /mnt/user/media
Downloads: /mnt/user/downloads
Appdata: /mnt/user/appdata
ISOs: /mnt/user/isos
Share these paths only with trusted users.
```

#### 5. Guest Wi‑Fi

| Field | Value |
|-------|-------|
| Title | Guest Wi‑Fi |
| Type | Note |
| Grid span | 1×1 |
| Guest visible | Yes |

```text
Network: Homelab-Guest
Password: (ask host)
IoT and LAN services are not available on this SSID.
```

#### 6. Unraid parity check

| Field | Value |
|-------|-------|
| Title | Array health |
| Type | Note |
| Grid span | 1×1 |
| Guest visible | No |

```text
Run parity check monthly. If the array is degraded, do not run heavy writes until the rebuild finishes. Check Unraid notifications before large library imports.
```

#### 7. Emergency reboot steps

| Field | Value |
|-------|-------|
| Title | If things break |
| Type | Note |
| Grid span | 2×1 |
| Guest visible | No |

```text
1. Check Proxmox/Unraid UI first. 2. Restart the affected Docker stack, not the whole host. 3. Verify disk space and DNS. 4. Check AMUD server logs before rebooting hardware.
```

#### 8. Homelab changelog

| Field | Value |
|-------|-------|
| Title | Recent changes |
| Type | Note |
| Grid span | 2×1 |
| Guest visible | No |

```text
2026-06: Added Jellyfin, moved *arr stack to Docker. 2026-05: Migrated DNS to AdGuard. Update this widget when you change production services.
```

---

### Links (8)

#### 9. *arr stack quick links

| Field | Value |
|-------|-------|
| Title | Media stack |
| Type | Links |
| Grid span | 1×2 |
| Guest visible | No |

```text
https://radarr.example.com|Radarr
https://sonarr.example.com|Sonarr
https://prowlarr.example.com|Prowlarr
https://bazarr.example.com|Bazarr
https://overseerr.example.com|Overseerr
```

#### 10. Proxmox and Unraid admin

| Field | Value |
|-------|-------|
| Title | Hypervisor |
| Type | Links |
| Grid span | 1×1 |
| Guest visible | No |

```text
https://proxmox.local:8006|Proxmox
https://tower.local|Unraid
```

#### 11. Documentation hub

| Field | Value |
|-------|-------|
| Title | Docs |
| Type | Links |
| Grid span | 1×1 |
| Guest visible | Yes |

```text
https://boubli.github.io/AMUD-Dashboard/|AMUD Docs
https://wiki.example.com|Homelab Wiki
https://github.com/your-org/runbooks|Runbooks
```

#### 12. Smart home dashboards

| Field | Value |
|-------|-------|
| Title | Smart home |
| Type | Links |
| Grid span | 1×1 |
| Guest visible | No |

```text
https://homeassistant.local:8123|Home Assistant
https://nodered.local:1880|Node-RED
https://zigbee2mqtt.local|Zigbee2MQTT
```

#### 13. Monitoring tools

| Field | Value |
|-------|-------|
| Title | Monitoring |
| Type | Links |
| Grid span | 1×1 |
| Guest visible | No |

```text
https://uptime.example.com|Uptime Kuma
https://grafana.example.com|Grafana
https://beszel.example.com|Beszel
```

#### 14. Download clients

| Field | Value |
|-------|-------|
| Title | Downloads |
| Type | Links |
| Grid span | 1×1 |
| Guest visible | No |

```text
https://qbittorrent.example.com|qBittorrent
https://sabnzbd.example.com|SABnzbd
```

#### 15. Password manager and secrets

| Field | Value |
|-------|-------|
| Title | Security |
| Type | Links |
| Grid span | 1×1 |
| Guest visible | No |

```text
https://vault.example.com|Vaultwarden
https://auth.example.com|Authentik
```

#### 16. Public status pages

| Field | Value |
|-------|-------|
| Title | Status |
| Type | Links |
| Grid span | 2×1 |
| Guest visible | Yes |

```text
https://status.example.com|Service status
https://cloudflare.com|Cloudflare dashboard
```

---

### HTML (4)

#### 17. Simple status strip

| Field | Value |
|-------|-------|
| Title | System status |
| Type | HTML |
| Grid span | 2×1 |
| Guest visible | Yes |

```html
<div style="display:flex;gap:1rem;flex-wrap:wrap;font-size:0.85rem;">
  <span><strong style="color:#4ade80;">●</strong> Core online</span>
  <span><strong style="color:#4ade80;">●</strong> Media online</span>
  <span><strong style="color:#fbbf24;">●</strong> Backup pending</span>
</div>
```

#### 18. Two-column link grid

| Field | Value |
|-------|-------|
| Title | Quick access |
| Type | HTML |
| Grid span | 2×1 |
| Guest visible | No |

```html
<div style="display:grid;grid-template-columns:1fr 1fr;gap:0.5rem;font-size:0.85rem;">
  <a href="https://jellyfin.example.com" target="_blank" rel="noopener">Jellyfin</a>
  <a href="https://plex.example.com" target="_blank" rel="noopener">Plex</a>
  <a href="https://immich.example.com" target="_blank" rel="noopener">Immich</a>
  <a href="https://nextcloud.example.com" target="_blank" rel="noopener">Nextcloud</a>
</div>
```

#### 19. Alert banner

| Field | Value |
|-------|-------|
| Title | Heads up |
| Type | HTML |
| Grid span | 2×1 |
| Guest visible | Yes |

```html
<p style="margin:0;padding:0.75rem 1rem;border-radius:8px;background:rgba(251,191,36,0.15);border:1px solid rgba(251,191,36,0.4);font-size:0.85rem;">
  <strong>Maintenance tonight</strong> — expect brief downtime on media services after 2 AM.
</p>
```

#### 20. Server IP cheat sheet

| Field | Value |
|-------|-------|
| Title | LAN addresses |
| Type | HTML |
| Grid span | 1×2 |
| Guest visible | No |

```html
<table style="width:100%;font-size:0.8rem;border-collapse:collapse;">
  <tr><td style="padding:0.25rem 0;color:#94a3b8;">Proxmox</td><td>192.168.1.10</td></tr>
  <tr><td style="padding:0.25rem 0;color:#94a3b8;">NAS</td><td>192.168.1.20</td></tr>
  <tr><td style="padding:0.25rem 0;color:#94a3b8;">Docker host</td><td>192.168.1.30</td></tr>
  <tr><td style="padding:0.25rem 0;color:#94a3b8;">Pi-hole</td><td>192.168.1.5</td></tr>
</table>
```

---

## Create your own

### Notes

- Plain text only — **Markdown is not rendered**.
- Good for: reminders, Wi‑Fi info, runbook snippets, changelog stubs.
- Use **Guest visible = No** for anything with internal hostnames, passwords, or ops detail.

### Links

- Format: `https://full-url|Display name` per line.
- URLs must be valid `http://` or `https://` addresses.
- Links open in a new tab (`target="_blank"`).

### HTML

- Keep markup small; widgets share the same glass/bento styling as the rest of the dashboard.
- Avoid `<script>`, `<style>`, `<iframe>`, and embedded objects — they are stripped.
- Prefer inline styles on simple elements for layout (flex, grid, padding).
- Test after saving: if content disappears, the sanitizer likely removed a blocked tag.

---

## Limitations

- **No drag-reorder** for dashboard widgets yet — order follows creation time (`sort_order`).
- **`calendar_ics`** is reserved in the backend but has no settings UI or renderer today.
- Widgets always render **above** the app grid, not mixed between app cards.
- HTML widgets are sanitized for safety; complex layouts may need simplification.

---

## Related

- [Features](./features) — full dashboard capability list
- [Configuration](./configuration) — grid columns, appearance, integrations
