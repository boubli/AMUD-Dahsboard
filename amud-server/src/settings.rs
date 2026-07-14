use std::collections::HashMap;

pub(crate) fn get_default_settings() -> HashMap<&'static str, &'static str> {
    let mut s = HashMap::new();
    s.insert("app_name", "AMUD");
    s.insert("tagline", "Homelab Operations Cockpit");
    s.insert("accent_color", "#cf6427");
    s.insert("custom_bg_url", "/static/wallpaper.png");
    s.insert("app_logo", "");
    s.insert("glass_blur_intensity", "16");
    s.insert("glass_opacity", "0.45");
    s.insert("wallpaper_overlay_strength", "0.85");
    s.insert("bento_radius", "16");
    s.insert("grid_columns", "3");
    s.insert("pve_api_token", "");
    s.insert("donate_enabled", "1");
    s.insert("telemetry_public", "0");
    s.insert("ha_url", "");
    s.insert("ha_token", "");
    s.insert("custom_css", "");
    s.insert("active_theme_id", "default");
    s.insert("theme_mode", "dark");
    s.insert("theme_scheduler", "off");
    s.insert("theme_light_at", "07:00");
    s.insert("theme_dark_at", "19:00");
    s.insert("guest_category_restrict", "0");
    s.insert("guest_visible_categories", "");
    s.insert("dashboard_layout", "tabs");
    s.insert("status_page_public", "0");
    s.insert("kiosk_mode", "0");
    s.insert("iframe_embeds_enabled", "0");
    s.insert("oidc_enabled", "0");
    s.insert("oidc_issuer", "");
    s.insert("oidc_client_id", "");
    s.insert("oidc_client_secret", "");
    s.insert("oidc_redirect_uri", "");
    s.insert("oidc_default_role", "Guest");
    s.insert("integration_cache_ttl_secs", "45");
    s.insert("integration_cache_max_entries", "256");
    s.insert("feeds_enabled", "1");
    s.insert("agent_telemetry_interval_secs", "5");
    s.insert("agent_lxc_poll_interval_secs", "10");
    s.insert("agent_docker_poll_interval_secs", "10");
    s.insert("status_poll_interval_secs", "15");
    s.insert("media_poll_interval_secs", "5");
    s.insert("ha_poll_interval_secs", "15");
    s.insert("telemetry_broadcast_interval_secs", "5");
    s.insert("integration_coordinator_interval_secs", "45");
    s.insert("ldap_enabled", "0");
    s.insert("ldap_url", "");
    s.insert("ldap_bind_dn", "");
    s.insert("ldap_bind_password", "");
    s.insert("ldap_base_dn", "");
    s.insert("ldap_user_filter", "(uid={username})");
    s.insert("oidc_admin_group", "");
    s.insert("agent_node_tag", "Local");
    s.insert("performance_preset", "light");
    s.insert("idle_grace_secs", "45");
    s.insert("alert_cpu_threshold", "90");
    s.insert("alert_ram_threshold", "90");
    s.insert("alert_disk_threshold", "95");
    s.insert("backup_reminder_days", "30");
    s.insert("webgl_effects_enabled", "1");
    s.insert("greeting_animations_enabled", "1");
    s.insert("dashboard_reorder_enabled", "1");

    s
}

pub(crate) const SECRET_SETTING_KEYS: &[&str] = &[
    "pve_api_token",
    "ha_token",
    "oidc_client_secret",
    "ldap_bind_password",
];

