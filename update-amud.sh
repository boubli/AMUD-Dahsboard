#!/usr/bin/env bash
# ==============================================================================
# update-amud.sh
# 
# Autopilot updater script for the AMUD ecosystem.
# Run this on the Proxmox VE host shell as root to update to the latest release.
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
    AMUD Dashboard Autopilot Release Updater
===============================================
EOF
}

# Display Header
header_info

msg_info "Querying latest release from GitHub API"
REPO="boubli/AMUD-Dashboard"
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

if [ -z "$LATEST_RELEASE" ]; then
    msg_error "Could not fetch the latest release version from GitHub API"
    echo -e "  Please check your internet connection or GitHub rate limits." >&2
    exit 1
fi
msg_ok "Latest available release version: $LATEST_RELEASE"

# 1. Update AMUD Dashboard Server inside the LXC container
CT_ID=$(pct list 2>/dev/null | awk '$3 == "amud-dashboard" {print $1}' | head -n1 || true)

if [ -n "$CT_ID" ]; then
    echo -e "\n  ${INFO}  Updating AMUD Dashboard Server inside LXC container $CT_ID..."
    
    msg_info "Stopping server service"
    pct exec "$CT_ID" -- systemctl stop amud >/dev/null 2>&1 || true
    msg_ok "Stopped server service"
    
    msg_info "Downloading server release binary"
    pct exec "$CT_ID" -- curl -L -s -o /opt/amud/amud-server "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-server"
    pct exec "$CT_ID" -- chmod +x /opt/amud/amud-server
    msg_ok "Server release binary downloaded"
    
    msg_info "Downloading and updating UI templates/assets"
    pct exec "$CT_ID" -- curl -L -s -o /tmp/ui.tar.gz "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/ui.tar.gz"
    pct exec "$CT_ID" -- tar -xzf /tmp/ui.tar.gz -C /opt/amud/
    pct exec "$CT_ID" -- rm -f /tmp/ui.tar.gz
    msg_ok "UI templates/assets updated"
    
    msg_info "Restarting server service"
    pct exec "$CT_ID" -- systemctl start amud
    msg_ok "Restarted server service"
    
    echo -e "  ${CM}  Dashboard Server inside LXC container $CT_ID updated to $LATEST_RELEASE"
else
    echo -e "  ${WARNING}  AMUD LXC container (amud-dashboard) not found. Skipping server update."
fi

# 2. Update Host Telemetry Agent on Proxmox Host
if [ -f "/usr/local/bin/amud-agent" ]; then
    echo -e "\n  ${INFO}  Updating amud-agent on Proxmox host..."
    
    msg_info "Stopping amud-agent service"
    systemctl stop amud-agent || true
    msg_ok "Stopped amud-agent service"
    
    msg_info "Downloading host agent release binary"
    curl -L -s -o /usr/local/bin/amud-agent "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-agent"
    chmod +x /usr/local/bin/amud-agent
    msg_ok "Host agent release binary downloaded"
    
    msg_info "Restarting amud-agent service"
    systemctl start amud-agent
    msg_ok "Restarted amud-agent service"
    
    echo -e "  ${CM}  Host telemetry agent updated to $LATEST_RELEASE"
else
    echo -e "\n  ${WARNING}  amud-agent binary not found at /usr/local/bin/amud-agent. Skipping host agent update."
fi

echo -e "\n=============================================================="
echo -e "  ${CM}  ${BGN}AMUD ecosystem update completed successfully!${CL}"
echo -e "=============================================================="
echo
