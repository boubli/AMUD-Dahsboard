---
sidebar_position: 3
---

# Portainer Deployment

Portainer is a popular web-based interface for managing Docker environments. You can deploy the complete AMUD dashboard and agent ecosystem within Portainer using a Custom Stack (which maps directly to a Docker Compose definition).

---

## 1. How Portainer Manages AMUD

When you deploy AMUD as a Portainer Stack, Portainer provisions two containers and connects them via a shared volume:
* **`amud-dashboard`**: Serves the web-based cockpit.
* **`amud-agent`**: Queries the host's `/var/run/docker.sock` to detect container states and streams this telemetry to the dashboard via a Unix socket.

---

## 2. Step-by-Step Stack Deployment

Follow these steps to deploy AMUD on your Portainer instance:

1. Log in to your **Portainer Dashboard**.
2. Select your environment from the home screen (usually named **local**).
3. Select **Stacks** from the left-hand navigation sidebar.
4. Click the **+ Add stack** button in the top-right corner.
5. Configure the stack details:
   - **Name**: `amud`
   - **Build method**: Select **Web editor**
6. Paste the following configuration into the web editor panel:

```yaml title="AMUD Portainer Stack"
version: '3.8'

services:
  amud-dashboard:
    image: tradmss/amud-dashboard:latest
    container_name: amud-dashboard
    ports:
      - "8000:8000"
    environment:
      - PORT=8000
      - DB_PATH=/app/data/amud.db
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
    volumes:
      - amud_data:/app/data
      - amud_run:/var/run/amud
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '0.25'
          memory: 128M

  amud-agent:
    image: tradmss/amud-dashboard:latest
    container_name: amud-agent
    entrypoint: ["/app/amud-agent"]
    environment:
      - AMUD_SOCKET_PATH=/var/run/amud/amud.sock
    volumes:
      - amud_run:/var/run/amud
      - /var/run/docker.sock:/var/run/docker.sock:ro
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '0.10'
          memory: 64M

volumes:
  amud_data:
    name: amud_data
  amud_run:
    name: amud_run
```

7. **Review Environment Variables**:
   Under the editor, you can optionally define environment variables if you wish to override parameters like `PORT` dynamically, though they are already set in the stack file.
8. **Deploy**:
   Scroll to the bottom of the page and click **Deploy the stack**.

Portainer will download the required images, build the networking, map the volumes, and spin up the services.

---

## 3. Persistent Volumes & Data Backups

Portainer creates two named Docker volumes:
* **`amud_data`**: Stores the SQLite database (`amud.db`) which contains your dashboard layouts, custom app cards, user configurations, and preferences.
* **`amud_run`**: An ephemeral communication directory containing the Unix socket (`amud.sock`). This volume does not require backups and is recreated on container startup.

To backup your dashboard configuration:
1. In Portainer, go to **Volumes**.
2. Locate the volume named `amud_data`.
3. Back up the files in the directory indicated by the host path (e.g. `/var/lib/docker/volumes/amud_data/_data`).

---

## 4. Security Hardening inside Portainer

To secure your Portainer deployment:
* **Read-Only Docker Socket**: Ensure `/var/run/docker.sock` in the volume mount has the Access Control flag set to Read-only (`:ro`). This prevents the telemetry agent from mutating container states on the host.
* **Network Isolation**: If you deploy multiple stacks, consider placing AMUD on a isolated internal Docker bridge network, and use a reverse proxy stack (like Nginx Proxy Manager) to route traffic to `amud-dashboard` port `8000`.

---

## 5. Verification

Navigate to your server's IP address on port 8000:
```
http://<YOUR_SERVER_IP>:8000/
```

:::tip Default Credentials
- **Username**: `admin`
- **Password**: `admin` (or `password` depending on version configuration)
:::
