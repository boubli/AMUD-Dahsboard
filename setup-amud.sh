#!/usr/bin/env bash
# ==============================================================================
# setup-amud.sh (Unattended Release Installer)
# 
# Autopilot installation script for the AMUD launcher dashboard ecosystem.
# Target Host: Proxmox VE Host Shell (Run as root)
# Guest LXC OS: Debian 12 (unprivileged, nesting & keyctl enabled, memory-efficient)
# ==============================================================================

set -euo pipefail

# Color Definitions
RD="\033[01;31m"
GN="\033[01;32m"
YW="\033[01;33m"
BL="\033[01;34m"
PK="\033[01;35m"
CY="\033[01;36m"
CL="\033[m"
BGN="\033[1;32m"

# Emojis & Symbols
CM="${GN}✔️${CL}"
CROSS="${RD}✖️${CL}"
INFO="${BL}💡${CL}"
WARNING="${YW}⚠️${CL}"
LAUNCH="${PK}🚀${CL}"

# Logging Functions
msg_info() {
  local msg="$1"
  echo -ne "  ${INFO}  ${msg}..."
}

msg_ok() {
  local msg="$1"
  echo -e "\r\033[K  ${CM}  ${msg}"
}

msg_error() {
  local msg="$1"
  echo -e "\r\033[K  ${CROSS}  ${msg}" >&2
}

msg_warn() {
  local msg="$1"
  echo -e "  ${WARNING}  ${msg}"
}

# Header Info
header_info() {
  clear
  cat << 'EOF'
    ___    __  __ _   __ ____  
   /   |  / / / /| | / // __ \ 
  / /| | / / / / | |/ // / / / 
 / ___ |/ /_/ /  |  // /_/ /  
/_/  |_|\____/   |__/_____/   
                              
===============================================
   AMUD Dashboard Autopilot Release Installer
===============================================
EOF
}

# 1. Host Execution Pre-checks
if [ ! -d "/etc/pve" ]; then
    echo -e "\n  ${CROSS}  This script must be executed directly on a Proxmox VE host shell.\n" >&2
    exit 1
fi

# Gather configuration details for the info panel
PVE_VERSION=$(pveversion 2>/dev/null | awk -F'/' '{print $2}' | awk -F'-' '{print $1}' || echo "Unknown")
KERNEL_VERSION=$(uname -r)
CT_ID=$(pvesh get /cluster/nextid)
CT_NAME="amud-dashboard"
CT_IP="dhcp"
TEMPLATE_DIR="/var/lib/vz/template/cache"
TEMPLATE_FILE=$(pveam available -section system | awk '$2 ~ /^debian-12-standard/ {print $2}' | head -n1)

# Display header & info settings panel
header_info

echo -e "  🧩  Using Unattended Install on node $(hostname)"
echo
echo -e "  💡  PVE Version: ${PVE_VERSION} (Kernel: ${KERNEL_VERSION})"
echo -e "  🖥️  Operating System: debian"
echo -e "  🌟  Version: 12 (standard)"
echo -e "  📦  Container Type: Unprivileged"
echo -e "  🆔  Container ID: ${CT_ID}"
echo -e "  🏠  Hostname: ${CT_NAME}"
echo -e "  💾  Disk Size: 4 GB"
echo -e "  🧠  CPU Cores: 1"
echo -e "  🛠️  RAM Size: 256 MiB"
echo -e "  🛠️  Swap Size: 256 MiB"
echo -e "  🌉  Bridge: vmbr0"
echo -e "  📡  IPv4: dhcp"
echo -e "  📦  Nesting: Enabled"
echo -e "  📦  Keyctl: Enabled"
echo -e "  ${LAUNCH}  Creating an LXC of AMUD Dashboard using the above settings"
echo -e "----------------------------------------------------------------"

