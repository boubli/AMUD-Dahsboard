---
sidebar_position: 2
---

# Docker Deployment

You can deploy AMUD effortlessly using the official Docker images. We provide instructions for both `Docker Compose` and the `Docker CLI`.

> [!NOTE]
> The included AMUD Agent container automatically maps your Docker socket. This allows the dashboard to stream live **Running/Stopped** indicators for all of your Docker containers directly into the UI!

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
      - ./amud_run:/var/run/amud
    restart: unless-stopped

  amud-agent:
    image: tradmss/amud-dashboard:latest
    container_name: amud-agent
    entrypoint: ["/app/amud-agent"]
    volumes:
      - ./amud_run:/var/run/amud
      - /var/run/docker.sock:/var/run/docker.sock:ro
    restart: unless-stopped
```

To start the AMUD ecosystem, run:

```bash
docker-compose up -d
```

## Option B: Docker CLI (docker run)

If you prefer to run a single command without creating a compose file, you will need to create a shared docker volume for the socket, and run both containers:

```bash
# Create shared volume for IPC
docker volume create amud_run

# Start the dashboard server
docker run -d \
  --name amud-dashboard \
  -p 8000:8000 \
  -v amud_data:/app/data \
  -v amud_run:/var/run/amud \
  --restart unless-stopped \
  tradmss/amud-dashboard:latest

# Start the Docker telemetry agent
docker run -d \
  --name amud-agent \
  -v amud_run:/var/run/amud \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  --restart unless-stopped \
  --entrypoint "/app/amud-agent" \
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
