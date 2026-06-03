---
sidebar_position: 3
---

# Portainer Deployment

Portainer provides an excellent Web UI for managing Docker containers. You can deploy the AMUD Dashboard inside Portainer using a Custom Stack.

> [!WARNING]  
> Deploying via Docker/Portainer means the native Proxmox LXC telemetry features (live container CPU/RAM stats) will be disabled, since the dashboard is running inside an isolated Docker network instead of directly on your hypervisor.

## Deploying the Stack

1. Log in to your Portainer instance.
2. Select your environment (usually **local**).
3. In the left sidebar, click on **Stacks**.
4. Click the **+ Add stack** button in the top right.
5. Name your stack `amud`.
6. Choose the **Web editor** option and paste the following configuration:

```yaml
version: '3.8'

services:
  amud-dashboard:
    image: tradmss/amud-dashboard:latest
    container_name: amud-dashboard
    ports:
      - "8000:8000"
    volumes:
      - amud_data:/app/data
    restart: unless-stopped

volumes:
  amud_data:
    name: amud_data
```

7. Scroll down to the bottom and click **Deploy the stack**.

## Accessing the Dashboard

Once Portainer finishes downloading the image and starting the container, you can access your dashboard by navigating to your server's IP address on port `8000`:

```
http://<YOUR_SERVER_IP>:8000/
```

> [!TIP]  
> **Default Login Credentials:**  
> **Username:** `admin`  
> **Password:** `password`  
> *(We highly recommend changing these from the settings page after your first login!)*
