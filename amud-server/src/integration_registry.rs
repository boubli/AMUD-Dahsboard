//! Integration catalog: TTL tiers, manifest for UI, Homepage widget type mapping.

use serde_json::{json, Value};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegrationTier {
    Full,
    Standard,
    Health,
    Custom,
}

pub struct IntegrationMeta {
    pub id: &'static str,
    pub label: &'static str,
    pub group: &'static str,
    pub tier: IntegrationTier,
    pub ttl_secs: u64,
}

pub fn ttl_for_type(integration_type: &str) -> Duration {
    let secs = INTEGRATION_CATALOG
        .iter()
        .find(|m| m.id == integration_type)
        .map(|m| m.ttl_secs)
        .unwrap_or(45);
    Duration::from_secs(secs)
}

pub fn map_homepage_widget_type(widget_type: &str) -> Option<&'static str> {
    HOMEPAGE_WIDGET_MAP
        .iter()
        .find(|(k, _)| *k == widget_type)
        .map(|(_, v)| *v)
}

pub fn integration_manifest_json() -> Value {
    let mut groups: std::collections::BTreeMap<&str, Vec<Value>> =
        std::collections::BTreeMap::new();
    for meta in INTEGRATION_CATALOG {
        groups.entry(meta.group).or_default().push(json!({
            "id": meta.id,
            "label": meta.label,
            "tier": tier_str(meta.tier),
            "health_only": meta.tier == IntegrationTier::Health,
        }));
    }
    let grouped: Vec<Value> = groups
        .into_iter()
        .map(|(name, items)| json!({ "group": name, "integrations": items }))
        .collect();
    json!({ "groups": grouped })
}

fn tier_str(tier: IntegrationTier) -> &'static str {
    match tier {
        IntegrationTier::Full => "full",
        IntegrationTier::Standard => "standard",
        IntegrationTier::Health => "health",
        IntegrationTier::Custom => "custom",
    }
}

/// Homepage `widget.type` → AMUD `integration_type`
const HOMEPAGE_WIDGET_MAP: &[(&str, &str)] = &[
    ("pihole", "pihole"),
    ("adguard", "adguard"),
    ("radarr", "radarr"),
    ("sonarr", "sonarr"),
    ("lidarr", "lidarr"),
    ("readarr", "readarr"),
    ("whisparr", "whisparr"),
    ("prowlarr", "prowlarr"),
    ("bazarr", "bazarr"),
    ("overseerr", "overseerr"),
    ("jellyseerr", "jellyseerr"),
    ("qbittorrent", "qbittorrent"),
    ("transmission", "transmission"),
    ("sabnzbd", "sabnzbd"),
    ("nzbget", "nzbget"),
    ("deluge", "deluge"),
    ("jackett", "jackett"),
    ("tautulli", "tautulli"),
    ("plex", "plex"),
    ("jellyfin", "jellyfin"),
    ("emby", "emby"),
    ("immich", "immich"),
    ("portainer", "portainer"),
    ("traefik", "traefik"),
    ("npm", "nginx_proxy_manager"),
    ("nginxproxy", "nginx_proxy_manager"),
    ("grafana", "grafana"),
    ("netdata", "netdata"),
    ("glances", "glances"),
    ("beszel", "beszel"),
    ("uptimekuma", "uptime_kuma"),
    ("truenas", "truenas"),
    ("unraid", "unraid"),
    ("proxmox", "proxmox"),
    ("proxmoxbackupserver", "proxmox_backup"),
    ("opnsense", "opnsense"),
    ("pfsense", "pfsense"),
    ("unifi", "unifi"),
    ("homeassistant", "homeassistant"),
    ("homeassistant", "homeassistant"),
    ("paperless", "paperless"),
    ("mealie", "mealie"),
    ("nextcloud", "nextcloud"),
    ("gitea", "gitea"),
    ("gitlab", "gitlab"),
    ("jenkins", "jenkins"),
    ("minio", "minio"),
    ("frigate", "frigate"),
    ("tdarr", "tdarr"),
    ("audiobookshelf", "audiobookshelf"),
    ("navidrome", "navidrome"),
    ("komga", "komga"),
    ("photoprism", "photoprism"),
    ("fritzbox", "fritz"),
    ("tailscale", "tailscale"),
    ("crowdsec", "crowdsec"),
    ("authentik", "authentik"),
    ("autobrr", "autobrr"),
    ("gotify", "gotify"),
    ("changedetectionio", "changedetection"),
    ("prometheus", "prometheus"),
    ("openmediavault", "openmediavault"),
    ("omada", "omada"),
    ("mikrotik", "mikrotik"),
    ("freshrss", "freshrss"),
    ("customapi", "custom_api"),
];

