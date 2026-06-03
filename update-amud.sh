#!/usr/bin/env bash
# ==============================================================================
# update-amud.sh
# 
# Autopilot updater script for the AMUD ecosystem.
# Run this on the Proxmox VE host shell as root to update to the latest release.
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

REPO="boubli/AMUD-Dahsboard"
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

if [ -z "$LATEST_RELEASE" ]; then
    error "Could not fetch the latest release version from GitHub API. Please check your internet connection or GitHub rate limits."
    exit 1
fi

echo "=============================================================="
echo "      AMUD Ecosystem Autopilot Updater"
echo "=============================================================="
info "Latest available release version: $LATEST_RELEASE"

# 1. Update AMUD Dashboard Server inside the LXC container
CT_ID=$(pct list 2>/dev/null | awk '$3 == "amud-dashboard" {print $1}' | head -n1 || true)

if [ -n "$CT_ID" ]; then
    info "Updating AMUD Dashboard Server inside LXC container $CT_ID..."
    
    # Stop server systemd service
    pct exec "$CT_ID" -- systemctl stop amud >/dev/null 2>&1 || true
    
    # Download latest server release binary
    info "Downloading server release binary..."
    pct exec "$CT_ID" -- curl -L -s -o /opt/amud/amud-server "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-server"
    pct exec "$CT_ID" -- chmod +x /opt/amud/amud-server
    
    # Download and extract updated UI templates and assets
    info "Downloading and updating UI templates/assets..."
    pct exec "$CT_ID" -- curl -L -s -o /tmp/ui.tar.gz "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/ui.tar.gz"
    pct exec "$CT_ID" -- tar -xzf /tmp/ui.tar.gz -C /opt/amud/
    pct exec "$CT_ID" -- rm -f /tmp/ui.tar.gz
    
    # Restart server systemd service
    pct exec "$CT_ID" -- systemctl start amud
    success "Dashboard Server inside LXC container $CT_ID updated successfully to $LATEST_RELEASE."
else
    info "AMUD LXC container (amud-dashboard) not found. Skipping server update."
fi

# 2. Update Host Telemetry Agent on Proxmox Host
if [ -f "/usr/local/bin/amud-agent" ]; then
    info "Updating amud-agent on Proxmox host..."
    systemctl stop amud-agent || true
    
    info "Downloading host agent release binary..."
    curl -L -s -o /usr/local/bin/amud-agent "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-agent"
    chmod +x /usr/local/bin/amud-agent
    
    systemctl start amud-agent
    success "Host telemetry agent updated successfully to $LATEST_RELEASE."
else
    info "amud-agent binary not found at /usr/local/bin/amud-agent. Skipping host agent update."
fi

success "AMUD ecosystem update completed successfully!"
echo "=============================================================="
