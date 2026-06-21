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

    s
}

pub(crate) const SECRET_SETTING_KEYS: &[&str] = &[
    "pve_api_token",
    "jellyfin_api_key",
    "plex_token",
    "ha_token",
];

pub(crate) const EXTRA_SETTING_KEYS: &[&str] = &[
    "weather_latitude",
    "weather_longitude",
    "accept_invalid_certs",
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
        if chars[i] == '<' {
            let mut temp_idx = i + 1;
            while temp_idx < chars.len() && chars[temp_idx].is_whitespace() {
                temp_idx += 1;
            }
            if temp_idx < chars.len() {
                let is_slash = chars[temp_idx] == '/';
                if is_slash {
                    temp_idx += 1;
                    while temp_idx < chars.len() && chars[temp_idx].is_whitespace() {
                        temp_idx += 1;
                    }
                }

                let mut tag_name = String::new();
                while temp_idx < chars.len() && chars[temp_idx].is_alphabetic() {
                    tag_name.push(chars[temp_idx].to_ascii_lowercase());
                    temp_idx += 1;
                }

                if tag_name == "style"
                    || tag_name == "script"
                    || tag_name == "iframe"
                    || tag_name == "object"
                    || tag_name == "html"
                    || tag_name == "body"
                {
                    i += 1;
                    continue;
                }
            }
        }
        cleaned.push(chars[i]);
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

/// Bento card span — unknown values become 1x1.
pub(crate) fn sanitize_card_span(value: &str) -> String {
    match value.trim() {
        "2x1" | "1x2" => value.trim().to_string(),
        _ => "1x1".to_string(),
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
            "/style>script>alert(1)/script>"
        );
        assert_eq!(sanitize_custom_css("< sCrIpt >"), " sCrIpt >");
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
    fn test_sanitize_card_span() {
        assert_eq!(sanitize_card_span("1x1"), "1x1");
        assert_eq!(sanitize_card_span("2x1"), "2x1");
        assert_eq!(sanitize_card_span("1x2"), "1x2");
        assert_eq!(sanitize_card_span("2x2"), "1x1");
        assert_eq!(sanitize_card_span(""), "1x1");
    }
}