/// Catalog entries — extend as integrations ship.
pub const INTEGRATION_CATALOG: &[IntegrationMeta] = &[
    // DNS
    IntegrationMeta {
        id: "pihole",
        label: "Pi-hole",
        group: "DNS & adblock",
        tier: IntegrationTier::Full,
        ttl_secs: 45,
    },
    IntegrationMeta {
        id: "adguard",
        label: "AdGuard Home",
        group: "DNS & adblock",
        tier: IntegrationTier::Full,
        ttl_secs: 45,
    },
    IntegrationMeta {
        id: "technitium",
        label: "Technitium DNS",
        group: "DNS & adblock",
        tier: IntegrationTier::Full,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "blocky",
        label: "Blocky DNS",
        group: "DNS & adblock",
        tier: IntegrationTier::Full,
        ttl_secs: 60,
    },
    // Servarr
    IntegrationMeta {
        id: "radarr",
        label: "Radarr",
        group: "Servarr & requests",
        tier: IntegrationTier::Full,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "sonarr",
        label: "Sonarr",
        group: "Servarr & requests",
        tier: IntegrationTier::Full,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "lidarr",
        label: "Lidarr",
        group: "Servarr & requests",
        tier: IntegrationTier::Full,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "readarr",
        label: "Readarr",
        group: "Servarr & requests",
        tier: IntegrationTier::Full,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "whisparr",
        label: "Whisparr",
        group: "Servarr & requests",
        tier: IntegrationTier::Full,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "overseerr",
        label: "Overseerr",
        group: "Servarr & requests",
        tier: IntegrationTier::Full,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "jellyseerr",
        label: "Jellyseerr",
        group: "Servarr & requests",
        tier: IntegrationTier::Full,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "prowlarr",
        label: "Prowlarr",
        group: "Servarr & requests",
        tier: IntegrationTier::Full,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "bazarr",
        label: "Bazarr",
        group: "Servarr & requests",
        tier: IntegrationTier::Full,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "plex",
        label: "Plex",
        group: "Media & photos",
        tier: IntegrationTier::Full,
        ttl_secs: 15,
    },
    IntegrationMeta {
        id: "jellyfin",
        label: "Jellyfin",
        group: "Media & photos",
        tier: IntegrationTier::Full,
        ttl_secs: 15,
    },
    IntegrationMeta {
        id: "custom_api",
        label: "Custom API",
        group: "Custom",
        tier: IntegrationTier::Custom,
        ttl_secs: 30,
    },
    // Phase 5 long-tail (standard tier)
    IntegrationMeta {
        id: "autobrr",
        label: "Autobrr",
        group: "Download",
        tier: IntegrationTier::Standard,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "gotify",
        label: "Gotify",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "changedetection",
        label: "ChangeDetection.io",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "prometheus",
        label: "Prometheus",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "openmediavault",
        label: "OpenMediaVault",
        group: "Network & infra",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "freshrss",
        label: "FreshRSS",
        group: "Self-hosted apps",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "ntfy",
        label: "ntfy",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "coolify",
        label: "Coolify",
        group: "Network & infra",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "aria2",
        label: "Aria2",
        group: "Download",
        tier: IntegrationTier::Standard,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "speedtest_tracker",
        label: "Speedtest Tracker",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 120,
    },
    IntegrationMeta {
        id: "kubernetes",
        label: "Kubernetes",
        group: "Network & infra",
        tier: IntegrationTier::Standard,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "github_release",
        label: "GitHub release tracker",
        group: "Dev & CI",
        tier: IntegrationTier::Standard,
        ttl_secs: 300,
    },
    IntegrationMeta {
        id: "dockerhub_release",
        label: "Docker Hub release tracker",
        group: "Dev & CI",
        tier: IntegrationTier::Standard,
        ttl_secs: 300,
    },
    IntegrationMeta {
        id: "healthchecks",
        label: "Healthchecks.io",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "gatus",
        label: "Gatus",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "scrutiny",
        label: "Scrutiny",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 120,
    },
    IntegrationMeta {
        id: "uptime_robot",
        label: "UptimeRobot",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 120,
    },
    IntegrationMeta {
        id: "mikrotik",
        label: "Mikrotik",
        group: "Network & infra",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "omada",
        label: "TP-Link Omada",
        group: "Network & infra",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "qnap",
        label: "QNAP",
        group: "Network & infra",
        tier: IntegrationTier::Standard,
        ttl_secs: 120,
    },
    IntegrationMeta {
        id: "gluetun",
        label: "Gluetun",
        group: "Network & infra",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "wgeasy",
        label: "WG-Easy",
        group: "Network & infra",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "tubearchivist",
        label: "Tube Archivist",
        group: "Media & photos",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "kavita",
        label: "Kavita",
        group: "Media & photos",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "esphome",
        label: "ESPHome",
        group: "Smart home",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "octoprint",
        label: "OctoPrint",
        group: "Self-hosted apps",
        tier: IntegrationTier::Standard,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "minecraft",
        label: "Minecraft server",
        group: "Self-hosted apps",
        tier: IntegrationTier::Standard,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "firefly_iii",
        label: "Firefly III",
        group: "Self-hosted apps",
        tier: IntegrationTier::Standard,
        ttl_secs: 120,
    },
    IntegrationMeta {
        id: "vikunja",
        label: "Vikunja",
        group: "Self-hosted apps",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "wallos",
        label: "Wallos",
        group: "Self-hosted apps",
        tier: IntegrationTier::Standard,
        ttl_secs: 120,
    },
    IntegrationMeta {
        id: "slskd",
        label: "Slskd",
        group: "Download",
        tier: IntegrationTier::Standard,
        ttl_secs: 30,
    },
    IntegrationMeta {
        id: "umami",
        label: "Umami",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 120,
    },
    IntegrationMeta {
        id: "zabbix",
        label: "Zabbix",
        group: "Monitoring",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "kopia",
        label: "Kopia",
        group: "Backup",
        tier: IntegrationTier::Standard,
        ttl_secs: 120,
    },
    IntegrationMeta {
        id: "headscale",
        label: "Headscale",
        group: "Network & infra",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
    IntegrationMeta {
        id: "stash",
        label: "Stash",
        group: "Media & photos",
        tier: IntegrationTier::Standard,
        ttl_secs: 60,
    },
];
