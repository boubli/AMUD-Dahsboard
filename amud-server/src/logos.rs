use std::collections::HashMap;

const LOGO_DIRS: &[&str] = &["ui/static/logos", "static/logos"];
const LOGO_EXTENSIONS: &[&str] = &["svg", "png", "jpg"];

/// Scan logo directories once at startup. Keys are lowercase stems; values are web paths.
pub(crate) fn build_logo_manifest() -> HashMap<String, String> {
    let mut manifest = HashMap::new();
    for dir in LOGO_DIRS {
        scan_logo_dir(dir, &mut manifest);
    }
    manifest
}

fn scan_logo_dir(dir: &str, manifest: &mut HashMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        let ext = ext.to_lowercase();
        if !LOGO_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let stem_key = stem.to_lowercase();
        if manifest.contains_key(&stem_key) {
            continue;
        }
        let web_path = if dir.starts_with("ui/") {
            format!("/static/logos/{}.{}", stem, ext)
        } else {
            format!("/{}/{}.{}", dir, stem, ext)
        };
        manifest.insert(stem_key, web_path);
    }
}

pub(crate) fn resolve_logo_from_manifest(icon: &str, manifest: &HashMap<String, String>) -> String {
    if icon.starts_with("http") || icon.starts_with('/') {
        return icon.to_string();
    }
    let lowercase = icon.to_lowercase();
    if lowercase.is_empty() {
        return String::new();
    }
    for key in [lowercase.as_str(), lowercase.replace(' ', "-").as_str()] {
        if let Some(path) = manifest.get(key) {
            return path.clone();
        }
    }
    String::new()
}

