#!/usr/bin/env bash
# ==============================================================================
# uninstall-amud.sh
# 
# Autopilot uninstaller script for the AMUD ecosystem.
# Run this on the Proxmox VE host shell as root to completely clean up.
# ==============================================================================

set -euo pipefail

# Print styled messages
info() {
    echo -e "\033[1;34m[INFO]\033[0m $1"
}

success() {
    echo -e "\033[1;32m[SUCCESS]\033[0m $1"
}

echo "=============================================================="
echo "      AMUD Ecosystem Autopilot Uninstaller"
echo "=============================================================="

# 1. Stop and Destroy the LXC Container
CT_ID=$(pct list 2>/dev/null | awk '$3 == "amud-dashboard" {print $1}' | head -n1)

if [ -n "$CT_ID" ]; then
    info "Found AMUD LXC container (ID: $CT_ID). Stopping and destroying..."
    pct stop "$CT_ID" >/dev/null 2>&1 || true
    pct destroy "$CT_ID" >/dev/null 2>&1
    success "LXC Container $CT_ID (amud-dashboard) destroyed successfully."
else
    info "No LXC container named 'amud-dashboard' found."
fi

# 2. Stop and Disable Host systemd Agent Service
if systemctl list-unit-files | grep -q "amud-agent.service"; then
    info "Stopping and disabling amud-agent service on Proxmox host..."
    systemctl stop amud-agent >/dev/null 2>&1 || true
    systemctl disable amud-agent >/dev/null 2>&1 || true
    rm -f /etc/systemd/system/amud-agent.service
    systemctl daemon-reload
    success "Host agent systemd service removed."
fi

# 3. Clean up host files and sockets
info "Cleaning up host binaries, files, and sockets..."
rm -f /usr/local/bin/amud-agent
rm -rf /opt/amud/run /var/run/amud

success "AMUD Dashboard ecosystem has been completely uninstalled from your Proxmox host!"
echo "=============================================================="
