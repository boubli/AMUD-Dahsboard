---
sidebar_position: 1
---

# Introduction

Welcome to **AMUD (Advanced Modern Unified Dashboard)**.

AMUD is a high-performance, intelligent home lab cockpit engineered strictly for resource-constrained environments. While legacy dashboards demand heavy runtimes, bloated frameworks, and complex text-file configurations, AMUD provides a single-binary, zero-dependency ecosystem control center that idles under **10MB of RAM** (combined server and agent).

## Why AMUD Demolishes Legacy Dashboards

### 1. Bare-Metal Resource Discipline
* **The Legacy Problem:** Legacy dashboards rely on heavy PHP/Laravel lifecycles, requiring background web servers (Nginx/Apache) and PHP-FPM daemons that swallow 150MB+ RAM just sitting idle. 
* **The AMUD Solution:** Written in pure, compiled Rust. It executes native machine code with zero interpreter overhead, running the entire dashboard, telemetry layer, and database inside a strict **~10MB RAM** envelope at idle.

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
