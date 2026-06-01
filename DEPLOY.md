# AMUD Deployment Guide

This document outlines the verified production deployment methodologies for the AMUD Dashboard across micro-resource footprints, virtual environments, and container orchestrators.

---

## 1. Proxmox VE Autopilot Deployment (LXC Native)

The elite deployment method for Proxmox clusters uses our native shell script (`setup-amud.sh`) to automate container provisioning and hardware scaling directly from the host.

### Automation Script Functions:
1. Provisions a minimal Debian Linux Container (LXC).
2. Allocates a strict maximum pool of **512MB RAM** and 1 vCPU.
3. Automatically sets up the base runtime environment, clones the repository, and pulls the single-service micro-container image.
4. Downscales host-level CPU priority variables to production constraints.

### Execution:
SSH into your Proxmox VE host and execute the automated onboarding workflow:
```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/boubli/AMUD-Dahsboard/main/setup-amud.sh)"
```

---

## 2. Portainer Stack Deployment (Web Editor)

For standard containerized host management panels:
1. Open your Portainer Web UI.
2. Select **Stacks** -> **Add Stack**.
3. Under the Web Editor panel, paste the following single-service definition:
```yaml
version: '3.8'

services:
  app:
    image: boubli/amud:latest
    container_name: amud_app
    restart: always
    ports:
      - "8000:8000"
    environment:
      - DB_PATH=/app/data/amud.db
      - PORT=8000
    volumes:
      - ./data:/app/data
```
4. Click **Deploy the stack**.

---

## 3. Standalone Docker Compose (CLI Native)

To compile and launch the dashboard locally on any Linux server:
```bash
# Clone the repository
git clone https://github.com/boubli/AMUD-Dahsboard.git
cd AMUD-Dahsboard

# Run the compose stack in detached mode
docker compose up -d
```
The dashboard service is now serving at `http://localhost:8000`.
