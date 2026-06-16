#!/usr/bin/env bash
# ==============================================================================
# uninstall-amud.sh
# 
# Autopilot uninstaller script for the AMUD ecosystem.
# Run this on the Proxmox VE host shell as root to completely clean up.
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
  AMUD Dashboard Autopilot Release Uninstaller
===============================================
EOF
}

# Display Header
header_info

# 1. Stop and Destroy the LXC Container
CT_ID=$(pct list 2>/dev/null | awk '$3 == "amud-dashboard" || $3 == "hydrivax-amud" {print $1}' | head -n1 || true)

if [ -n "$CT_ID" ]; then
    CT_NAME=$(pct list 2>/dev/null | awk -v id="$CT_ID" '$1 == id {print $3}' || echo "amud-dashboard")
    echo -e "  ${INFO}  Found AMUD LXC container (ID: $CT_ID, Name: $CT_NAME). Stopping and destroying..."
    
    msg_info "Stopping LXC container $CT_ID"
    pct stop "$CT_ID" >/dev/null 2>&1 || true
    msg_ok "Stopped LXC container $CT_ID"
    
    msg_info "Destroying LXC container $CT_ID"
    pct destroy "$CT_ID" >/dev/null 2>&1
    msg_ok "LXC Container $CT_ID ($CT_NAME) destroyed successfully"
else
    echo -e "  ${INFO}  No LXC container named 'amud-dashboard' or 'hydrivax-amud' found."
fi

# 2. Stop and Disable Host systemd Agent Service
if systemctl list-unit-files | grep -q "amud-agent.service"; then
    echo -e "\n  ${INFO}  Uninstalling amud-agent service on Proxmox host..."
    
    msg_info "Stopping and disabling service"
    systemctl stop amud-agent >/dev/null 2>&1 || true
    systemctl disable amud-agent >/dev/null 2>&1 || true
    msg_ok "Stopped and disabled service"
    
    msg_info "Removing systemd service file"
    rm -f /etc/systemd/system/amud-agent.service
    systemctl daemon-reload
    msg_ok "Systemd service files removed"
fi

# 3. Clean up host files and sockets
echo -e "\n  ${INFO}  Cleaning up host files..."

msg_info "Removing binaries, temporary directories, and sockets"
rm -f /usr/local/bin/amud-agent
rm -rf /opt/amud/run /var/run/amud
msg_ok "All host files and sockets cleaned up"

echo -e "\n=============================================================="
echo -e "  ${CM}  ${BGN}AMUD Dashboard has been completely uninstalled from host!${CL}"
echo -e "=============================================================="
echo