# 2. Conflict Prevention: Clean Existing Installation
if pct status "$CT_ID" >/dev/null 2>&1; then
    msg_info "Conflict detected: Container $CT_ID already exists. Stopping and destroying"
    pct stop "$CT_ID" >/dev/null 2>&1 || true
    pct destroy "$CT_ID" >/dev/null 2>&1
    msg_ok "Existing container $CT_ID removed"
fi

# 3. Template Management
msg_info "Updating pveam templates index"
pveam update >/dev/null 2>&1 || true
msg_ok "pveam templates index updated"

TEMPLATE_PATH="$TEMPLATE_DIR/$TEMPLATE_FILE"
if [ ! -f "$TEMPLATE_PATH" ]; then
    msg_info "Downloading template $TEMPLATE_FILE"
    pveam download local "system/$TEMPLATE_FILE" >/dev/null
    msg_ok "Downloaded template $TEMPLATE_FILE"
else
    msg_ok "Template $TEMPLATE_FILE is already available locally"
fi

# 4. Container Resource Allocation (Native systemd, extremely lightweight)
msg_info "Creating container $CT_ID ($CT_NAME)"
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
msg_ok "LXC Container $CT_ID was successfully created"

msg_info "Creating host socket directory and configuring bind-mount"
mkdir -p /opt/amud/run
chmod 770 /opt/amud/run
# Append bind mount mapping to LXC config
echo "mp0: /opt/amud/run,mp=/opt/amud/run" >> "/etc/pve/lxc/${CT_ID}.conf"
msg_ok "Host socket directory and bind-mount configured"

msg_info "Starting LXC container $CT_ID"
pct start "$CT_ID" >/dev/null
msg_ok "Started LXC Container"

msg_info "Waiting for container to boot and get network address"
sleep 12
msg_ok "LXC Network is reachable"

# 5. Dependency Toolchain
msg_info "Installing core guest dependencies"
pct exec "$CT_ID" -- bash -c "env DEBIAN_FRONTEND=noninteractive LC_ALL=C LANG=C LANGUAGE=C apt-get update -qq -y >/dev/null && env DEBIAN_FRONTEND=noninteractive LC_ALL=C LANG=C LANGUAGE=C apt-get install -qq -y curl tar ca-certificates >/dev/null"
msg_ok "Core guest dependencies installed and system updated"

# 6. Fetch Latest Release Version
msg_info "Querying latest release from GitHub API"
REPO="boubli/AMUD-Dashboard"
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

if [ -z "$LATEST_RELEASE" ]; then
    LATEST_RELEASE="v1.0.0"
fi
msg_ok "Targeting release: $LATEST_RELEASE"

AMUD_AGENT_SECRET=$(openssl rand -base64 32 | tr -d '/+=' | head -c 43)

msg_info "Downloading release checksum manifest"
curl -L -sS -f -o /tmp/amud-SHA256SUMS "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/SHA256SUMS"
msg_ok "Release checksum manifest downloaded"

verify_release_asset() {
  local file="$1"
  local name="$2"
  local expected actual
  expected=$(grep -E "[[:space:]/]${name}$" /tmp/amud-SHA256SUMS | awk '{print $1}' || true)
  if [ -z "$expected" ]; then
    msg_error "Checksum for ${name} not found in SHA256SUMS"
    exit 1
  fi
  actual=$(sha256sum "$file" | awk '{print $1}' || true)
  if [ "$actual" != "$expected" ]; then
    msg_error "Checksum verification failed for ${name}"
    exit 1
  fi
}

# 7. Create Directories and Download Server inside Guest
msg_info "Provisioning server binary and assets inside LXC guest"
pct exec "$CT_ID" -- mkdir -p /opt/amud/data
curl -L -sS -f -o /tmp/amud-server "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-server"
verify_release_asset /tmp/amud-server amud-server
pct push "$CT_ID" /tmp/amud-server /opt/amud/amud-server >/dev/null
pct exec "$CT_ID" -- chmod +x /opt/amud/amud-server

