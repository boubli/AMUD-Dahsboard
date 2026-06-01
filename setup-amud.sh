#!/usr/bin/env bash
# ==============================================================================
# setup-amud.sh (Unattended Installer)
# 
# Autopilot installation script for AMUD launcher dashboard ecosystem
# Target Host: Proxmox VE Host Shell (Run as root)
# Guest LXC OS: Debian 12 (unprivileged, nesting & keyctl enabled, memory-efficient)
# ==============================================================================

set -euo pipefail

# Print styled messages
info() {
    echo -e "\033[1;34m[INFO]\033[0m $1"
}

success() {
    echo -e "\033[1;32m[SUCCESS]\033[0m $1"
}

error() {
    echo -e "\033[1;31m[ERROR]\033[0m $1" >&2
}

PCT_EXPORTS='export DEBIAN_FRONTEND=noninteractive; export LANG=C.UTF-8; export LC_ALL=C.UTF-8;'

# 1. Host Execution Pre-checks
if [ ! -d "/etc/pve" ]; then
    error "This script must be executed directly on a Proxmox VE host shell."
    exit 1
fi

# Update pveam templates index before querying
pveam update >/dev/null 2>&1 || true

CT_ID=$(pvesh get /cluster/nextid)
CT_NAME="amud-dashboard"
CT_IP="dhcp"
TEMPLATE_DIR="/var/lib/vz/template/cache"
# Dynamically fetch the latest Debian 12 template from the Proxmox mirror
TEMPLATE_FILE=$(pveam available -section system | awk '$2 ~ /^debian-12-standard/ {print $2}' | head -n1)

cat << 'EOF'
  ______   __       __  __    __  _______  
 /      \ |  \     /  \|  \  |  \|       \ 
|  $$$$$$\| $$\   /  $$| $$  | $$| $$$$$$$\
| $$__| $$| $$$\ /  $$$| $$  | $$| $$  | $$
| $$    $$| $$$$\  $$$$| $$  | $$| $$  | $$
| $$$$$$$$| $$\$$ $$ $$| $$  | $$| $$  | $$
| $$  | $$| $$ \$$$| $$| $$__/ $$| $$__/ $$
| $$  | $$| $$  \$ | $$ \$$    $$| $$    $$
 \$$   \$$ \$$      \$$  \$$$$$$  \$$$$$$$ 
===============================================
      AMUD Dashboard Autopilot Installer
===============================================
EOF

info "Executing unattended AMUD Ecosystem Deployment..."
echo "------------------------------------------------"

# 2. Conflict Prevention: Clean Existing Installation
if pct status "$CT_ID" >/dev/null 2>&1; then
    info "Conflict detected: Container $CT_ID already exists. Stopping and destroying..."
    pct stop "$CT_ID" >/dev/null 2>&1 || true
    pct destroy "$CT_ID" >/dev/null 2>&1
    success "Existing container $CT_ID removed."
fi

# 3. Template Management
TEMPLATE_PATH="$TEMPLATE_DIR/$TEMPLATE_FILE"
if [ ! -f "$TEMPLATE_PATH" ]; then
    info "Downloading template $TEMPLATE_FILE..."
    pveam download local "system/$TEMPLATE_FILE" >/dev/null
else
    info "Template $TEMPLATE_FILE is already available locally."
fi

# 4. Container Resource Allocation
info "Creating container $CT_ID ($CT_NAME)..."
pct create "$CT_ID" "local:vztmpl/$TEMPLATE_FILE" \
    -cores 2 \
    -memory 2048 \
    -swap 2048 \
    -hostname "$CT_NAME" \
    -ostype debian \
    -storage local-lvm \
    -rootfs local-lvm:10 \
    -net0 "name=eth0,bridge=vmbr0,ip=$CT_IP" \
    -unprivileged 1 \
    -features nesting=1,keyctl=1 \
    -nameserver "1.1.1.1 8.8.8.8" \
    -start 1 >/dev/null

info "Waiting for container to boot and get network address..."
sleep 12

# Force system-wide locale to silence Perl and Apt warnings permanently.
pct exec "$CT_ID" -- bash -c "
  echo 'LC_ALL=C.UTF-8' >> /etc/environment
  echo 'LANG=C.UTF-8' >> /etc/environment
  ${PCT_EXPORTS}
  apt-get update -qq >/dev/null
  apt-get install -y -qq locales >/dev/null
  echo 'en_US.UTF-8 UTF-8' > /etc/locale.gen
  locale-gen >/dev/null
  update-locale LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8
"

# 5. Dependency Toolchain & Docker Installation
info "Installing core guest dependencies..."
pct exec "$CT_ID" -- bash -c "${PCT_EXPORTS} apt-get update -y >/dev/null"
pct exec "$CT_ID" -- bash -c "${PCT_EXPORTS} apt-get install -y curl gnupg lsb-release ca-certificates git >/dev/null"

