---
sidebar_position: 4
title: Comparison
---

# AMUD vs Homepage vs Homarr

AMUD targets **full parity** with [Homepage](https://gethomepage.dev/) and [Homarr](https://homarr.dev/) while staying a **single Rust binary** (no Redis, no Node runtime).

## Platform

| Feature | AMUD | Homepage | Homarr |
|---------|:----:|:--------:|:------:|
| UI configuration | SQLite + web UI | YAML | Web UI |
| Idle RAM (typical) | 30–50 MB | Higher (Node) | Higher (Node + Redis) |
| Proxmox/Docker agent | Native | Docker stats | Partial |
| LXC power controls | Yes | No | Limited |
| OIDC | Yes | No | Yes |
| LDAP | Yes | No | Yes |
| Custom API widgets | Yes | Yes | Yes |
| Homepage YAML import | Yes | N/A | No |
| *arr calendar widget | Yes | Yes | Yes |
| Per-user boards | Yes | No | Yes |
| Audit log | Yes | No | Partial |

## Integrations

- **AMUD**: 130+ integration types (full cards + health-only + custom API)
- **Homepage**: ~150 service widgets
- **Homarr**: ~40 first-class integrations

See [Features](./features.md) for the live AMUD catalog. Use **Custom API** for any service not yet built-in.

## Migration

- [Import from Homepage](./migration/homepage.md)
- [Import from Homarr](./migration/homarr.md)
