#!/usr/bin/env bash
# ==============================================================================
# update-amud.sh
#
# Autopilot updater for the AMUD ecosystem on Proxmox VE.
# Run on the Proxmox host as root.
# ==============================================================================

set -euo pipefail

REPO="boubli/AMUD-Dashboard"
CT_ID=""
SERVER_WAS_RUNNING=false
AGENT_WAS_RUNNING=false

# Color Definitions
RD="\033[01;31m"
GN="\033[01;32m"
YW="\033[01;33m"
BL="\033[01;34m"
CL="\033[m"
BGN="\033[1;32m"

CM="${GN}✔️${CL}"
CROSS="${RD}✖️${CL}"
INFO="${BL}💡${CL}"
WARNING="${YW}⚠️${CL}"

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

failure_trap() {
  local exit_code=$?
  local line_no=$1
  echo -e "\n" >&2
  msg_error "Update failed on line ${line_no} (exit ${exit_code}). Attempting to restore services..."
  if [ -n "$CT_ID" ] && [ "$SERVER_WAS_RUNNING" = true ]; then
    pct exec "$CT_ID" -- systemctl start amud >/dev/null 2>&1 || true
  fi
  if [ "$AGENT_WAS_RUNNING" = true ]; then
    systemctl start amud-agent >/dev/null 2>&1 || true
  fi
  exit "$exit_code"
}
trap 'failure_trap $LINENO' ERR

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

download_file() {
  local url="$1"
  local dest="$2"
  local description="$3"
  if ! curl -L -sS -f --connect-timeout 15 --retry 3 -o "$dest" "$url"; then
    echo -e "\n" >&2
    msg_error "Failed to download ${description}"
    echo -e "  URL: ${url}" >&2
    exit 1
  fi
}

verify_release_asset() {
  local file="$1"
  local name="$2"
  local expected actual
  expected=$(grep -E "[[:space:]/]${name}$" /tmp/AMUD-SHA256SUMS | awk '{print $1}' || true)
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

pct_push_file() {
  local ct_id="$1"
  local src="$2"
  local dest="$3"
  if ! pct push "$ct_id" "$src" "$dest" >/dev/null; then
    msg_error "Failed to push file to container ${ct_id} (${src} -> ${dest})"
    exit 1
  fi
}

pct_exec_cmd() {
  local ct_id="$1"
  local cmd="$2"
  if ! pct exec "$ct_id" -- bash -c "$cmd"; then
    msg_error "Command failed inside container ${ct_id}: ${cmd}"
    exit 1
  fi
}

generate_agent_secret() {
  openssl rand -base64 32 | tr -d '/+=' | head -c 43
}

read_local_systemd_env() {
  local file="$1"
  local key="$2"
  grep -E "^Environment=${key}=" "$file" 2>/dev/null | head -n1 | sed -E "s/^Environment=${key}=//" || true
}

ensure_local_systemd_env() {
  local file="$1"
  local key="$2"
  local value="$3"
  if grep -q "^Environment=${key}=" "$file"; then
    sed -i "s|^Environment=${key}=.*|Environment=${key}=${value}|" "$file"
  else
    sed -i "/^\[Service\]/a Environment=${key}=${value}" "$file"
  fi
}

read_container_agent_secret() {
  local ct_id="$1"
  local secret
  secret=$(pct exec "$ct_id" -- bash -c 'grep -E "^Environment=AMUD_AGENT_SECRET=" /etc/systemd/system/amud.service 2>/dev/null | head -n1 | sed "s/^Environment=AMUD_AGENT_SECRET=//"' 2>/dev/null || true)
  if [ -n "$secret" ]; then
    echo "$secret"
    return
  fi
  secret=$(pct exec "$ct_id" -- bash -c 'command -v sqlite3 >/dev/null 2>&1 || apt-get install -qq -y sqlite3 >/dev/null 2>&1; sqlite3 /opt/amud/data/amud.db "SELECT value FROM settings WHERE key='"'"'agent_shared_secret'"'"';" 2>/dev/null' 2>/dev/null || true)
  echo "$secret"
}

ensure_container_systemd_env() {
  local ct_id="$1"
  local file="$2"
  local key="$3"
  local value="$4"
  pct exec "$ct_id" -- bash -c "
    if grep -q '^Environment=${key}=' '${file}'; then
      sed -i 's|^Environment=${key}=.*|Environment=${key}=${value}|' '${file}'
    else
      sed -i '/^\[Service\]/a Environment=${key}=${value}' '${file}'
    fi
  "
}

sync_agent_ipc_secret() {
  local ct_id="$1"
  local secret="$2"
  msg_info "Synchronizing AMUD_AGENT_SECRET between server and host agent"
  ensure_container_systemd_env "$ct_id" "/etc/systemd/system/amud.service" "AMUD_AGENT_SECRET" "$secret"
  ensure_container_systemd_env "$ct_id" "/etc/systemd/system/amud.service" "AMUD_ENABLE_PROXMOX" "true"
  pct exec "$ct_id" -- systemctl daemon-reload
  if [ -f "/etc/systemd/system/amud-agent.service" ]; then
    ensure_local_systemd_env "/etc/systemd/system/amud-agent.service" "AMUD_AGENT_SECRET" "$secret"
    systemctl daemon-reload
  fi
  msg_ok "Agent IPC secret synchronized"
}

verify_ui_assets() {
  local ct_id="$1"
  pct exec "$ct_id" -- test -f /opt/amud/ui/static/vendor/alpine.min.js
  pct exec "$ct_id" -- test -f /opt/amud/ui/static/vendor/lucide.min.js
  pct exec "$ct_id" -- test -f /opt/amud/ui/static/style.css
}

verify_agent_connection() {
  local ct_id="$1"
  sleep 3
  if pct exec "$ct_id" -- journalctl -u amud --no-pager --since "2 minutes ago" 2>/dev/null | grep -q "invalid IPC authentication"; then
    msg_warn "Agent IPC authentication still failing. Check AMUD_AGENT_SECRET on server and host agent."
    return 1
  fi
  msg_ok "Agent IPC authentication looks healthy"
}

header_info

msg_info "Querying latest release from GitHub API"
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)
if [ -z "$LATEST_RELEASE" ]; then
  msg_error "Could not fetch the latest release version from GitHub API"
  exit 1
