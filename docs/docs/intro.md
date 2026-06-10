---
sidebar_position: 1
---

# Introduction

Welcome to **AMUD (Advanced Modern Unified Dashboard)**.

AMUD is a high-performance, intelligent home lab cockpit engineered strictly for resource-constrained environments. While legacy dashboards demand heavy runtimes, bloated frameworks, and complex text-file configurations, AMUD provides a single-binary, zero-dependency ecosystem control center that idles at roughly **~26MB of RAM** (combined server and agent) with a **~660MB disk footprint** when deployed as a full Debian LXC container.

## How AMUD Works (Architecture)

Below is an overview of how the AMUD Dashboard and Telemetry Agent communicate to aggregate metrics and report container status in real-time.

![AMUD Architecture Diagram](/img/amud-architecture.svg)

AMUD uses a decoupled client-server architecture:
- **amud-agent**: Runs on the hypervisor host (e.g. Proxmox VE) or docker host. It polls system metrics and container states, sending them through a fast Unix domain socket.
- **amud-socket (`amud.sock`)**: A shared Unix Domain socket. By using a secure socket instead of standard TCP network ports, we avoid exposure and overhead, transferring telemetry at lightning speed.
- **amud-server**: Listens to the socket and serves the dashboard user interface to your browser over HTTP. It utilizes lightweight **WebSockets** to stream live statistics directly to you.


## Why AMUD Demolishes Legacy Dashboards

### 1. Bare-Metal Resource Discipline
* **The Legacy Problem:** Legacy dashboards rely on heavy PHP/Laravel lifecycles, requiring background web servers (Nginx/Apache) and PHP-FPM daemons that swallow 150MB+ RAM just sitting idle. 
* **The AMUD Solution:** Written in pure, compiled Rust. It executes native machine code with zero interpreter overhead, running the entire dashboard, telemetry layer, and database inside a strict **~26MB RAM** envelope at idle in a full LXC container.

### 2. Zero-YAML, 100% UI-Driven Control
* **The Legacy Problem:** Next-gen dashboards force you to spend hours manually writing, indenting, and debugging hundreds of lines of complex YAML text files just to add a shortcut.
* **The AMUD Solution:** Powered by an embedded, ultra-fast **SQLite** architecture. You get the advanced layout categories, tagging, and sub-pages of a modern dashboard, but configured entirely through an elegant, reactive user interface. 

### 3. Active Cockpit vs. Passive Bookmarks
* **The Legacy Problem:** Traditional dashboards are just glorified lists of web links. If a service freezes or crashes, they are completely blind to it.
* **The AMUD Solution:** 
  * **Native LXC Telemetry:** AMUD natively polls your Proxmox host via `pvesh` to stream real-time CPU, RAM, and true ON/OFF statuses directly to your custom application cards.
  * **Asynchronous Tokio Telemetry:** Background threads concurrently poll your metrics and stream live updates to the UI via WebSockets without blocking your browser.

### 4. Admin vs. Guest Profiles
* **The Legacy Problem:** Sharing your landing page with family members usually means exposing your sensitive admin tools.
* **The AMUD Solution:** Built-in cryptographic user roles. Admins see the full cluster control array; guests or family profiles get a clean, read-only dashboard layout out of the box.

## Next Steps

- [Dashboard Configuration](./configuration.md) — appearance, grid columns, Jellyfin/Plex integrations
- [Security](./security.md) — Argon2id passwords, sessions, rate limits, HTTPS cookies
- [Troubleshooting](./troubleshooting.md) — upgrades, IPC auth, PWA cache, CLI recovery