pub(crate) const EXTRA_SETTING_KEYS: &[&str] = &[
    "weather_latitude",
    "weather_longitude",
    "weather_temp_unit",
    "accept_invalid_certs",
    "webhooks_allow_private_ips",
    "enable_proxmox",
    "last_backup_export_at",
    "telemetry_external_ifaces",
    "telemetry_internal_ifaces",
    "telemetry_disk_mounts",
    "theme_scheduler",
    "theme_light_at",
    "theme_dark_at",
    "active_theme_id",
    "guest_category_restrict",
    "guest_visible_categories",
    "dashboard_layout",
    "status_page_public",
    "kiosk_mode",
    "iframe_embeds_enabled",
    "oidc_enabled",
    "oidc_issuer",
    "oidc_client_id",
    "oidc_redirect_uri",
    "oidc_default_role",
    "integration_cache_ttl_secs",
    "integration_cache_max_entries",
    "feeds_enabled",
    "agent_telemetry_interval_secs",
    "agent_lxc_poll_interval_secs",
    "agent_docker_poll_interval_secs",
    "status_poll_interval_secs",
    "media_poll_interval_secs",
    "ha_poll_interval_secs",
    "telemetry_broadcast_interval_secs",
    "integration_coordinator_interval_secs",
    "ldap_enabled",
    "ldap_url",
    "ldap_bind_dn",
    "ldap_base_dn",
    "ldap_user_filter",
    "oidc_admin_group",
    "agent_node_tag",
    "performance_preset",
    "idle_grace_secs",
    "alert_cpu_threshold",
    "alert_ram_threshold",
    "alert_disk_threshold",
    "backup_reminder_days",
    "webgl_effects_enabled",
    "greeting_animations_enabled",
    "dashboard_reorder_enabled",
    "installed_version",
    "last_version_change_at",
    "last_update_method",
];

pub(crate) const AGENT_CONFIG_SETTING_KEYS: &[&str] = &[
    "pve_api_token",
    "telemetry_external_ifaces",
    "telemetry_internal_ifaces",
    "telemetry_disk_mounts",
    "enable_proxmox",
    "agent_telemetry_interval_secs",
    "agent_lxc_poll_interval_secs",
    "agent_docker_poll_interval_secs",
    "agent_node_tag",
];

pub(crate) fn allowed_setting_keys() -> std::collections::HashSet<String> {
    let mut keys: std::collections::HashSet<String> = get_default_settings()
        .keys()
        .map(|k| (*k).to_string())
        .collect();
    for key in EXTRA_SETTING_KEYS {
        keys.insert((*key).to_string());
    }
    keys
}

pub(crate) fn setting_key_allowed(key: &str) -> bool {
    allowed_setting_keys().contains(key)
}

pub(crate) fn sanitize_wallpaper_overlay_strength(value: &str) -> String {
    let v: f64 = value.trim().parse().unwrap_or(0.85);
    format!("{:.2}", v.clamp(0.0, 1.0))
}

pub(crate) fn sanitize_custom_css(value: &str) -> String {
    let mut cleaned = String::new();
    let chars: Vec<char> = value.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] != '<' {
            cleaned.push(chars[i]);
            i += 1;
            continue;
        }

        let mut temp_idx = i + 1;
        while temp_idx < chars.len() && chars[temp_idx].is_whitespace() {
            temp_idx += 1;
        }

        let mut is_slash = false;
        if temp_idx < chars.len() && chars[temp_idx] == '/' {
            is_slash = true;
            temp_idx += 1;
            while temp_idx < chars.len() && chars[temp_idx].is_whitespace() {
                temp_idx += 1;
            }
        }

        let tag_start = temp_idx;
        while temp_idx < chars.len() && chars[temp_idx].is_alphabetic() {
            temp_idx += 1;
        }
        let tag_name: String = chars[tag_start..temp_idx]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase();

        let dangerous = matches!(
            tag_name.as_str(),
            "style" | "script" | "iframe" | "object" | "html" | "body"
        );

        if dangerous {
            // Remove the full dangerous tag token instead of leaving broken fragments like "/style>".
            while temp_idx < chars.len() && chars[temp_idx] != '>' {
                temp_idx += 1;
            }
            if temp_idx < chars.len() && chars[temp_idx] == '>' {
                temp_idx += 1;
            }
            i = temp_idx;
            continue;
        }

        // Keep benign "<" content unchanged (e.g. @media (width < 900px)).
        cleaned.push('<');
        if is_slash {
            cleaned.push('/');
        }
        i += 1;
    }

    cleaned
}

/// Integration base URLs must be empty or absolute http(s).
pub(crate) fn sanitize_integration_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }
    String::new()
}

pub(crate) fn sanitize_setting_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('/') && !trimmed.contains("..") {
        return trimmed.to_string();
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }
    String::new()
}

/// Light/dark only — invalid values fall back to dark.
pub(crate) fn sanitize_theme_mode(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "light" => "light".to_string(),
        _ => "dark".to_string(),
    }
}

