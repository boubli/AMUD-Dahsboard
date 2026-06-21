---
slug: contribute-to-amud
title: AMUD Is Open Source — Here's How to Actually Help
authors: [boubli]
tags: [homelab, selfhosted]
description: Stars, issues, PRs, themes, docs. Homelab projects grow from real users reporting real weird setups.
---

AMUD isn't a startup. It's a homelab tool I built because I wanted it. But it gets better when other people break it in ways I didn't imagine.

## Fastest help: star the repo

[github.com/boubli/AMUD-Dashboard](https://github.com/boubli/AMUD-Dashboard)

Stars help GitHub discovery and lists like awesome-selfhosted. Sounds shallow. Matters anyway.

## Try it and tell me your stack

Proxmox version, RAM numbers, screenshot of your dashboard. [GitHub Discussions](https://github.com/boubli/AMUD-Dashboard/discussions) is the place. "Works on my weird ARM NAS" posts are gold.

## Bug reports that get fixed fast

- Steps to reproduce
- `journalctl -u amud-agent -n 50` output
- Whether you're Docker or native LXC

Vague "doesn't work" issues sit longer. Fair warning.

## PR areas

| Area | Stack |
|------|-------|
| amud-server | Rust, Axum |
| amud-agent | Rust, Tokio |
| UI | HTML, Alpine.js, CSS |
| Docs | Docusaurus, Markdown |
| Themes | CSS variables |

Roadmap: [/docs/roadmap](/docs/roadmap) — ARM64 CI, webhooks, multi-node agents on the list.

## Submit a theme

CSS file + preview screenshot → PR to `docs/static/themes/` → shows up in [/themes](/themes).

That's a contribution normies can do without touching Rust. Please do.

Thanks for reading this series. Go break something and file an issue.