pub(crate) fn fallback_brand_logo(lowercase_icon: &str) -> String {
    match lowercase_icon {
        "plex" => "/static/logos/plex.svg".to_string(),
        "jellyfin" => "/static/logos/jellyfin.svg".to_string(),
        "proxmox" | "pve" => "/static/logos/proxmox.svg".to_string(),
        "portainer" => "/static/logos/portainer.svg".to_string(),
        "home-assistant" | "homeassistant" => "/static/logos/home-assistant.svg".to_string(),
        "nextcloud" => "/static/logos/nextcloud.svg".to_string(),
        "adguard" | "adguard-home" => "/static/logos/adguard-home.svg".to_string(),
        "pihole" | "pi-hole" => "/static/logos/pi-hole.svg".to_string(),
        "sonarr" => "/static/logos/sonarr.svg".to_string(),
        "radarr" => "/static/logos/radarr.svg".to_string(),
        "lidarr" => "/static/logos/lidarr.svg".to_string(),
        "readarr" => "/static/logos/readarr.svg".to_string(),
        "whisparr" => "/static/logos/whisparr.svg".to_string(),
        "qbittorrent" => "/static/logos/qbittorrent.svg".to_string(),
        "transmission" => "/static/logos/transmission.svg".to_string(),
        "sabnzbd" => "/static/logos/sabnzbd.svg".to_string(),
        "nzbget" => "/static/logos/nzbget.svg".to_string(),
        "jackett" => "/static/logos/jackett.svg".to_string(),
        "tautulli" => "/static/logos/tautulli.svg".to_string(),
        "immich" => "/static/logos/immich-frame.svg".to_string(),
        "maintainerr" => "/static/logos/maintainerr.svg".to_string(),
        "frigate" => "/static/logos/frigate.svg".to_string(),
        "overseerr" => "/static/logos/overseerr.svg".to_string(),
        "prowlarr" => "/static/logos/servarr.svg".to_string(),
        "bazarr" => "/static/logos/bazarr-dark.svg".to_string(),
        "uptime-kuma" | "uptimekuma" | "uptime_kuma" => "/static/logos/uptime-kuma.svg".to_string(),
        "cloudflare" | "cloudflared" | "cloudflare-tunnel" | "cloudflare_tunnel" => {
            "/static/logos/cloudflared-light.svg".to_string()
        }
        "peanut" => "/static/logos/peanut.svg".to_string(),
        "fritz" | "fritzbox" | "fritz-box" | "avm" => "/static/logos/fritzbox.svg".to_string(),
        "truenas" => "/static/logos/truenas.svg".to_string(),
        "unifi" | "ubiquiti" => "/static/logos/unifi.svg".to_string(),
        "opnsense" => "/static/logos/opnsense.svg".to_string(),
        "pfsense" => "/static/logos/pfsense.svg".to_string(),
        "grafana" => "/static/logos/grafana.svg".to_string(),
        "netdata" => "/static/logos/netdata.svg".to_string(),
        "glances" => "/static/logos/glances-light.svg".to_string(),
        "beszel" => "/static/logos/beszel.svg".to_string(),
        "paperless" | "paperless-ngx" | "paperless_ngx" => {
            "/static/logos/paperless-ngx.svg".to_string()
        }
        "mealie" => "/static/logos/mealie.svg".to_string(),
        "vaultwarden" => "/static/logos/vaultwarden-light.svg".to_string(),
        "deluge" => "/static/logos/deluge.svg".to_string(),
        "navidrome" => "/static/logos/navidrome.svg".to_string(),
        "komga" => "/static/logos/komga.svg".to_string(),
        "photoprism" => "/static/logos/photoprism.svg".to_string(),
        "tailscale" => "/static/logos/tailscale-light.svg".to_string(),
        "netbird" => "/static/logos/netbird.svg".to_string(),
        "synology" | "dsm" => "/static/logos/synology.svg".to_string(),
        "unraid" => "/static/logos/unraid.svg".to_string(),
        "dockge" => "/static/logos/dockge.svg".to_string(),
        "nginx_proxy_manager" | "npm" => "/static/logos/nginx-proxy-manager.svg".to_string(),
        "traefik" => "/static/logos/traefik-proxy.svg".to_string(),
        "authentik" => "/static/logos/authentik.svg".to_string(),
        "authelia" => "/static/logos/authelia.svg".to_string(),
        "crowdsec" => "/static/logos/crowdsec-web-ui.svg".to_string(),
        "node_red" | "node-red" => "/static/logos/node-red.svg".to_string(),
        "zigbee2mqtt" => "/static/logos/zigbee2mqtt.svg".to_string(),
        "emby" => "/static/logos/emby.svg".to_string(),
        "scrypted" => "/static/logos/scrypted.svg".to_string(),
        "mylar" => "/static/logos/mylar.svg".to_string(),
        "kapowarr" => "/static/logos/kapowarr.svg".to_string(),
        "huntarr" => "/static/logos/huntarr.svg".to_string(),
        "proxmox_backup" | "pbs" => "/static/logos/proxmox-light.svg".to_string(),
        "technitium" => "/static/logos/technitium.svg".to_string(),
        "blocky" => "/static/logos/blocky.svg".to_string(),
        "openwrt" => "/static/logos/openwrt.svg".to_string(),
        "gitea" => "/static/logos/gitea.svg".to_string(),
        "forgejo" => "/static/logos/forgejo.svg".to_string(),
        "gitlab" => "/static/logos/gitlab.svg".to_string(),
        "jenkins" => "/static/logos/jenkins.svg".to_string(),
        "drone" => "/static/logos/drone.svg".to_string(),
        "minio" => "/static/logos/minio.svg".to_string(),
        "garage" => "/static/logos/garage.svg".to_string(),
        "seaweedfs" => "/static/logos/seaweedfs.svg".to_string(),
        "kopia" => "/static/logos/kopia.svg".to_string(),
        "restic" => "/static/logos/restic.svg".to_string(),
        "duplicati" => "/static/logos/duplicati.svg".to_string(),
        "urbackup" => "/static/logos/urbackup.svg".to_string(),
        "kodi" => "/static/logos/kodi.svg".to_string(),
        "stash" => "/static/logos/stash.svg".to_string(),
        "channels_dvr" => "/static/logos/channels-dvr.svg".to_string(),
        "calibre_web" | "calibre-web" => "/static/logos/calibre-web.svg".to_string(),
        "headscale" => "/static/logos/headscale.svg".to_string(),
        "wireguard_ui" => "/static/logos/wireguard.svg".to_string(),
        "openvpn" => "/static/logos/openvpn.svg".to_string(),
        "hubitat" => "/static/logos/hubitat.svg".to_string(),
        "smartthings" => "/static/logos/smartthings.svg".to_string(),
        "iobroker" => "/static/logos/iobroker.svg".to_string(),
        "blue_iris" => "/static/logos/blue-iris.svg".to_string(),
        "shinobi" => "/static/logos/shinobi.svg".to_string(),
        "agent_dvr" => "/static/logos/agent-dvr.svg".to_string(),
        "casaos" => "/static/logos/casaos.svg".to_string(),
        _ => "/static/fallback.svg".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_manifest_entry() {
        let mut manifest = HashMap::new();
        manifest.insert("sonarr".to_string(), "/static/logos/sonarr.svg".to_string());
        assert_eq!(
            resolve_logo_from_manifest("Sonarr", &manifest),
            "/static/logos/sonarr.svg"
        );
    }

    #[test]
    fn resolve_passes_through_urls() {
        let manifest = HashMap::new();
        assert_eq!(
            resolve_logo_from_manifest("https://cdn.example/logo.png", &manifest),
            "https://cdn.example/logo.png"
        );
    }
}
