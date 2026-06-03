---
sidebar_position: 2
---

# Docker Deployment

You can deploy AMUD effortlessly using the official Docker images. We provide instructions for both `Docker Compose` and the `Docker CLI`.

> [!WARNING]
> Running AMUD inside Docker will currently disable the native Proxmox LXC telemetry features (live LXC CPU/RAM stats), as the dashboard runs isolated inside the container rather than on the host bare-metal.

## Option A: Docker Compose (Recommended)

Create a `docker-compose.yml` file with the following contents:

```yaml
version: '3.8'

services:
  amud-dashboard:
    image: tradmss/amud-dashboard:latest
    container_name: amud-dashboard
    ports:
      - "8000:8000"
    volumes:
      - ./amud_data:/app/data
    restart: unless-stopped
```

To start the AMUD ecosystem, run:

```bash
docker-compose up -d
```

## Option B: Docker CLI (docker run)

If you prefer to run a single command without creating a compose file, use the following `docker run` command:

```bash
docker run -d \
  --name amud-dashboard \
  -p 8000:8000 \
  -v amud_data:/app/data \
  --restart unless-stopped \
  tradmss/amud-dashboard:latest
```

## Accessing the Dashboard

Navigate to your server's IP address on port 8000:
```
http://<YOUR_SERVER_IP>:8000/
```

> [!TIP]  
> **Default Admin Login:**  
> Username: `admin`  
> Password: `password`  
