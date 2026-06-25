---
sidebar_position: 3
title: Dashboard Widgets
---

# Dashboard Widgets

Custom blocks that sit **above the app grid** — quick links, a short note, or a small HTML layout. Not the same as app cards or RSS feeds.

**Settings → Account → Widgets** to add one. Pick a title, type, content, grid span (`1×1`, `2×1`, `1×2`), and whether guests can see it.

---

## Types at a glance

| Type | What you paste in **Content** |
|------|-------------------------------|
| **Note** | Plain text |
| **Links** | One line per link: `https://url\|Label` |
| **HTML** | A small HTML snippet (sanitized) |

---

## Example: Note

**Title:** `Welcome`  
**Type:** Note  
**Content:**

```text
Homelab dashboard — apps below are live services. Guest access is read-only.
```

That's it. No formatting, no markdown.

---

## Example: Links

**Title:** `Media stack`  
**Type:** Links  
**Content:**

```text
https://radarr.local:7878|Radarr
https://sonarr.local:8989|Sonarr
https://prowlarr.local:9696|Prowlarr
```

One `url|label` per line. The `|` separates the address from what users click. Not JSON.

Swap in your real URLs and names. Links open in a new tab.

---

## Example: HTML

**Title:** `Status`  
**Type:** HTML  
**Grid span:** 2×1 (optional — wider card)  
**Content:**

```html
<div style="display:flex;gap:1rem;flex-wrap:wrap;font-size:0.85rem;">
  <span><strong style="color:#4ade80;">●</strong> Core online</span>
  <span><strong style="color:#fbbf24;">●</strong> Backup pending</span>
</div>
```

Use simple tags: `div`, `p`, `a`, `span`, `strong`, `ul`, `li`, `table`. Inline `style` on those is fine.

**Stripped for safety:** `script`, `style`, `iframe`, `object`, `html`, `body`. If your widget comes out empty, you probably used a blocked tag.

Another common pattern — a small alert:

```html
<p style="margin:0;padding:0.75rem;border-radius:8px;background:rgba(251,191,36,0.15);font-size:0.85rem;">
  <strong>Maintenance tonight</strong> — media may be down after 2 AM.
</p>
```

---

## Tips

- **Guest visible = No** for internal IPs, passwords, or ops notes.
- Widget order follows when you created them — no drag-reorder yet.
- Widgets always show above the app grid, not mixed in with cards.

---

## Related

- [Features](./features)
- [Configuration](./configuration)
