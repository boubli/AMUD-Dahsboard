$ErrorActionPreference = "SilentlyContinue"

# List of all target apps
$targetApps = @(
    "AdGuard Home", "Technitium DNS", "Blocky", "Unbound", "WireGuard", "Tailscale", "Headscale",
    "NetBird", "ZeroTier", "Nebula", "Nginx Proxy Manager", "Traefik", "Caddy", "SWAG", "HAProxy",
    "Cloudflared", "Pomerium", "Pingora", "Bungee", "Zoraxy", "Authelia", "Authentik", "Keycloak",
    "Vaultwarden", "CrowdSec", "Fail2Ban", "Apache Guacamole", "Kasm Workspaces", "Teleport", "LLDAP",
    "Homepage", "Dashy", "Homer", "Heimdall", "Flame", "Uptime Kuma", "Grafana", "Prometheus", "Netdata",
    "Glances", "Plex", "Emby", "Immich", "PhotoPrism", "Nextcloud Memories", "Navidrome", "Audiobookshelf",
    "Kavita", "Komga", "Tdarr", "Sonarr", "Radarr", "Lidarr", "Readarr", "Prowlarr", "Jackett", "Bazarr",
    "Overseerr", "qBittorrent", "SABnzbd", "Nextcloud", "ownCloud", "Seafile", "Syncthing", "FileBrowser",
    "Paperless-ngx", "Stirling-PDF", "Kopia", "Duplicati", "Restic", "Home Assistant", "Node-RED", "n8n",
    "Huginn", "Eclipse Mosquitto", "Zigbee2MQTT", "Frigate", "Scrypted", "Mealie", "Tandoor Recipes",
    "BookStack", "Wiki.js", "Memos", "Joplin Server", "Matrix", "Element", "Gitea", "GitLab", "Ghost",
    "WordPress", "Portainer", "Dockge", "Yacht", "Watchtower", "Dozzle", "Ouroboros", "It-Tools", "RomM",
    "ChangeDetection.io", "RustDesk", "Proxmox", "Pi-hole", "Docker", "TrueNAS", "CasaOS", "Jellyfin"
)

Write-Host "Creating output directory: ui/static/logos..."
New-Item -ItemType Directory -Force -Path "ui/static/logos" | Out-Null

$successCount = 0
$failCount = 0

# A list of common suffixes/words to strip for matching
$suffixes = @("-dns", "-workspaces", "-server", "-recipes", "-memories", "-home", "-assistant", "-io", "home")

$webClient = New-Object System.Net.WebClient
$webClient.Headers.Add("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")

foreach ($app in $targetApps) {
    # Generate direct kebab name
    $kebab = $app.ToLower().Replace(" ", "-").Replace(".", "")
    if ($kebab -eq "changedetectionio") { $kebab = "changedetection" }

    # Candidates to try
    $candidates = [System.Collections.Generic.List[string]]::new()
    $candidates.Add($kebab)

    # Add variations by stripping suffixes
    foreach ($suffix in $suffixes) {
        if ($kebab.EndsWith($suffix)) {
            $candidates.Add($kebab.Substring(0, $kebab.Length - $suffix.Length))
        }
        if ($kebab.Contains($suffix)) {
            $candidates.Add($kebab.Replace($suffix, ""))
        }
    }

    # Custom special mapping rules
    if ($kebab -eq "cloudflared") { $candidates.Add("cloudflare") }
    if ($kebab -eq "apache-guacamole") { $candidates.Add("guacamole") }
    if ($kebab -eq "eclipse-mosquitto") { $candidates.Add("mosquitto") }
    if ($kebab -eq "wiki-js") { $candidates.Add("wikijs") }
    if ($kebab -eq "changedetectionio" -or $kebab -eq "changedetection-io") { $candidates.Add("changedetection") }

    $downloaded = $false
    $destFile = "ui/static/logos/$kebab.svg"

    Write-Host "Downloading icon for: $app... " -NoNewline

    foreach ($candidate in $candidates) {
        if ([string]::IsNullOrEmpty($candidate)) { continue }
        $url = "https://raw.githubusercontent.com/homarr-labs/dashboard-icons/main/svg/$candidate.svg"

        try {
            $webClient.DownloadFile($url, $destFile)
            if (Test-Path $destFile) {
                # Check if it's an actual XML/SVG file (not a GitHub 404 page or error message)
                $firstBytes = Get-Content -Path $destFile -TotalCount 1
                if ($firstBytes -like "*<svg*" -or $firstBytes -like "*xml*") {
                    Write-Host "SUCCESS (as $candidate.svg)" -ForegroundColor Green
                    $downloaded = $true
                    $successCount++
                    break;
                } else {
                    Remove-Item -Path $destFile -Force
                }
            }
        } catch {
            Write-Verbose "Icon candidate failed: $($_.Exception.Message)"
        }
    }

    if (-not $downloaded) {
        Write-Host "FAILED" -ForegroundColor Red
        $failCount++
    }
}

Write-Host "Download complete. Success: $successCount, Failed: $failCount"