/// Theme scheduler mode: off, sunrise_sunset, or manual.
pub(crate) fn sanitize_theme_scheduler(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "sunrise_sunset" => "sunrise_sunset".to_string(),
        "manual" => "manual".to_string(),
        _ => "off".to_string(),
    }
}

/// Bundled theme id from manifest (alphanumeric + hyphens).
pub(crate) fn sanitize_active_theme_id(value: &str) -> String {
    let trimmed = value.trim().to_ascii_lowercase();
    if trimmed.is_empty() || trimmed == "default" {
        return "default".to_string();
    }
    let valid = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
        && trimmed.len() <= 64;
    if valid {
        trimmed
    } else {
        "default".to_string()
    }
}

/// HH:MM clock time for theme scheduler manual mode.
pub(crate) fn sanitize_time_hhmm(value: &str, default: &str) -> String {
    let parts: Vec<&str> = value.trim().split(':').collect();
    if parts.len() == 2 {
        if let (Ok(h), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            if h < 24 && m < 60 {
                return format!("{h:02}:{m:02}");
            }
        }
    }
    default.to_string()
}

pub(crate) fn sanitize_bool_setting(value: &str) -> String {
    if value.trim() == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("on")
    {
        "1".to_string()
    } else {
        "0".to_string()
    }
}

/// Comma-separated dashboard category names visible to guests when restriction is enabled.
pub(crate) fn sanitize_guest_visible_categories(value: &str) -> String {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() <= 64)
        .filter(|s| s.chars().all(|c| !c.is_control()))
        .collect::<Vec<_>>()
        .join(",")
}

/// When restriction is off, returns None (all categories). When on, returns allowed names (may be empty).
pub(crate) fn parse_guest_visible_categories(
    settings: &HashMap<String, String>,
) -> Option<std::collections::HashSet<String>> {
    if settings
        .get("guest_category_restrict")
        .map(|s| s.as_str())
        .unwrap_or("0")
        != "1"
    {
        return None;
    }
    let list = settings
        .get("guest_visible_categories")
        .map(|s| s.as_str())
        .unwrap_or("");
    Some(
        list.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

/// Comma-separated network interface names (e.g. `eth0,vmbr0`).
pub(crate) fn sanitize_iface_list(value: &str) -> String {
    let mut seen = std::collections::HashSet::new();
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase())
        .map(|s| {
            s.trim_matches(|c: char| c.is_whitespace() || c == ',')
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .filter(|s| {
            s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        })
        .filter(|s| seen.insert(s.clone()))
        .collect::<Vec<_>>()
        .join(",")
}

/// Comma-separated absolute mount paths (e.g. `/,/mnt/user`).
pub(crate) fn sanitize_disk_mount_list(value: &str) -> String {
    let mut seen = std::collections::HashSet::new();
    value
        .split(',')
        .map(str::trim)
        .filter(|s| s.starts_with('/') && !s.contains(".."))
        .map(|s| {
            let mut v = s.to_string();
            while v.ends_with('/') && v.len() > 1 {
                v.pop();
            }
            v
        })
        .filter(|s| seen.insert(s.clone()))
        .collect::<Vec<_>>()
        .join(",")
}

/// Bento card span — unknown values become 1x1.
pub(crate) fn sanitize_card_span(value: &str) -> String {
    match value.trim() {
        "2x1" | "1x2" => value.trim().to_string(),
        _ => "1x1".to_string(),
    }
}

/// Integration + container metrics need a tall card when API metrics are shown.
pub(crate) fn resolve_card_span(
    integration_type: &str,
    show_container_metrics: bool,
    integration_visible_metrics: &str,
    requested: &str,
) -> String {
    if integration_type.is_empty() || integration_type == "rss" {
        return sanitize_card_span(requested);
    }
    if integration_api_metrics_hidden(integration_visible_metrics) {
        if show_container_metrics {
            return "1x1".to_string();
        }
        return sanitize_card_span(requested);
    }
    "1x2".to_string()
}

/// Empty JSON array hides API integration metrics on the app card (CPU/RAM may still show).
pub(crate) fn integration_api_metrics_hidden(integration_visible_metrics: &str) -> bool {
    integration_visible_metrics.trim() == "[]"
}

pub(crate) fn default_integration_visible_metrics(integration_type: &str) -> String {
    match integration_type.trim().to_ascii_lowercase().as_str() {
        "jellyfin" | "plex" | "emby" => "[]".to_string(),
        _ => String::new(),
    }
}

pub(crate) fn sanitize_integration_visible_metrics(raw: Option<&str>) -> String {
    let Some(value) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return String::new();
    };
    if value == "[]" {
        return "[]".to_string();
    }
    let Ok(parsed) = serde_json::from_str::<Vec<String>>(value) else {
        return String::new();
    };
    let cleaned: Vec<String> = parsed
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if cleaned.is_empty() {
        return "[]".to_string();
    }
    serde_json::to_string(&cleaned).unwrap_or_default()
}

/// Per-app embed mode: link, iframe, or tab.
pub(crate) fn sanitize_embed_mode(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "iframe" | "tab" => value.trim().to_ascii_lowercase(),
        _ => "link".to_string(),
    }
}

/// Dashboard layout: tabs or sections.
pub(crate) fn sanitize_dashboard_layout(value: &str) -> String {
    if value.trim().eq_ignore_ascii_case("sections") {
        "sections".to_string()
    } else {
        "tabs".to_string()
    }
}

/// Widget type whitelist.
pub(crate) fn sanitize_widget_type(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "links" | "html" | "calendar_ics" | "arr_calendar" | "datetime" | "resources" => {
            value.trim().to_ascii_lowercase()
        }
        _ => "note".to_string(),
    }
}

