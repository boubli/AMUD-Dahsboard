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
        "proxmox" => "/static/logos/proxmox.svg".to_string(),
        "portainer" => "/static/logos/portainer.svg".to_string(),
        "home-assistant" | "homeassistant" => "/static/logos/home-assistant.svg".to_string(),
        "nextcloud" => "/static/logos/nextcloud.svg".to_string(),
        "adguard" | "adguard-home" => "/static/logos/adguard-home.svg".to_string(),
        "pihole" | "pi-hole" => "/static/logos/pi-hole.svg".to_string(),
        "sonarr" => "/static/logos/sonarr.svg".to_string(),
        "radarr" => "/static/logos/radarr.svg".to_string(),
        "qbittorrent" => "/static/logos/qbittorrent.svg".to_string(),
        "transmission" => "/static/logos/transmission.svg".to_string(),
        "overseerr" => "/static/logos/overseerr.svg".to_string(),
        "prowlarr" => "/static/logos/servarr.svg".to_string(),
        "bazarr" => "/static/logos/bazarr-dark.svg".to_string(),
        "uptime-kuma" | "uptimekuma" | "uptime_kuma" => "/static/logos/uptime-kuma.svg".to_string(),
        "cloudflare" | "cloudflared" | "cloudflare-tunnel" | "cloudflare_tunnel" => {
            "/static/logos/cloudflared-light.svg".to_string()
        }
        "peanut" => "/static/logos/peanut.svg".to_string(),
        "truenas" => "/static/logos/truenas.svg".to_string(),
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
