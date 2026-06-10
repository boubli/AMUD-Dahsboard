use std::collections::HashMap;

// Global default settings
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

    s.into()
}

pub(crate) const SECRET_SETTING_KEYS: &[&str] = &["pve_api_token", "jellyfin_api_key", "plex_token"];

pub(crate) const EXTRA_SETTING_KEYS: &[&str] = &[
    "overlay_theme",
    "custom_overlay_color",
    "weather_latitude",
    "weather_longitude",
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




// Donation links are fixed to the AMUD author. Self-hosters can toggle the
// Support card on/off in Settings, but cannot change these links.
pub(crate) const DONATION_MESSAGE: &str = "AMUD is completely free and you already have every feature unlocked. A donation is not required and unlocks nothing extra - it is simply a kind way to support continued development. Thank you!";
pub(crate) const DONATION_LINKS: [(&str, &str, &str); 3] = [
    ("https://github.com/sponsors/boubli", "GitHub Sponsors", "github"),
    ("https://buy.stripe.com/cNi14n6b9a7v5Jg4Rq4ko00", "Donate via Card", "credit-card"),
    ("https://ko-fi.com/Youssefboubli", "Ko-fi", "coffee"),
];