/// Per-app CPU/RAM row on cards — default on for existing apps.
pub(crate) fn parse_show_container_metrics(value: Option<&str>) -> i64 {
    match value.map(str::trim) {
        Some("1") | Some("true") | Some("on") => 1,
        _ => 0,
    }
}

// Donation links are locked to the author; toggle via show_donation setting.
pub(crate) const DONATION_MESSAGE: &str = "AMUD is completely free and you already have every feature unlocked. A donation is not required and unlocks nothing extra - it is simply a kind way to support continued development. Thank you!";
pub(crate) const DONATION_LINKS: [(&str, &str, &str); 3] = [
    (
        "https://github.com/sponsors/boubli",
        "GitHub Sponsors",
        "github",
    ),
    (
        "https://buy.stripe.com/cNi14n6b9a7v5Jg4Rq4ko00",
        "Donate via Card",
        "credit-card",
    ),
    ("https://ko-fi.com/Youssefboubli", "Ko-fi", "coffee"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_wallpaper_overlay_strength() {
        assert_eq!(sanitize_wallpaper_overlay_strength("0.85"), "0.85");
        assert_eq!(sanitize_wallpaper_overlay_strength("1.5"), "1.00");
        assert_eq!(sanitize_wallpaper_overlay_strength("-0.2"), "0.00");
    }

    #[test]
    fn test_sanitize_custom_css() {
        assert_eq!(
            sanitize_custom_css("body { color: red; }"),
            "body { color: red; }"
        );
        assert_eq!(
            sanitize_custom_css("div > p { color: blue; }"),
            "div > p { color: blue; }"
        );
        assert_eq!(
            sanitize_custom_css("@media (max-width < 600px) { }"),
            "@media (max-width < 600px) { }"
        );
        assert_eq!(
            sanitize_custom_css("</style><script>alert(1)</script>"),
            "alert(1)"
        );
        assert_eq!(sanitize_custom_css("< sCrIpt >"), "");
        assert_eq!(
            sanitize_custom_css("@media (max-width < 900px) { .x { color: red; } }"),
            "@media (max-width < 900px) { .x { color: red; } }"
        );
    }

    #[test]
    fn test_sanitize_theme_mode() {
        assert_eq!(sanitize_theme_mode("light"), "light");
        assert_eq!(sanitize_theme_mode("LIGHT"), "light");
        assert_eq!(sanitize_theme_mode("dark"), "dark");
        assert_eq!(sanitize_theme_mode("invalid"), "dark");
        assert_eq!(sanitize_theme_mode(""), "dark");
    }

    #[test]
    fn test_sanitize_theme_scheduler() {
        assert_eq!(sanitize_theme_scheduler("off"), "off");
        assert_eq!(sanitize_theme_scheduler("sunrise_sunset"), "sunrise_sunset");
        assert_eq!(sanitize_theme_scheduler("manual"), "manual");
        assert_eq!(sanitize_theme_scheduler("bogus"), "off");
    }

    #[test]
    fn test_sanitize_active_theme_id() {
        assert_eq!(sanitize_active_theme_id("default"), "default");
        assert_eq!(sanitize_active_theme_id("nord"), "nord");
        assert_eq!(
            sanitize_active_theme_id("terminal-matrix"),
            "terminal-matrix"
        );
        assert_eq!(sanitize_active_theme_id(""), "default");
        assert_eq!(sanitize_active_theme_id("bad id!"), "default");
    }

    #[test]
    fn test_sanitize_time_hhmm() {
        assert_eq!(sanitize_time_hhmm("7:5", "07:00"), "07:05");
        assert_eq!(sanitize_time_hhmm("25:00", "07:00"), "07:00");
        assert_eq!(sanitize_time_hhmm("", "19:00"), "19:00");
    }

    #[test]
    fn test_parse_guest_visible_categories() {
        let mut settings = HashMap::new();
        assert!(parse_guest_visible_categories(&settings).is_none());

        settings.insert("guest_category_restrict".to_string(), "1".to_string());
        settings.insert(
            "guest_visible_categories".to_string(),
            "Media,General".to_string(),
        );
        let allowed = parse_guest_visible_categories(&settings).unwrap();
        assert!(allowed.contains("Media"));
        assert!(allowed.contains("General"));
    }

    #[test]
    fn test_sanitize_card_span() {
        assert_eq!(sanitize_card_span("1x1"), "1x1");
        assert_eq!(sanitize_card_span("2x1"), "2x1");
        assert_eq!(sanitize_card_span("1x2"), "1x2");
        assert_eq!(sanitize_card_span("2x2"), "1x1");
        assert_eq!(sanitize_card_span(""), "1x1");
    }

    #[test]
    fn test_resolve_card_span_integration_metrics() {
        assert_eq!(resolve_card_span("radarr", true, "", "1x1"), "1x2");
        assert_eq!(resolve_card_span("radarr", false, "", "2x1"), "1x2");
        assert_eq!(resolve_card_span("jellyfin", true, "[]", "1x1"), "1x1");
        assert_eq!(resolve_card_span("jellyfin", false, "[]", "2x1"), "2x1");
        assert_eq!(resolve_card_span("rss", true, "", "1x1"), "1x1");
        assert_eq!(resolve_card_span("", true, "", "2x1"), "2x1");
    }

    #[test]
    fn test_parse_show_container_metrics() {
        assert_eq!(parse_show_container_metrics(Some("1")), 1);
        assert_eq!(parse_show_container_metrics(Some("true")), 1);
        assert_eq!(parse_show_container_metrics(Some("0")), 0);
        assert_eq!(parse_show_container_metrics(None), 0);
    }

    #[test]
    fn test_sanitize_iface_list() {
        assert_eq!(sanitize_iface_list("eth0, vmbr0"), "eth0,vmbr0");
        assert_eq!(sanitize_iface_list("ETH0, eth0 , vmbr0 "), "eth0,vmbr0");
        assert_eq!(sanitize_iface_list("eth0,bad!name"), "eth0");
    }

    #[test]
    fn test_sanitize_disk_mount_list() {
        assert_eq!(sanitize_disk_mount_list("/,/mnt/user"), "/,/mnt/user");
        assert_eq!(
            sanitize_disk_mount_list("/mnt/user/, /mnt/user, /mnt/user/cache/"),
            "/mnt/user,/mnt/user/cache"
        );
        assert_eq!(sanitize_disk_mount_list("relative,/../etc"), "");
    }
}

pub(crate) fn setting_flag(settings: &HashMap<String, String>, key: &str, default: bool) -> bool {
    settings.get(key).map(|s| s == "1").unwrap_or(default)
}

pub(crate) fn feeds_enabled(settings: &HashMap<String, String>) -> bool {
    setting_flag(settings, "feeds_enabled", true)
}

pub(crate) fn setting_u64_bounded(
    settings: &HashMap<String, String>,
    key: &str,
    default: u64,
    min: u64,
    max: u64,
) -> u64 {
    let v = settings
        .get(key)
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(default);
    v.clamp(min, max)
}

pub(crate) fn sanitize_interval_setting(value: &str, default: u64, min: u64, max: u64) -> String {
    let v = value.trim().parse::<u64>().unwrap_or(default);
    v.clamp(min, max).to_string()
}

pub(crate) fn sanitize_cache_max_entries(value: &str) -> String {
    sanitize_interval_setting(value, 256, 16, 512)
}

pub(crate) fn sanitize_cache_ttl_secs(value: &str) -> String {
    sanitize_interval_setting(value, 45, 5, 600)
}

pub(crate) fn apply_integration_cache_limits(
    cache: &crate::integration_cache::IntegrationCache,
    settings: &HashMap<String, String>,
) {
    let max = setting_u64_bounded(settings, "integration_cache_max_entries", 256, 16, 512) as usize;
    let ttl = setting_u64_bounded(settings, "integration_cache_ttl_secs", 45, 5, 600);
    cache.set_limits(max, ttl);
}

pub(crate) fn apply_performance_preset(db: &rusqlite::Connection, preset: &str) {
    let (coord, status, cache_max, cache_ttl, tel_bcast, media, ha, agent_tel) = match preset {
        "balanced" => (45, 15, 48, 45, 5, 5, 15, 5),
        "active" => (20, 10, 48, 30, 3, 3, 10, 3),
        "custom" => return,
        _ => (90, 30, 32, 60, 10, 10, 30, 10),
    };
    for (key, val) in [
        ("integration_coordinator_interval_secs", coord),
        ("status_poll_interval_secs", status),
        ("integration_cache_max_entries", cache_max),
        ("integration_cache_ttl_secs", cache_ttl),
        ("telemetry_broadcast_interval_secs", tel_bcast),
        ("media_poll_interval_secs", media),
        ("ha_poll_interval_secs", ha),
        ("agent_telemetry_interval_secs", agent_tel),
    ] {
        let _ = db.execute(
            "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, val.to_string()],
        );
    }
}

pub(crate) fn sanitize_performance_preset(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "balanced" => "balanced".into(),
        "active" => "active".into(),
        "custom" => "custom".into(),
        _ => "light".into(),
    }
}

