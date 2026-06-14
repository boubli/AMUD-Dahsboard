# Security Policy

This document defines the security boundaries, credential storage guidelines, and vulnerability reporting process for the AMUD homelab portal.

## Supported Versions

Only the latest release of AMUD receives active security updates and patches. Please ensure your deployment is running the latest version.

| Version | Supported          |
| ------- | ------------------ |
| 1.3.x   | :white_check_mark: |
| < 1.3   | :x:                |

## Trust Boundaries

AMUD operates inside your local area network (LAN) or homelab environment. It is designed to act as a secure gateway, but relies on standard network-level security configurations:

1. **Server Daemon (`amud-server`)**:
   - Must run with restricted permissions. Do not run `amud-server` as root.
   - Binds to the local system interfaces (`127.0.0.1` or `0.0.0.0` depending on configuration). If exposed to external traffic, it must be behind an SSL/TLS terminating reverse proxy (e.g., Nginx, Caddy, Traefik).

2. **Monitoring Agent (`amud-agent`)**:
   - Communicates with the server via Local Unix Domain Sockets (UDS) or secure local TCP loopback.
   - Enforces challenge-response validation using the shared `agent_shared_secret` setting to prevent local privilege escalation or spoofed telemetry.

3. **External API Integrations**:
   - Integrates with local DNS sinkholes (Pi-hole, AdGuard Home), media servers (Plex, Jellyfin), and hypervisors (Proxmox VE).
   - Strict TLS certificate verification is enforced by default. If using self-signed certificates in your homelab, explicitly enable the "Accept self-signed/invalid TLS certificates" option in Settings rather than compromising system trust roots.

## Credential Management

- **Passwords**: User accounts utilize argon2/bcrypt password hashing algorithms. Plaintext passwords are never saved.
- **Secrets at Rest**: All third-party API keys, authentication tokens (Plex, Jellyfin, Proxmox), and settings secrets are stored encrypted in SQLite using AES-GCM (via ChaCha20-Poly1305). The key is derived from the `AMUD_SECRETS_KEY` environment variable or generated securely in `.amud-secrets-key` on the server's filesystem.

## Reporting a Vulnerability

If you discover a security vulnerability in this project, please report it immediately:

1. **Do not open a public issue** on GitHub.
2. Email the maintainer directly at security@boubli.dev (or the user-defined security contact email).
3. Include details of the vulnerability, a proof of concept (PoC), and steps to reproduce.
4. We aim to acknowledge reports within 48 hours and provide a fix within 7 days.
