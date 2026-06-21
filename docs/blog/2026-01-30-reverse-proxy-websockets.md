---
slug: reverse-proxy-websockets
title: Putting AMUD Behind Nginx Without Breaking Live Metrics
authors: [boubli]
tags: [homelab, security]
description: If your CPU graphs freeze at 0%, your reverse proxy probably isn't forwarding WebSocket upgrades. Fix inside.
---

Deployed AMUD behind Nginx, opened the page, everything looked fine except the CPU bar was stuck at 0% like the server was dead. Server wasn't dead. Nginx just ate the WebSocket upgrade.

AMUD streams telemetry over WebSockets. Your reverse proxy **must** forward upgrade headers or you get a pretty but useless dashboard.

## Nginx snippet that actually works

```nginx
location / {
    proxy_pass http://127.0.0.1:8000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;

    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
}
```

## HTTPS cookies

Once you're on HTTPS:

```bash
AMUD_SECURE_COOKIES=1
```

Restart the server. Forget this and you'll chase phantom login bugs.

## Caddy

```caddy
amud.homelab.local {
    reverse_proxy localhost:8000
}
```

Usually just works. Caddy's the easy mode.

Full configs: [/docs/installation/reverse-proxy](/docs/installation/reverse-proxy)

After setup: login works, graphs move, LXC badges update. If not, browser console first. Always browser console first.
