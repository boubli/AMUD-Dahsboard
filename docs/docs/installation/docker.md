---
sidebar_position: 2
---

# Docker Compose Installation

If you prefer to run AMUD inside a standard Docker environment rather than the native Proxmox LXC ecosystem, you can use our official Docker Compose stack.

> [!WARNING]
> Running AMUD inside Docker will currently disable the native Proxmox LXC telemetry features (live LXC CPU/RAM stats), as the agent runs isolated inside the container rather than on the host bare-metal.

## docker-compose.yml

Create a `docker-compose.yml` file with the following contents:

```yaml
version: '3.8'

services:
  amud-dashboard:
    image: ghcr.io/boubli/amud-dashboard:latest
    container_name: amud-dashboard
    ports:
      - "8000:8000"
    volumes:
      - ./amud_data:/opt/amud/data
      - ./amud_run:/var/run/amud
    restart: unless-stopped

  amud-agent:
    image: ghcr.io/boubli/amud-agent:latest
    container_name: amud-agent
    volumes:
      - ./amud_run:/var/run/amud
    restart: unless-stopped
```

## Running the Stack

To start the AMUD ecosystem, run:

```bash
docker-compose up -d
```

## Accessing the Dashboard

Navigate to your server's IP address on port 8000:
```
http://<YOUR_SERVER_IP>:8000/
```

**Default Admin Login:**
Username: `admin`
Password: `password`
