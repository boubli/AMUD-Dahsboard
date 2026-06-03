#!/usr/bin/env bash
# ==============================================================================
# setup-amud.sh (Unattended Release Installer)
# 
# Autopilot installation script for the AMUD launcher dashboard ecosystem.
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
# Fetch latest Debian 12 template
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
    AMUD Dashboard Autopilot Release Installer
===============================================
EOF

info "Executing unattended AMUD Release Deployment..."
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

# 4. Container Resource Allocation (Native systemd, extremely lightweight)
info "Creating container $CT_ID ($CT_NAME)..."
pct create "$CT_ID" "local:vztmpl/$TEMPLATE_FILE" \
    -cores 1 \
    -memory 256 \
    -swap 256 \
    -hostname "$CT_NAME" \
    -ostype debian \
    -storage local-lvm \
    -rootfs local-lvm:4 \
    -net0 "name=eth0,bridge=vmbr0,ip=$CT_IP" \
    -unprivileged 1 \
    -features nesting=1,keyctl=1 \
    -nameserver "1.1.1.1 8.8.8.8" >/dev/null

info "Creating host socket directory and configuring bind-mount..."
mkdir -p /opt/amud/run
chmod 777 /opt/amud/run
# Append bind mount mapping to LXC config
echo "mp0: /opt/amud/run,mp=/opt/amud/run" >> "/etc/pve/lxc/${CT_ID}.conf"

info "Starting container $CT_ID..."
pct start "$CT_ID" >/dev/null

info "Waiting for container to boot and get network address..."
sleep 12

# 5. Dependency Toolchain
info "Installing core guest dependencies..."
pct exec "$CT_ID" -- bash -c "export DEBIAN_FRONTEND=noninteractive; apt-get update -y && apt-get install -y curl tar ca-certificates >/dev/null"

# 6. Fetch Latest Release Version
info "Querying latest release from GitHub API..."
REPO="boubli/AMUD-Dahsboard"
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

if [ -z "$LATEST_RELEASE" ]; then
    LATEST_RELEASE="v1.0.0"
fi
info "Targeting release: $LATEST_RELEASE"

# 7. Create Directories and Download Server inside Guest
info "Provisioning server binary and assets inside LXC guest..."
pct exec "$CT_ID" -- mkdir -p /opt/amud/data
pct exec "$CT_ID" -- curl -L -s -o /opt/amud/amud-server "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-server"
pct exec "$CT_ID" -- chmod +x /opt/amud/amud-server

pct exec "$CT_ID" -- curl -L -s -o /tmp/ui.tar.gz "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/ui.tar.gz"
pct exec "$CT_ID" -- tar -xzf /tmp/ui.tar.gz -C /opt/amud/
pct exec "$CT_ID" -- rm -f /tmp/ui.tar.gz

# 8. Configure Systemd Service inside Guest
info "Configuring amud systemd daemon inside LXC guest..."
pct exec "$CT_ID" -- bash -c "cat << 'EOF' > /etc/systemd/system/amud.service
[Unit]
Description=AMUD Dashboard Server
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/amud
ExecStart=/opt/amud/amud-server
Restart=always
RestartSec=5
Environment=PORT=8000
Environment=DB_PATH=/opt/amud/data/amud.db
Environment=AMUD_SOCKET_PATH=/opt/amud/run/amud.sock

[Install]
WantedBy=multi-user.target
EOF"

pct exec "$CT_ID" -- systemctl daemon-reload
pct exec "$CT_ID" -- systemctl enable --now amud

# 9. Download and Install Host Telemetry Agent on Proxmox Host
info "Installing amud-agent on Proxmox host..."
curl -L -s -o /usr/local/bin/amud-agent "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-agent"
chmod +x /usr/local/bin/amud-agent

info "Installing amud-agent systemd service on Proxmox host..."
cat << 'EOF' > /etc/systemd/system/amud-agent.service
[Unit]
Description=AMUD Host Telemetry Agent
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/amud-agent
Restart=always
RestartSec=5
Environment=AMUD_SOCKET_PATH=/opt/amud/run/amud.sock

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable --now amud-agent
success "Host telemetry agent is running and streaming metrics!"

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
AMUD-Dahsboard (LXC OS: Debian 12 - Native Service)
==============================================================
  Local IP Address: http://__IP__
  Access UI / API:   http://__IP__:8000 (Port 8000)
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
echo "  AMUD UI & API:     http://${CT_IP_ADDR}:8000"
echo "=============================================================="
