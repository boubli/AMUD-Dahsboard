# Draft reply for GitHub issue #15 (paste manually)

Thanks for reporting this — sorry for the frustration, especially right after an upgrade.

From your screenshot it looks like the **app name and ONLINE badge** render, but the **card body is empty** (no CPU/RAM, no integration stats). That helps narrow it down: URL health checks are working, but container metrics and/or integrations may not be showing.

**Could you try these and reply back?**

1. Hard refresh: **Ctrl+Shift+R** (or clear site data / unregister the service worker — stale cache after upgrades is common).
2. Top bar **WebSocket pill** — does it say **Live**, **Offline**, or **Reconnecting**?
3. Edit one app (e.g. Radarr) — is **Show container metrics** checked?
4. Any **Integration** configured on those apps? If yes, open DevTools → Network and check whether `/api/apps/<id>/integration` returns 200.
5. Browser + version (Chrome, Firefox, Safari, etc.)?

**Optional but very helpful:** View page source on the dashboard, search inside one card for `app-card-metrics-slot` or `data-lxc-metrics`. If those tags are missing, metrics are turned off in app settings. If they are present but invisible, that points to a layout/CSS issue we are fixing.

We are testing a Docker bridge setup on Proxmox to mirror Unraid and have a CSS patch ready for card metrics clipping after the v1.5.5.9 bento grid change. I will not ask you to upgrade again until we have a build we have validated — will follow up here with a tag when ready.

Thanks for sticking with the project.
