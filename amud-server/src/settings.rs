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
    s.insert("bento_radius", "16");
    s.insert("grid_columns", "3");
    s.insert("jellyfin_url", "");
    s.insert("jellyfin_api_key", "");
    s.insert("plex_url", "");
    s.insert("plex_token", "");
    s.insert("pve_api_token", "");
    s.insert("donate_enabled", "1");
    s.insert("telemetry_public", "0");
    s.insert("ha_url", "");
    s.insert("ha_token", "");
    s.insert("custom_css", "");
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

    s
}

pub(crate) const SECRET_SETTING_KEYS: &[&str] = &[
    "pve_api_token",
    "jellyfin_api_key",
    "plex_token",
    "ha_token",
    "oidc_client_secret",
];

pub(crate) const EXTRA_SETTING_KEYS: &[&str] = &[
    "weather_latitude",
    "weather_longitude",
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
];

pub(crate) const AGENT_CONFIG_SETTING_KEYS: &[&str] = &[
    "pve_api_token",
    "telemetry_external_ifaces",
    "telemetry_internal_ifaces",
    "telemetry_disk_mounts",
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
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter(|s| {
            s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Comma-separated absolute mount paths (e.g. `/,/mnt/user`).
pub(crate) fn sanitize_disk_mount_list(value: &str) -> String {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| s.starts_with('/') && !s.contains(".."))
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
        "links" | "html" | "calendar_ics" => value.trim().to_ascii_lowercase(),
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
    fn test_parse_show_container_metrics() {
        assert_eq!(parse_show_container_metrics(Some("1")), 1);
        assert_eq!(parse_show_container_metrics(Some("true")), 1);
        assert_eq!(parse_show_container_metrics(Some("0")), 0);
        assert_eq!(parse_show_container_metrics(None), 0);
    }

    #[test]
    fn test_sanitize_iface_list() {
        assert_eq!(sanitize_iface_list("eth0, vmbr0"), "eth0,vmbr0");
        assert_eq!(sanitize_iface_list("eth0,bad!name"), "eth0");
    }

    #[test]
    fn test_sanitize_disk_mount_list() {
        assert_eq!(sanitize_disk_mount_list("/,/mnt/user"), "/,/mnt/user");
        assert_eq!(sanitize_disk_mount_list("relative,/../etc"), "");
    }
}