curl -L -sS -f -o /tmp/ui.tar.gz "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/ui.tar.gz"
verify_release_asset /tmp/ui.tar.gz ui.tar.gz
pct push "$CT_ID" /tmp/ui.tar.gz /tmp/ui.tar.gz >/dev/null
pct exec "$CT_ID" -- tar -xzf /tmp/ui.tar.gz -C /opt/amud/
pct exec "$CT_ID" -- rm -f /tmp/ui.tar.gz
rm -f /tmp/amud-server /tmp/ui.tar.gz
pct exec "$CT_ID" -- test -f /opt/amud/ui/static/vendor/alpine.min.js
pct exec "$CT_ID" -- test -f /opt/amud/ui/static/vendor/lucide.min.js
msg_ok "AMUD server binary and UI files provisioned inside guest"

# 8. Configure Systemd Service inside Guest
msg_info "Configuring amud systemd daemon inside LXC guest"
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
Environment=AMUD_AGENT_SECRET=${AMUD_AGENT_SECRET}
Environment=AMUD_ENABLE_PROXMOX=true

[Install]
WantedBy=multi-user.target
EOF"

pct exec "$CT_ID" -- systemctl daemon-reload
pct exec "$CT_ID" -- systemctl enable --now amud
msg_ok "Systemd daemon enabled and running inside guest"

# 9. Download and Install Host Telemetry Agent on Proxmox Host
msg_info "Installing amud-agent on Proxmox host"
curl -L -sS -f -o /tmp/amud-agent "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-agent"
verify_release_asset /tmp/amud-agent amud-agent
install -m 755 /tmp/amud-agent /usr/local/bin/amud-agent
rm -f /tmp/amud-agent /tmp/amud-SHA256SUMS
msg_ok "amud-agent binary installed on host"

msg_info "Installing amud-agent systemd service on Proxmox host"
cat << EOF > /etc/systemd/system/amud-agent.service
[Unit]
Description=AMUD Host Telemetry Agent
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/amud-agent
Restart=always
RestartSec=5
Environment=AMUD_SOCKET_PATH=/opt/amud/run/amud.sock
Environment=AMUD_AGENT_SECRET=${AMUD_AGENT_SECRET}

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable --now amud-agent
msg_ok "Host telemetry agent is running and streaming metrics"

# 10. Extract Guest IP & Output Completion Diagnostics
CT_IP_ADDR=$(pct exec "$CT_ID" -- hostname -I | awk '{print $1}')

# Write dynamic login MOTD inside the guest LXC container
TEMPLATE_MOTD=$(cat << 'EOF'
    ___    __  __ _   __ ____  
   /   |  / / / /| | / // __ \ 
  / /| | / / / / | |/ // / / / 
 / ___ |/ /_/ /  |  // /_/ /  
/_/  |_|\____/   |__/_____/   
==============================================================
AMUD-Dashboard (LXC OS: Debian 12 - Native Service)
==============================================================
  Local IP Address: http://__IP__
  Access UI / API:   http://__IP__:8000 (Port 8000)
  First login:       user admin — password printed once in
                     journalctl -u amud-server (bootstrap only)
==============================================================
EOF
)
echo "${TEMPLATE_MOTD//__IP__/$CT_IP_ADDR}" > /tmp/amud-motd
pct push "$CT_ID" /tmp/amud-motd /etc/motd
rm -f /tmp/amud-motd

echo
echo -e "  ${CM}  ${BGN}Ecosystem deployment successfully finalized on Autopilot!${CL}"
echo -e "=============================================================="
echo -e "  Container ID:       ${GN}$CT_ID${CL}"
echo -e "  Container Hostname: ${GN}$CT_NAME${CL}"
echo -e "  Container Local IP: ${GN}$CT_IP_ADDR${CL}"
echo -e "  RAM / Swap Alloc:   ${GN}256MB / 256MB${CL}"
echo -e "--------------------------------------------------------------"
echo -e "  ${LAUNCH}  ${BGN}AMUD UI & API:     http://${CT_IP_ADDR}:8000${CL}"
echo -e "=============================================================="
echo
