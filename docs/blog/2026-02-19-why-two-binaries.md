---
slug: why-two-binaries
title: Why AMUD Is Two Binaries Instead of One Blob
authors: [boubli]
tags: [rust, proxmox, homelab]
description: Server in LXC, agent on the hypervisor host. Separation of privileges and why the Unix socket exists.
---

"Why not one binary?"

Because on Proxmox the dashboard server lives in an **unprivileged LXC** and the thing that talks to `/proc`, the Docker socket, and the PVE API on port 8006 needs to live on the **host**.

Different trust boundaries. Different privilege levels.

## amud-server

- Serves HTTP/WebSockets
- Owns SQLite
- Runs as unprivileged `amud` user in the LXC
- Never needs root

## amud-agent

- Reads host hardware
- Queries Proxmox API
- Optionally reads Docker socket
- Runs on hypervisor host with the permissions that requires

They talk over `/opt/amud/run/amud.sock`. Bind-mounted into the LXC on Proxmox installs. Shared Docker volume in container deployments.

## Challenge-response auth

The socket isn't world-writable chaos. Agent proves it knows `AMUD_AGENT_SECRET` before the server accepts telemetry. Stops random local processes from feeding fake metrics.

## Could I merge them?

On single-box Docker or bare-metal where server and agent share a host, they're still separate processes for isolation. Server crash doesn't take down polling. Agent restart doesn't drop active web sessions.

Might be over-engineered for a homelab. I'd rather over-engineer the security boundary than under-engineer it.

[/docs/ARCHITECTURE](/docs/ARCHITECTURE)