fi
msg_ok "Latest available release version: $LATEST_RELEASE"

msg_info "Fetching release checksums (SHA256SUMS)"
download_file "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/SHA256SUMS" "/tmp/AMUD-SHA256SUMS" "release checksums (SHA256SUMS)"
msg_ok "SHA256SUMS downloaded"

msg_info "Downloading release assets to Proxmox host staging area"
download_file "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-server" "/tmp/amud-server" "server release binary"
verify_release_asset /tmp/amud-server amud-server
download_file "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/ui.tar.gz" "/tmp/ui.tar.gz" "UI templates/assets"
verify_release_asset /tmp/ui.tar.gz ui.tar.gz
if [ -f "/usr/local/bin/amud-agent" ]; then
  download_file "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-agent" "/tmp/amud-agent" "host agent release binary"
  verify_release_asset /tmp/amud-agent amud-agent
fi
msg_ok "Release assets downloaded and verified"

CT_ID=$(pct list 2>/dev/null | awk '$3 == "amud-dashboard" {print $1}' | head -n1 || true)

if [ -n "$CT_ID" ]; then
  echo -e "\n  ${INFO}  Updating AMUD Dashboard Server inside LXC container $CT_ID..."

  if pct exec "$CT_ID" -- systemctl is-active --quiet amud 2>/dev/null; then
    SERVER_WAS_RUNNING=true
  fi

  msg_info "Resolving agent IPC shared secret"
  AMUD_AGENT_SECRET=$(read_container_agent_secret "$CT_ID")
  if [ -z "$AMUD_AGENT_SECRET" ]; then
    AMUD_AGENT_SECRET=$(generate_agent_secret)
    msg_warn "No existing agent IPC secret found. Generated a new shared secret."
  fi
  sync_agent_ipc_secret "$CT_ID" "$AMUD_AGENT_SECRET"

  msg_info "Stopping server service for in-place upgrade"
  pct exec "$CT_ID" -- systemctl stop amud >/dev/null 2>&1 || true
  msg_ok "Stopped server service"

  msg_info "Installing server release binary"
  pct_push_file "$CT_ID" /tmp/amud-server /opt/amud/amud-server
  pct_exec_cmd "$CT_ID" "chmod +x /opt/amud/amud-server"
  msg_ok "Server release binary installed"

  msg_info "Installing UI templates/assets"
  pct_push_file "$CT_ID" /tmp/ui.tar.gz /tmp/ui.tar.gz
  pct_exec_cmd "$CT_ID" "tar -xzf /tmp/ui.tar.gz -C /opt/amud/"
  pct_exec_cmd "$CT_ID" "rm -f /tmp/ui.tar.gz"
  msg_ok "UI templates/assets updated"

  msg_info "Verifying required UI static assets"
  if verify_ui_assets "$CT_ID"; then
    msg_ok "UI static assets verified"
  else
    msg_error "Required UI assets missing under /opt/amud/ui/static/"
    exit 1
  fi

  msg_info "Restarting server service"
  pct_exec_cmd "$CT_ID" "systemctl start amud"
  msg_ok "Restarted server service"

  echo -e "  ${CM}  Dashboard Server inside LXC container $CT_ID updated to $LATEST_RELEASE"
