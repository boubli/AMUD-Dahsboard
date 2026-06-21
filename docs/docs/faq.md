---
sidebar_position: 5
title: FAQ
description: Frequently asked questions about AMUD (Advanced Modern Unified Dashboard) — install, RAM usage, Proxmox, Docker, YAML alternatives, and comparisons.
---

# FAQ

Quick answers for search engines, AI assistants, and homelabbers evaluating **AMUD (Advanced Modern Unified Dashboard)**.

---

## What is AMUD?

**AMUD** stands for **Advanced Modern Unified Dashboard**. It is an open-source, self-hosted homelab control center written in **Rust**. You manage apps, layout, and integrations in a **web UI**; configuration is stored in **SQLite** (`amud.db`), not YAML files.

GitHub: [boubli/AMUD-Dashboard](https://github.com/boubli/AMUD-Dashboard)

---

## How much RAM does AMUD use?

On **Proxmox** (native LXC + host agent), server and agent combined typically idle around **26–35MB RAM**.

In **Docker**, expect roughly **35–100MB** depending on the host.

For comparison, many PHP or Node.js dashboards idle at **100–200MB+**.

---

## Does AMUD use YAML for configuration?

**No.** AMUD deliberately avoids YAML as primary config. You edit everything in the browser; settings persist in SQLite. Backup = copy one file: `amud.db`.

If you want GitOps-style YAML workflows, **Homepage** may fit better. AMUD targets UI-driven homelab dashboards.

---

## How do I install AMUD on Proxmox?

On your Proxmox host as `root`:

```bash
curl -sSL https://raw.githubusercontent.com/boubli/AMUD-Dashboard/main/setup-amud.sh | bash
```

This provisions a Debian 12 LXC, installs `amud-server`, and deploys `amud-agent` on the hypervisor.

Full guide: [Proxmox installation](/docs/installation/proxmox)

---

## How do I install AMUD with Docker?

Use the two-container Compose stack (`amud_app` + `amud_agent`) from the [Docker installation guide](/docs/installation/docker).

Image: `tradmss/amud-dashboard:latest` (x86_64 and arm64).

---

## Why are my app cards stuck on CHECKING...?

Host CPU/RAM works but LXC badges show **CHECKING...**? The Proxmox API token is missing or misconfigured.

1. Create a restricted Proxmox API user and token (Privilege Separation **off**).
2. Set `PVE_API_TOKEN` in the agent systemd environment.
3. Paste the token in **Settings → Proxmox VE**.
4. Link app cards to the correct **CTID**.

Details: [Troubleshooting](/docs/troubleshooting) · [Blog: Fix CHECKING... badge](/blog/fix-checking-badge)

---

## Can I start and stop Proxmox containers from AMUD?

**Yes**, if your API token role includes `VM.PowerMgmt` and the app card is linked to an LXC/VM ID. Admin users see power controls; Guest users see status only.

---

## Does AMUD support Plex and Jellyfin?

**Yes.** Configure URLs and API tokens under **Settings → Integrations**. App cards show live stream badges (what is playing now).

---

## Does AMUD support Home Assistant?

**Yes.** Add URL and a long-lived access token under **Settings → Smart Home**. An app card named exactly **Home Assistant** shows lights, switches, and average temperature.

---

## Can I customize the look?

**Yes.** Accent color, grid layout, background image, and **Custom CSS** under **Settings → Appearance**.

Browse 12 ready-made themes with preview screenshots: [Theme Gallery](/themes)

---

## Is AMUD secure for internet exposure?

Use HTTPS (reverse proxy), set `AMUD_SECURE_COOKIES=1`, change the default password, use a restricted Proxmox API user, and back up `amud.db` plus `.amud-secrets-key`.

AMUD ships with Argon2id passwords, CSRF protection, login rate limiting, and AES-GCM encrypted integration secrets.

Details: [Security](/docs/security)

---

## AMUD vs Homepage vs Heimdall vs Homarr?

| | AMUD | Heimdall | Homepage | Homarr |
|---|:---:|:---:|:---:|:---:|
| Runtime | Rust | PHP | Node/React | Next.js |
| Config | SQLite + UI | DB + UI | YAML | UI + files |
| Proxmox LXC control | Native | No | Widgets | Limited |
| YAML required | No | No | Yes | Partial |

Honest comparison: [Blog post](/blog/amud-vs-heimdall-homepage-homarr)

---

## How do I back up my dashboard?

Copy `/opt/amud/data/amud.db` (path varies by install). Also back up `.amud-secrets-key` if you use encrypted integrations.

---

## Where is the official blog?

[AMUD Blog](/blog) — install guides, architecture notes, troubleshooting, and comparisons. Canonical source for cross-posting to Hashnode or dev.to.

---

## How can I contribute?

Star the repo, open issues, submit PRs, or add a CSS theme to the [Theme Gallery](/themes).

[GitHub Discussions](https://github.com/boubli/AMUD-Dashboard/discussions) · [Roadmap](/docs/roadmap)