info "Installing Docker Engine inside guest container..."
pct exec "$CT_ID" -- bash -c "${PCT_EXPORTS} curl -fsSL https://get.docker.com | sh" >/dev/null
# Configure Docker default DNS to bypass Proxmox LXC nested network/DNS resolution bottlenecks
pct exec "$CT_ID" -- bash -c "${PCT_EXPORTS} mkdir -p /etc/docker && echo '{\"dns\": [\"1.1.1.1\", \"8.8.8.8\"]}' > /etc/docker/daemon.json"
pct exec "$CT_ID" -- systemctl restart docker >/dev/null || true
pct exec "$CT_ID" -- systemctl enable --now docker >/dev/null

# 6. Persistent Docker Volume Orchestration
info "Creating named volumes for data persistence..."
pct exec "$CT_ID" -- bash -c "${PCT_EXPORTS} docker volume create portainer_data >/dev/null"

# 7. Deploy Portainer Community Edition
info "Deploying Portainer CE on port 9000..."
pct exec "$CT_ID" -- bash -c "${PCT_EXPORTS} docker run -d \
    --name portainer \
    --restart always \
    -p 9000:9000 \
    -p 9443:9443 \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -v portainer_data:/data \
  portainer/portainer-ce:latest >/dev/null"

# 8. Clone AMUD Workspace and Generate Lightweight Go Compose
info "Cloning AMUD-Dashboard repository for local compilation..."
pct exec "$CT_ID" -- bash -c "${PCT_EXPORTS} git clone https://github.com/boubli/AMUD-Dahsboard.git /opt/amud >/dev/null"

info "Writing local-build Docker Compose file to /opt/amud/..."
pct exec "$CT_ID" -- bash -c "${PCT_EXPORTS} cat << 'EOF' > /opt/amud/docker-compose.yml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: amud_app
    restart: always
    ports:
      - \"80:8000\"
    environment:
      - DB_PATH=/app/data/amud.db
      - PORT=8000
    volumes:
      - ./data:/app/data
EOF"

# 9. Autostart & Local Compile Stack
info "Compiling and starting AMUD Go/HTMX environment (this may take a few minutes)..."
pct exec "$CT_ID" -- bash -c "${PCT_EXPORTS} cd /opt/amud && docker compose up --build -d" >/dev/null

# 9.5 Downscale container resources to production boundaries
info "Restricting container resources to runtime limits (256MB RAM)..."
pct set "$CT_ID" -memory 256 -swap 256

# 10. Extract Guest IP & Output Completion Diagnostics
CT_IP_ADDR=$(pct exec "$CT_ID" -- hostname -I | awk '{print $1}')

# Write dynamic login MOTD inside the guest LXC container
TEMPLATE_MOTD=$(cat << 'EOF'
  ______   __       __  __    __  _______  
 /      \ |  \     /  \|  \  |  \|       \ 
|  $$$$$$\| $$\   /  $$| $$  | $$| $$$$$$$\
| $$__| $$| $$$\ /  $$$| $$  | $$| $$  | $$
| $$    $$| $$$$\  $$$$| $$  | $$| $$  | $$
| $$$$$$$$| $$\$$ $$ $$| $$  | $$| $$  | $$
| $$  | $$| $$ \$$$| $$| $$__/ $$| $$__/ $$
| $$  | $$| $$  \$ | $$ \$$    $$| $$    $$
 \$$   \$$ \$$      \$$  \$$$$$$  \$$$$$$$ 
==============================================================
AMUD-Dahsboard (LXC OS: Debian 12)
==============================================================
  Local IP Address: http://__IP__
  Access UI / API:   http://__IP__:80 (Port 80)
  Portainer UI:      http://__IP__:9000 (Port 9000)
==============================================================
EOF
)
echo "${TEMPLATE_MOTD//__IP__/$CT_IP_ADDR}" > /tmp/amud-motd
pct push "$CT_ID" /tmp/amud-motd /etc/motd
rm -f /tmp/amud-motd

success "Ecosystem deployment successfully finalized on Autopilot!"
echo "=============================================================="
echo "AMUD LXC Deployment Diagnostics Summary"
echo "=============================================================="
echo "  Container ID:      $CT_ID"
echo "  Container Hostname: $CT_NAME"
echo "  Container Local IP: $CT_IP_ADDR"
echo "  RAM / Swap Alloc:  256MB / 256MB"
echo "--------------------------------------------------------------"
echo "  AMUD UI & API:     http://${CT_IP_ADDR}"
echo "  Portainer Panel:   http://${CT_IP_ADDR}:9000"
echo "=============================================================="
