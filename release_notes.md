## AMUD Dashboard Autopilot Release v1.2.1.0

This release focuses on massive stability improvements to the autopilot updater and introduces secure IPC authentication. These changes directly address the recent issues with failing updates and ensure future updates are robust and reliable.

### 🚀 Major Improvements

*   **Bulletproof Autopilot Updater (`update-amud.sh`)**
    *   **Host-Side Downloads:** The Proxmox host now securely downloads all release assets (`amud-server`, `ui.tar.gz`, `amud-agent`), verifies their SHA256 checksums, and pushes them directly into the LXC container. This completely bypasses container-side networking/DNS issues that caused previous updates to fail.
    *   **Strict Error Handling:** The updater now features global error traps. If any step fails, it halts immediately, preventing the server from being left in a broken state.
    *   **UI Asset Verification:** The updater now verifies the existence of critical UI assets before attempting to restart the service.

*   **Secure IPC Authentication**
    *   Communication between the host-side `amud-agent` and the container-side `amud-server` over Unix Domain Sockets is now authenticated using a shared secret (`AMUD_AGENT_SECRET`).
    *   **Auto-Sync:** The updater automatically generates and securely syncs this secret between the host and the LXC container during the update process.

*   **Installer Improvements (`setup-amud.sh`)**
    *   The initial installer has been refactored to use the same reliable host-side download and `pct push` pattern as the updater.
    *   Automatic setup and configuration of the `AMUD_AGENT_SECRET`.

### 🛡️ Security
*   Hardened socket permissions and authentication to prevent unauthorized local access to the AMUD Dashboard API.

### 🐛 Bug Fixes
*   Fixed the issue where `ui.tar.gz` and `SHA256SUMS` failed to download inside the LXC container.
*   Fixed silent failures during the update process by enforcing strict bash settings (`set -e`).

**Note for Testers:** After updating to this version, please hard-refresh your browser (`Ctrl+Shift+R`) to ensure you have the latest UI assets.
