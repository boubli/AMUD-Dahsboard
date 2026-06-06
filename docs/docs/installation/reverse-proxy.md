---
sidebar_position: 5
---

# Reverse Proxy Configuration

To access AMUD securely over HTTPS from outside your local network, or to use a custom domain, you should deploy a reverse proxy in front of the dashboard.

:::important WebSocket Support Required
AMUD relies on a persistent WebSocket connection to stream real-time CPU, RAM, disk metrics, and LXC statuses. If your reverse proxy is not configured to support WebSockets, the page will load, but the telemetry stream will fail, showing connection errors or stagnant metrics.
:::

---

## 1. Nginx (Standard Config)

If you are using a raw Nginx installation, add the following block to your virtual host configuration file (typically in `/etc/nginx/sites-available/`):

```nginx title="/etc/nginx/sites-available/amud"
server {
    listen 80;
    server_name amud.yourdomain.com;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name amud.yourdomain.com;

    # SSL configuration (Certbot paths shown as example)
    ssl_certificate /etc/letsencrypt/live/amud.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/amud.yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://localhost:8000; # Address of your amud-server
        
        # Standard Proxy Headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded-for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket Support (CRITICAL)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Timeouts to prevent WebSocket connection from dropping idle
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }
}
```

After modifying the file, verify and restart Nginx:
```bash
sudo nginx -t
sudo systemctl restart nginx
```

---

## 2. Nginx Proxy Manager (NPM)

Nginx Proxy Manager provides a user-friendly Web UI. To set up AMUD in NPM:

1. Log in to your NPM Admin Panel.
2. Go to **Hosts** → **Proxy Hosts** and click **Add Proxy Host**.
3. Fill in the **Details** tab:
   - **Domain Names:** `amud.yourdomain.com`
   - **Scheme:** `http`
   - **Forward Hostname / IP:** The IP address of your AMUD Dashboard server/LXC container.
   - **Forward Port:** `8000`
   - **⚠️ Toggle ON "Websockets Support"** (this is vital for metrics).
4. Go to the **SSL** tab:
   - Select **Request a new SSL Certificate** (or use an existing one).
   - Toggle ON **Force SSL** for secure redirecting.
5. Click **Save**.

---

## 3. Caddy

Caddy is an excellent modern web server that automates SSL acquisition and renews it out-of-the-box. Caddy supports WebSocket upgrading by default without extra parameters.

Add the following to your `Caddyfile` (typically at `/etc/caddy/Caddyfile`):

```caddy title="/etc/caddy/Caddyfile"
amud.yourdomain.com {
    reverse_proxy localhost:8000
}
```

If AMUD is hosted on a different machine or LXC container:

```caddy
amud.yourdomain.com {
    reverse_proxy 10.0.0.101:8000
}
```

Apply the configuration changes:
```bash
sudo systemctl reload caddy
```

---

## 4. Cloudflare Tunnels (cloudflared)

Cloudflare Tunnels allow you to expose your AMUD Dashboard safely without opening any ports on your home router firewall.

1. Install and authenticate `cloudflared` on your server.
2. In the **Cloudflare Zero Trust** dashboard, navigate to **Networks** → **Tunnels**.
3. Create a new tunnel or edit an existing one.
4. Add a **Public Hostname**:
   - **Subdomain/Domain:** `amud.yourdomain.com`
   - **Service Type:** `HTTP`
   - **URL:** `localhost:8000` (or the local IP/port of the container)
5. Under **Network** settings in your Cloudflare dashboard (for the main domain):
   - Make sure **WebSockets** is toggled **ON** (enabled by default in Cloudflare).

:::tip SSL Mode in Cloudflare
We recommend setting Cloudflare SSL/TLS encryption mode to **Full (strict)** if you are using an internal SSL cert, or **Flexible** if using HTTP internally.
:::