else
  echo -e "  ${WARNING}  AMUD LXC container (amud-dashboard) not found. Skipping server update."
fi

if [ -f "/usr/local/bin/amud-agent" ]; then
  echo -e "\n  ${INFO}  Updating amud-agent on Proxmox host..."

  if systemctl is-active --quiet amud-agent 2>/dev/null; then
    AGENT_WAS_RUNNING=true
  fi

  msg_info "Stopping amud-agent service"
  systemctl stop amud-agent || true
  msg_ok "Stopped amud-agent service"

  if [ ! -f /tmp/amud-agent ]; then
    msg_info "Downloading host agent release binary"
    download_file "https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/amud-agent" "/tmp/amud-agent" "host agent release binary"
    verify_release_asset /tmp/amud-agent amud-agent
  else
    msg_info "Installing staged host agent release binary"
  fi
  install -m 755 /tmp/amud-agent /usr/local/bin/amud-agent
  msg_ok "Host agent release binary installed"

  if [ -n "$CT_ID" ] && [ -n "${AMUD_AGENT_SECRET:-}" ]; then
    sync_agent_ipc_secret "$CT_ID" "$AMUD_AGENT_SECRET"
  elif [ -f "/etc/systemd/system/amud-agent.service" ] && [ -z "$(read_local_systemd_env /etc/systemd/system/amud-agent.service AMUD_AGENT_SECRET)" ]; then
    NEW_SECRET=$(generate_agent_secret)
    ensure_local_systemd_env "/etc/systemd/system/amud-agent.service" "AMUD_AGENT_SECRET" "$NEW_SECRET"
    systemctl daemon-reload
    msg_warn "Generated AMUD_AGENT_SECRET for host agent. Configure the same value on the dashboard server."
  fi

  msg_info "Restarting amud-agent service"
  systemctl start amud-agent
  msg_ok "Restarted amud-agent service"

  echo -e "  ${CM}  Host telemetry agent updated to $LATEST_RELEASE"
else
  echo -e "\n  ${WARNING}  amud-agent binary not found at /usr/local/bin/amud-agent. Skipping host agent update."
fi

if [ -n "$CT_ID" ]; then
  verify_agent_connection "$CT_ID" || true
fi

rm -f /tmp/amud-server /tmp/ui.tar.gz /tmp/amud-agent /tmp/AMUD-SHA256SUMS

echo -e "\n=============================================================="
echo -e "  ${CM}  ${BGN}AMUD ecosystem update completed successfully!${CL}"
echo -e "=============================================================="
echo -e "  ${INFO}  Hard-refresh your browser (Ctrl+Shift+R) after updating."
echo
