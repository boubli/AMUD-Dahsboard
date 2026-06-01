# AMUD Dashboard

AMUD (Advanced Modern Unified Dashboard) is a high-performance, intelligent home lab cockpit engineered strictly for resource-constrained environments. While legacy dashboards demand heavy runtimes, bloated frameworks, and complex text-file configurations, AMUD provides a single-binary, zero-dependency ecosystem control center that idles under **25MB of RAM**.

**Ready to deploy? Refer directly to our [AMUD Deployment Guide](DEPLOY.md) for automated installer scripts, Portainer configs, and Docker CLI instructions.**

---

## Why AMUD Demolishes Legacy Dashboards (Heimdall, Homepage, Homarr)

### 1. Bare-Metal Resource Discipline
* **The Legacy Problem:** Heimdall relies on a heavy PHP/Laravel lifecycle, requiring background web servers (Nginx/Apache) and PHP-FPM daemons that swallow 150MB+ RAM just sitting idle. 
* **The AMUD Solution:** Written in pure, compiled Go 1.22+. It executes native machine code with zero interpreter overhead, running the entire dashboard, telemetry layer, and database inside a strict **< 25MB RAM** envelope.

### 2. Zero-YAML, 100% UI-Driven Control
* **The Legacy Problem:** Next-gen dashboards force you to spend hours manually writing, indenting, and debugging hundreds of lines of complex YAML text files just to add a shortcut.
* **The AMUD Solution:** Powered by an embedded, ultra-fast **SQLite (Pure-Go/CGO-Free)** architecture. You get the advanced layout categories, tagging, and sub-pages of a modern dashboard, but configured entirely through an elegant, reactive user interface. 

### 3. Active Cockpit vs. Passive Bookmarks
* **The Legacy Problem:** Traditional dashboards are just glorified lists of web links. If a service freezes or crashes, they are completely blind to it.
* **The AMUD Solution:** 
  * **Asynchronous Goroutine Pings:** Background threads concurrently scan your services every 30s, streaming live status pings to the UI via HTMX without blocking your browser.
  * **Ecosystem Auto-Discovery:** Interfaces with the environment (like the local Docker socket) to auto-detect newly deployed services using simple container labels.
  * **Direct Power Actions:** Send secure `Stop`, `Start`, or `Restart` signals to microservices directly from the dashboard interface cards.

### 4. Micro-JWT Access Control
* **The Legacy Problem:** Sharing your landing page with family members usually means exposing your sensitive admin tools (Proxmox, Portainer) or setting up massive external proxy layers.
* **The AMUD Solution:** Built-in cryptographic user management. Admins see the full cluster control array; guests or family profiles get a stripped-down, read-only media configuration out of the box.

---

## Microscopic Production Footprint

| Dimension | Heimdall Application Dashboard | AMUD Dashboard |
| :--- | :---: | :---: |
| **Engine** | PHP 8+ / Laravel | Go 1.22+ Standard Library |
| **Runtime Overhead** | High (Interpreted PHP-FPM) | Zero (Native Compiled Machine Code) |
| **Assets Injection** | Read from host disk paths | Statically embedded into binary (`//go:embed`) |
| **Idle RAM Footprint** | 80MB - 150MB | **< 20MB** |
| **Boot Time** | 2 - 5 seconds | **Sub-millisecond (Instant)** |
