<div align="center">
  <img src="https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/assist/amud-logo-github.png" alt="AMUD Logo" width="300" />
</div>

# AMUD Dashboard

![AMUD Dashboard UI](https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/assist/AMUD-Dashboard.png)
AMUD (Advanced Modern Unified Dashboard) is a high-performance, intelligent home lab cockpit engineered strictly for resource-constrained environments. While legacy dashboards demand heavy runtimes, bloated frameworks, and complex text-file configurations, AMUD provides a single-binary, zero-dependency ecosystem control center that idles at roughly **~26MB of RAM** (combined server and agent) with a **~660MB disk footprint** when deployed as a full Debian LXC container.

**📚 Official Documentation:** [https://boubli.github.io/AMUD-Dashboard/](https://boubli.github.io/AMUD-Dashboard/)

**Ready to deploy? Refer directly to our [AMUD Deployment Guide](DEPLOY.md) for automated installer scripts, Portainer configs, and Docker CLI instructions.**

---

## Proxmox Telemetry Configuration

AMUD now communicates **directly with the Proxmox VE REST API** for container telemetry. The agent no longer shells out to the `pvesh` Python CLI on every poll — it issues native, lightweight HTTPS requests over `hyper`, dramatically reducing CPU and memory overhead on your Proxmox host.

To enable LXC telemetry, provide the agent with a Proxmox API token.

### 1. Create an API Token

In the Proxmox web UI:

1. Navigate to **Datacenter → Permissions → API Tokens**.
2. Click **Add**.
3. Select the **User** the token belongs to (e.g. `root@pam`).
4. Enter a **Token ID** (e.g. `amud`).
5. *(Optional)* Leave **Privilege Separation** unchecked to inherit the user's permissions, or assign the token explicit `VM.Audit` / `Sys.Audit` rights on the relevant nodes.
6. Click **Add**, then **copy the Secret value immediately** — Proxmox displays it only once.

### 2. Set the Environment Variable

Pass the credential to the agent via `PVE_API_TOKEN`. It must contain the **entire** value, including the `PVEAPIToken=` scheme prefix:

```bash
PVE_API_TOKEN=PVEAPIToken=USER@REALM!TOKENID=SECRET
```

For example:

```bash
PVE_API_TOKEN=PVEAPIToken=root@pam!amud=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

> If this variable is unset, the agent simply skips Proxmox polling — host CPU/RAM/disk metrics continue to work normally.

### 3. Add it to `docker-compose.yml`

```yaml
services:
  amud-agent:
    image: boubli/amud-agent:latest
    environment:
      - PVE_API_TOKEN=PVEAPIToken=root@pam!amud=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    volumes:
      - /opt/amud/run:/opt/amud/run
    restart: unless-stopped
```

---

## Why AMUD Demolishes Legacy Dashboards (Heimdall, Homepage, Homarr)

### 1. Bare-Metal Resource Discipline
* **The Legacy Problem:** Heimdall relies on a heavy PHP/Laravel lifecycle, requiring background web servers (Nginx/Apache) and PHP-FPM daemons that swallow 150MB+ RAM just sitting idle. 
* **The AMUD Solution:** Written in pure, compiled Rust. It executes native machine code with zero interpreter overhead, running the entire dashboard, telemetry layer, and database inside a strict **~26MB RAM** envelope at idle in a full LXC container.

### 2. Zero-YAML, 100% UI-Driven Control
* **The Legacy Problem:** Next-gen dashboards force you to spend hours manually writing, indenting, and debugging hundreds of lines of complex YAML text files just to add a shortcut.
* **The AMUD Solution:** Powered by an embedded, ultra-fast **SQLite (Rusqlite)** architecture. You get the advanced layout categories, tagging, and sub-pages of a modern dashboard, but configured entirely through an elegant, reactive user interface. 

### 3. Active Cockpit vs. Passive Bookmarks
* **The Legacy Problem:** Traditional dashboards are just glorified lists of web links. If a service freezes or crashes, they are completely blind to it.
* **The AMUD Solution:** 
  * **Asynchronous Tokio Telemetry:** Background tokio threads concurrently poll your metrics and stream live updates to the UI via WebSockets without blocking your browser or causing layout lags.
  * **Integrated Live Clock, Search & Category Filters:** View a live-updating local clock and customized greetings based on the hour of the day, search the web with a configurable search widget, and filter applications dynamically client-side with category filter tabs.
  * **Dynamic Media Streams:** The dashboard automatically hides Plex/Jellyfin stream cards if those applications aren't registered in your homelab database, showing them only when configured.

### 4. Admin vs. Guest Profiles
* **The Legacy Problem:** Sharing your landing page with family members usually means exposing your sensitive admin tools (Proxmox, Portainer) or setting up massive external proxy layers.
* **The AMUD Solution:** Built-in cryptographic user roles. Admins see the full cluster control array (with add/delete buttons and settings drawer); guests or family profiles get a clean, read-only dashboard layout out of the box.

---

## Microscopic Production Footprint

| Dimension | Heimdall Application Dashboard | AMUD Dashboard |
| :--- | :---: | :---: |
| **Engine** | PHP 8+ / Laravel | Rust / Axum / Tokio |
| **Runtime Overhead** | High (Interpreted PHP-FPM) | Zero (Native Compiled Machine Code) |
| **Assets Injection** | Read from host disk paths | Embedded templates (`include_str!`) + static files |
| **Idle RAM Footprint** | 80MB - 150MB | **~26MB (Combined server/agent)** |
| **Boot Time** | 2 - 5 seconds | **Sub-millisecond (Instant)** |
