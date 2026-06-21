---
slug: encrypted-secrets-at-rest
title: Where Your Plex Token Actually Lives in AMUD Dashboard
authors: [boubli]
tags: [security, rust]
description: AES-GCM encrypted integration creds in SQLite, .amud-secrets-key on disk. What gets encrypted and what doesn't.
image: themes/assets/AMUD-Theme-Dracula.png
---

Storing `X-Plex-Token=abc123` in plain text in a yaml file on disk always felt wrong. Storing it in plain text in SQLite felt equally wrong.

<!-- truncate -->

AMUD Dashboard encrypts integration secrets **at rest** with AES-GCM. Plex, Jellyfin, Proxmox API tokens, Home Assistant tokens — encrypted blob in the database, not readable if someone copies `amud.db` without the key.

## The keyfile

`.amud-secrets-key` on the host. Derived key material for encryption/decryption at runtime.

**Back it up with your database.** Restore DB without key = your tokens are toast. You'll re-enter them in settings. Annoying but not catastrophic.

## What's not encrypted

App URLs, card names, layout stuff — none of that's secret. Your Jellyfin URL being `http://10.0.0.5:8096` isn't the sensitive part. The API key is.

Password **hashes** use Argon2id separately. Different layer. Same philosophy: don't store usable secrets in recoverable form.

## Threat model (realistic)

This stops "someone copied my backup drive" from leaking your Proxmox token. It doesn't stop someone with root on your running server — they own the box anyway.

Homelab threat model. Not banking.

Full security writeup: [/docs/security](/docs/security)

Rotate tokens in integrations settings if you think the DB leaked. Same as any other dashboard.