pub(crate) fn backup_export_overdue(settings: &HashMap<String, String>) -> bool {
    let reminder_days = settings
        .get("backup_reminder_days")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(30);
    if reminder_days <= 0 {
        return false;
    }
    let last = settings
        .get("last_backup_export_at")
        .map(|s| s.as_str())
        .unwrap_or("");
    if last.is_empty() || last.eq_ignore_ascii_case("never") {
        return true;
    }
    chrono::DateTime::parse_from_rfc3339(last)
        .ok()
        .map(|dt| (chrono::Utc::now() - dt.with_timezone(&chrono::Utc)).num_days() >= reminder_days)
        .unwrap_or(true)
}

#[cfg(test)]
mod v177_tests {
    use super::*;

    #[test]
    fn feeds_enabled_defaults_true() {
        assert!(feeds_enabled(&HashMap::new()));
    }

    #[test]
    fn feeds_disabled_when_zero() {
        let mut s = HashMap::new();
        s.insert("feeds_enabled".to_string(), "0".to_string());
        assert!(!feeds_enabled(&s));
    }

    #[test]
    fn setting_u64_bounded_clamps_high() {
        let mut s = HashMap::new();
        s.insert(
            "agent_telemetry_interval_secs".to_string(),
            "999".to_string(),
        );
        assert_eq!(
            setting_u64_bounded(&s, "agent_telemetry_interval_secs", 5, 3, 60),
            60
        );
    }

    #[test]
    fn apply_integration_cache_limits_updates_ttl() {
        let cache = crate::integration_cache::IntegrationCache::new(64, 45);
        let mut s = HashMap::new();
        s.insert("integration_cache_ttl_secs".to_string(), "90".to_string());
        s.insert(
            "integration_cache_max_entries".to_string(),
            "32".to_string(),
        );
        apply_integration_cache_limits(&cache, &s);
        assert_eq!(cache.default_ttl(), std::time::Duration::from_secs(90));
        assert_eq!(cache.len(), 0);
    }
}
