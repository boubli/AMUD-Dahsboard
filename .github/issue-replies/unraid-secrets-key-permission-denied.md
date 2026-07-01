# Unraid: `.amud-secrets-key` permission denied

Use this template when closing or commenting on GitHub issues like [#16](https://github.com/boubli/AMUD-Dashboard/issues/16).

---

Thanks for the detailed report — this is a known **Unraid appdata permissions** issue, not a broken secrets/encryption feature.

**What happened:** On a fresh Unraid install, Community Applications creates appdata as `nobody:users` (UID 99). On images before v1.7.2, the dashboard ran as hardened root (`--cap-drop=ALL`) and could not write to that folder on first boot. The first failure is creating `/app/data/.amud-secrets-key`; SQLite would fail next for the same reason. The agent container often still appears to start because it does not write that file.

**Fixed in v1.7.2+:** The Docker image runs the dashboard as **PUID 99 / PGID 100** (Unraid defaults). Update and **recreate** the **AMUD-Dashboard** container from Community Applications.

**If you are on an older image or cannot update yet — manual fix (Unraid terminal):**

```bash
chown -R 99:100 /mnt/user/appdata/amud-dashboard/data
chown -R 99:100 /mnt/user/appdata/amud-dashboard/run
chmod -R 755 /mnt/user/appdata/amud-dashboard/data
chmod -R 770 /mnt/user/appdata/amud-dashboard/run
```

Then restart **AMUD-Dashboard**, then **AMUD-Agent**.

**Docs (full explanation + other Unraid pitfalls):**

- Troubleshooting — [Unraid: `.amud-secrets-key` permission denied](https://boubli.github.io/AMUD-Dashboard/docs/troubleshooting#unraid-secrets-key-permission-denied)
- Install guide — [Permission errors on appdata](https://boubli.github.io/AMUD-Dashboard/docs/installation/unraid#permission-errors-on-appdata)
- Docker deployment — [Docker install](https://boubli.github.io/AMUD-Dashboard/docs/installation/docker)

**Note:** Setting `AMUD_SECRETS_KEY` alone does not fix this — the database file still needs write access to the same folder.

If this persists after v1.7.2 + recreate, please reply with Unraid version, dashboard container log, and output of:

```bash
ls -la /mnt/user/appdata/amud-dashboard/
```
