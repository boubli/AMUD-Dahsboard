---
sidebar_position: 6
---

# Reverse Proxy Configuration

Exposing AMUD securely over HTTPS using a custom domain name is a standard practice for home lab developers. To do this, you should deploy a reverse proxy in front of the dashboard.

:::important WebSocket Support is Mandatory
AMUD relies on a persistent WebSocket connection to stream real-time system metrics (CPU, RAM, Disk) and live container states. If your reverse proxy does not forward WebSocket upgrade headers, the dashboard will load, but the telemetry graphs and app cards will remain frozen at 0% or show connectivity errors.
:::

---

## 1. Nginx (Hardened Configuration)

Below is an Nginx virtual host configuration that secures traffic using modern SSL standards (TLSv1.3, strong ciphers) and injects HTTP security headers.

Create or edit your site config (e.g. `/etc/nginx/sites-available/amud`):

```nginx title="/etc/nginx/sites-available/amud"
server {
    listen 80;
    server_name amud.yourdomain.com;
    
    # Redirect all HTTP requests to HTTPS
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name amud.yourdomain.com;

    # SSL Certificates (Managed via Let's Encrypt / Certbot)
    ssl_certificate /etc/letsencrypt/live/amud.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/amud.yourdomain.com/privkey.pem;

    # Hardened SSL Parameters
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384';
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;
    ssl_session_tickets off;

    # HTTP Security Headers
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ws: wss:;" always;

    location / {
        proxy_pass http://127.0.0.1:8000; # Target IP and Port of your amud-server
        
        # Standard Forwarding Headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded-for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket Support (CRITICAL)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Prevent idle WebSocket connections from timing out
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }
}
```

Verify the configuration syntax and reload Nginx:
```bash
sudo nginx -t
sudo systemctl reload nginx
```

---

## 2. Traefik v2 / v3 (Docker & File Configs)

Traefik is a modern, cloud-native edge router. We provide configurations for both Docker Compose setups and standalone files.

### Option A: Docker Compose Labels (Dynamic Routing)
If you deploy AMUD using Docker Compose and run Traefik in the same Docker network, append these labels to your `amud-dashboard` service block:

```yaml title="docker-compose.yml"
services:
  amud-dashboard:
    image: tradmss/amud-dashboard:latest
    container_name: amud-dashboard
    networks:
      - traefik-public
    volumes:
      - amud_data:/app/data
      - amud_run:/var/run/amud
    restart: unless-stopped
    labels:
      - "traefik.enable=true"
      # HTTP Router
      - "traefik.http.routers.amud-http.rule=Host(`amud.yourdomain.com`)"
      - "traefik.http.routers.amud-http.entrypoints=web"
      - "traefik.http.routers.amud-http.middlewares=redirect-to-https"
      # HTTPS Router
      - "traefik.http.routers.amud-secure.rule=Host(`amud.yourdomain.com`)"
      - "traefik.http.routers.amud-secure.entrypoints=websecure"
      - "traefik.http.routers.amud-secure.tls=true"
      - "traefik.http.routers.amud-secure.tls.certresolver=myresolver" # Replace with your TLS resolver
      # Target Port
      - "traefik.http.services.amud-service.loadbalancer.server.port=8000"
      # Security Middleware (HSTS, Headers)
      - "traefik.http.middlewares.amud-headers.headers.sslredirect=true"
      - "traefik.http.middlewares.amud-headers.headers.stsSeconds=63072000"
      - "traefik.http.middlewares.amud-headers.headers.stsIncludeSubdomains=true"
      - "traefik.http.middlewares.amud-headers.headers.stsPreload=true"
      - "traefik.http.middlewares.amud-headers.headers.customresponseheaders.X-Robots-Tag=none"
      - "traefik.http.routers.amud-secure.middlewares=amud-headers"

networks:
  traefik-public:
    external: true
```

:::tip WebSocket compatibility in Traefik
Traefik handles WebSocket protocol upgrading automatically. No separate headers or middleware are required for WebSocket traffic.
:::

---

### Option B: Traefik File Provider (Bare-Metal / LXC)
If you run Traefik in docker but AMUD is hosted on bare metal or in a separate Proxmox LXC container, define a file provider router rule:

```yaml title="dynamic_conf.yml"
http:
  routers:
    amud-router:
      rule: "Host(`amud.yourdomain.com`)"
      entryPoints:
        - websecure
      service: amud-service
      tls:
        certResolver: myresolver

  services:
    amud-service:
      loadBalancer:
        servers:
          - url: "http://10.0.0.101:8000" # LXC Container IP
```

---

## 3. Caddy (Modern & Simple Config)

Caddy handles automatic HTTPS certificate procurement, renewal, and WebSocket upgrades natively.

Edit your `/etc/caddy/Caddyfile`:

```caddy title="/etc/caddy/Caddyfile"
amud.yourdomain.com {
    # Forward requests to AMUD
    reverse_proxy localhost:8000

    # Inject security headers
    header {
        # Enable HTTP Strict Transport Security
        Strict-Transport-Security "max-age=63072000; includeSubDomains; preload"
        # Prevent clickjacking
        X-Frame-Options "DENY"
        # Prevent MIME-type sniffing
        X-Content-Type-Options "nosniff"
        # Secure Referrer configuration
        Referrer-Policy "no-referrer-when-downgrade"
    }
}
```

Reload Caddy to apply changes:
```bash
sudo systemctl reload caddy
```

---

## 4. Apache HTTPD (Enterprise Configuration)

For environments running Apache HTTPD as the corporate edge proxy, you must enable `mod_proxy`, `mod_proxy_wstunnel`, and `mod_rewrite` to forward standard HTTP traffic and intercept WebSocket upgrades.

### Step 1: Enable required modules
```bash
sudo a2enmod proxy proxy_http proxy_wstunnel rewrite ssl headers
```

### Step 2: Configure Virtual Host
Create or edit your site configuration (e.g. `/etc/apache2/sites-available/amud.conf`):

```apache title="/etc/apache2/sites-available/amud.conf"
<VirtualHost *:80>
    ServerName amud.yourdomain.com
    Redirect permanent / https://amud.yourdomain.com/
</VirtualHost>

<VirtualHost *:443>
    ServerName amud.yourdomain.com

    SSLEngine on
    SSLCertificateFile /etc/letsencrypt/live/amud.yourdomain.com/fullchain.pem
    SSLCertificateKeyFile /etc/letsencrypt/live/amud.yourdomain.com/privkey.pem

    # Secure Header Injection
    Header always set Strict-Transport-Security "max-age=63072000; includeSubDomains; preload"
    Header always set X-Frame-Options "DENY"
    Header always set X-Content-Type-Options "nosniff"

    # Reverse Proxy Configuration
    ProxyRequests Off
    ProxyPreserveHost On

    # WebSocket Interceptor (Mod_Rewrite)
    RewriteEngine On
    RewriteCond %{HTTP:Upgrade} =websocket [NC]
    RewriteRule ^/(.*)           ws://127.0.0.1:8000/$1 [P,L]

    # Standard HTTP Routing
    ProxyPass / http://127.0.0.1:8000/
    ProxyPassReverse / http://127.0.0.1:8000/
</VirtualHost>
```

Restart Apache:
```bash
sudo systemctl restart apache2
```
