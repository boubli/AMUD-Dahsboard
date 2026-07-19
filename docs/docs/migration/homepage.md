---
title: Migrate from Homepage
---

# Migrate from Homepage

AMUD does not use YAML as primary config, but supports **one-time import** from Homepage files.

## What you need

- `services.yaml` (and optionally `widgets.yaml`) from your Homepage config directory

## Import steps

1. Open **Settings → Infrastructure → Import from Homepage**
2. Paste `services.yaml` into the text area
3. Click **Preview** — AMUD maps known `widget.type` values to integration types
4. Click **Import** — apps are inserted into SQLite (duplicates by name/URL are skipped)

## API (admin)

- `POST /api/migration/homepage/preview` — form field `services_yaml`
- `POST /api/migration/homepage/import` — form field `services_yaml` + CSRF

## Docker label discovery

Containers with Homepage-style labels (`homepage.name`, `homepage.href`, `homepage.widget.type`) are included when using **Discover Docker** from Settings.

## After import

- Re-enter API keys if they were env-only in Homepage (AMUD stores them encrypted)
- Enable **Accept invalid certs** if you use HTTPS with self-signed homelab certs
- Remove Homepage when satisfied — AMUD agent telemetry and feeds are separate features Homepage does not provide
