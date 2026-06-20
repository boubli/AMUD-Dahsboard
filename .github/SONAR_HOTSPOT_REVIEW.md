# SonarCloud Security Hotspot Review Checklist

After pushing the code fixes, complete these steps in **SonarCloud** (required for Quality Gate).

## How to open

GitHub Actions → failed **SonarCloud Code Analysis** run → **See analysis details on SonarQube Cloud**

## Hotspots to review (mark **Safe** or **Fixed**)

| # | File | Topic | Recommended action |
|---|------|-------|-------------------|
| 1 | `Dockerfile` line ~41 | `FROM scratch` runs as root | **Safe** — static musl binary, minimal homelab image, data on volume; comment added in Dockerfile |
| 2 | `audit.rs` | SQL string in `execute()` | **Safe** — parameterized `?1..?5`, no string concatenation |
| 3 | `audit.rs` | Logging / user data | **Fixed** — removed verbose success logs with user fields |
| 4 | `dashboard.rs` | Template injection | **Fixed** — `escape_html`, `safe_css_url`, `safe_accent_hex` on branding fields |
| 5 | `settings.html` | DOM XSS / `innerHTML` | **Fixed** — `textContent` / `createElement` via `admin.js` helpers |
| 6–9 | Sonar UI | Other new-code hotspots | Review each in UI; fix or mark Safe with one-line justification |

## Quality Gate targets

- **Security Hotspots:** 100% reviewed
- **Reliability on New Code:** A
- **Security on New Code:** A

## If gate still fails

1. Confirm `sonar-project.properties` `sonar.organization` matches your SonarCloud org slug.
2. Re-run analysis after push.
3. Check **New Code** period in Sonar project settings (may include older commits).
