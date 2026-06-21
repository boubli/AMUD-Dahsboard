---
slug: securing-homelab-dashboard
title: Securing a Dashboard That Knows Where Everything Lives
authors: [boubli]
tags: [security, selfhosted]
description: Argon2id passwords, encrypted integration secrets, CSRF, rate limiting — what AMUD actually ships with.
---

Your dashboard is a map to your entire homelab. Treat it like one.

## Passwords

**Argon2id** hashes in SQLite. Not SHA-256 alone, not md5 (please). Older installs with legacy SHA-256 get transparently re-hashed on next login.

## Sessions

32-byte random tokens, 24h lifetime, `HttpOnly`, `SameSite=Strict`. Behind HTTPS set `AMUD_SECURE_COOKIES=1` or you're leaving cookies exposed on misconfigured proxies.

## Brute force

5 failed logins per username in 5 minutes. Enough to stop dumb scripts, not enough to lock you out forever if you typo once.

## Integration secrets

Plex tokens, Jellyfin keys, Proxmox API tokens, HA tokens — **AES-GCM encrypted at rest** with a host keyfile (`.amud-secrets-key`). Backup that file with your database or restores get awkward.

## CSRF + CSP

State-changing requests need CSRF tokens. CSP with per-request nonces on scripts. Boring security stuff that matters when your dashboard is internet-facing behind a reverse proxy.

## Production checklist

- Change `admin` / `password` (seriously)
- HTTPS + `AMUD_SECURE_COOKIES=1`
- Restricted Proxmox API user, not root
- Backup `amud.db` and `.amud-secrets-key`

Full doc: [/docs/security](/docs/security)
