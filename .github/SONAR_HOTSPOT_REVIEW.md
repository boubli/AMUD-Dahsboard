# SonarCloud Security Hotspot Review Checklist

After pushing code fixes, complete these steps in **SonarCloud** (required for Quality Gate **Security Hotspots Reviewed = 100%**).

## How to open

GitHub Actions → **SonarCloud Code Analysis** run → **See analysis details on SonarQube Cloud** → **Security Hotspots**

## Hotspots to review (mark **Safe** or **Fixed**)

| # | File | Topic | Recommended action |
|---|------|-------|-------------------|
| 1 | `Dockerfile` | `FROM scratch` runs as root | **Safe** — static musl binary, minimal homelab image, data on volume; comment in Dockerfile |
| 2 | `security.rs` → `get_rss_url_allowed` | Outbound HTTP (SSRF) | **Safe** — admin-only RSS; `url_allowed_for_rss_feed` blocks loopback/metadata; redirects disabled |
| 3 | `scripts/refactor-themes.py` | ReDoS in regex | **Fixed** — flat `[^{}]*` block matchers; helpers split cognitive complexity |
| 4 | `settings.html` RSS tab | DOM / `innerHTML` | **Fixed** — RSS tables use `appendLucideIcon` / `createLucideIconButton` |

If Sonar still lists separate hotspots in `rss_discover.rs` or `rss.rs`, they route through `get_rss_url_allowed` — mark **Safe** with the same SSRF justification.

## Quality Gate targets

- **Security Hotspots:** 100% reviewed
- **Security / Reliability / Maintainability:** A

## If gate still fails

1. Re-run analysis after push to `main`.
2. Review every open hotspot under **Security Hotspots** (not only Issues).
3. Confirm **New Code** period in Sonar project settings.
