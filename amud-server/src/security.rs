use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Mask webhook URLs in API responses (SEC-022).
pub(crate) fn mask_webhook_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Ok(parsed) = reqwest::Url::parse(trimmed) {
        let scheme = parsed.scheme();
        let host = parsed.host_str().unwrap_or("unknown");
        let suffix = trimmed
            .chars()
            .rev()
            .take(6)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        return format!("{scheme}://{host}/…{suffix}");
    }
    if trimmed.len() <= 8 {
        return "••••••••".to_string();
    }
    format!("{}…{}", &trimmed[..4], &trimmed[trimmed.len() - 4..])
}

/// Block outbound webhook targets to localhost/metadata/link-local (SSRF mitigation).
/// When `allow_private` is true, RFC1918 LAN targets are permitted (homelab ntfy/Gotify).
pub(crate) fn url_allowed_for_webhook(raw: &str, allow_private: bool) -> bool {
    let Ok(parsed) = reqwest::Url::parse(raw.trim()) else {
        return false;
    };
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host_lower = host.to_lowercase();
    if host_lower == "localhost" || host_lower.ends_with(".localhost") {
        return false;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if allow_private {
            return !is_blocked_webhook_target_strict(ip);
        }
        return !is_blocked_webhook_target_strict(ip) && !is_private_ip(ip);
    }
    !host_lower.contains("metadata.google")
}

fn is_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_private(),
        std::net::IpAddr::V6(v6) => v6.is_unique_local(),
    }
}

fn is_blocked_webhook_target_strict(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_unspecified()
                || v4.is_link_local()
                || v4.octets() == [169, 254, 169, 254]
                || v4.octets()[0] == 0
        }
        std::net::IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

/// Block health-check requests to localhost/metadata; allow RFC1918 homelab targets (SEC-007).
pub(crate) fn url_allowed_for_health_check(raw: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(raw.trim()) else {
        return false;
    };
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host_lower = host.to_lowercase();
    if host_lower == "localhost" || host_lower.ends_with(".localhost") {
        return false;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return !is_blocked_health_target(ip);
    }
    // Resolve-free: also block literal metadata hostname
    !host_lower.contains("metadata.google")
}

fn is_blocked_health_target(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_unspecified()
                || v4.is_link_local()
                || v4.octets() == [169, 254, 169, 254]
                || v4.octets()[0] == 0
        }
        std::net::IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

/// RSS feed fetch targets: homelab-friendly (RFC1918 allowed), block loopback/metadata.
pub(crate) fn url_allowed_for_rss_feed(raw: &str) -> bool {
    url_allowed_for_health_check(raw)
}

/// Returns a trimmed feed URL when it passes RSS SSRF policy.
pub(crate) fn sanitize_rss_feed_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || !url_allowed_for_rss_feed(trimmed) {
        return None;
    }
    Some(trimmed.to_string())
}

/// Only absolute http(s) links are safe for dashboard href binding.
pub(crate) fn sanitize_feed_link(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(parsed) = reqwest::Url::parse(trimmed) else {
        return String::new();
    };
    match parsed.scheme() {
        "http" | "https" => trimmed.to_string(),
        _ => String::new(),
    }
}

pub(crate) fn client_ip(headers: &axum::http::HeaderMap) -> String {
    let trust_proxy = std::env::var("AMUD_TRUST_PROXY").unwrap_or_default() == "1" || cfg!(test);
    if trust_proxy {
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .or_else(|| {
                headers
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or("unknown")
            .to_string()
    } else {
        "127.0.0.1".to_string()
    }
}

pub(crate) struct RateLimitConfig {
    pub max: usize,
    pub window: Duration,
}

const MAX_RATE_KEYS: usize = 4096;

pub(crate) fn rate_limit_exceeded(
    store: &Mutex<HashMap<String, Vec<Instant>>>,
    key: &str,
    max: usize,
    window: Duration,
) -> bool {
    let now = Instant::now();
    let mut attempts = store.lock().unwrap();
    attempts.retain(|_, values| {
        values.retain(|t| now.duration_since(*t) <= window);
        !values.is_empty()
    });
    if !attempts.contains_key(key) && attempts.len() >= MAX_RATE_KEYS {
        return true;
    }
    attempts.get(key).map(|v| v.len() >= max).unwrap_or(false)
}

pub(crate) fn record_rate_attempt(store: &Mutex<HashMap<String, Vec<Instant>>>, key: &str) {
    store
        .lock()
        .unwrap()
        .entry(key.to_string())
        .or_default()
        .push(Instant::now());
}

/// Returns `true` when the request is allowed.
pub(crate) fn enforce_rate_limit(
    store: &Mutex<HashMap<String, Vec<Instant>>>,
    key: &str,
    config: RateLimitConfig,
) -> bool {
    if rate_limit_exceeded(store, key, config.max, config.window) {
        return false;
    }
    record_rate_attempt(store, key);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_localhost_health_checks() {
        assert!(!url_allowed_for_health_check("http://127.0.0.1:8080"));
        assert!(!url_allowed_for_health_check("http://localhost/admin"));
        assert!(!url_allowed_for_health_check(
            "http://169.254.169.254/latest/meta-data"
        ));
    }

    #[test]
    fn allows_private_homelab_targets() {
        assert!(url_allowed_for_health_check("http://192.168.1.50:8096"));
        assert!(url_allowed_for_health_check("https://10.0.0.12:32400"));
    }

    #[test]
    fn webhook_blocks_private_ip_by_default() {
        assert!(!url_allowed_for_webhook("http://192.168.1.50:8080", false));
        assert!(!url_allowed_for_webhook("http://127.0.0.1:8080", false));
    }

    #[test]
    fn webhook_allows_private_ip_when_enabled() {
        assert!(url_allowed_for_webhook("http://192.168.1.50:8080", true));
        assert!(!url_allowed_for_webhook("http://127.0.0.1:8080", true));
    }

    #[test]
    fn masks_webhook_urls() {
        let masked =
            mask_webhook_url("https://discord.com/api/webhooks/123456789/abcdefghijklmnop");
        assert!(!masked.contains("abcdefghijklmnop"));
        assert!(masked.contains("discord.com"));
    }

    #[test]
    fn rss_feed_blocks_loopback_and_metadata() {
        assert!(!url_allowed_for_rss_feed("http://127.0.0.1/feed"));
        assert!(!url_allowed_for_rss_feed("http://localhost/feed"));
        assert!(!url_allowed_for_rss_feed(
            "http://169.254.169.254/latest/meta-data"
        ));
    }

    #[test]
    fn rss_feed_allows_homelab_and_public() {
        assert!(url_allowed_for_rss_feed("http://192.168.1.50/feed.xml"));
        assert!(url_allowed_for_rss_feed(
            "https://feeds.bbci.co.uk/news/rss.xml"
        ));
    }

    #[test]
    fn sanitize_rss_feed_url_rejects_invalid() {
        assert!(sanitize_rss_feed_url("http://127.0.0.1/feed").is_none());
        assert_eq!(
            sanitize_rss_feed_url("  https://example.com/feed.xml  ").as_deref(),
            Some("https://example.com/feed.xml")
        );
    }

    #[test]
    fn sanitize_feed_link_strips_dangerous_schemes() {
        assert_eq!(sanitize_feed_link("javascript:alert(1)"), "");
        assert_eq!(sanitize_feed_link("data:text/html,hi"), "");
        assert_eq!(sanitize_feed_link("/relative/path"), "");
        assert_eq!(
            sanitize_feed_link("https://example.com/article"),
            "https://example.com/article"
        );
    }

    #[test]
    fn rate_limit_blocks_burst() {
        let store = Mutex::new(HashMap::new());
        assert!(enforce_rate_limit(
            &store,
            "test:127.0.0.1",
            RateLimitConfig {
                max: 2,
                window: Duration::from_secs(60),
            },
        ));
        assert!(enforce_rate_limit(
            &store,
            "test:127.0.0.1",
            RateLimitConfig {
                max: 2,
                window: Duration::from_secs(60),
            },
        ));
        assert!(!enforce_rate_limit(
            &store,
            "test:127.0.0.1",
            RateLimitConfig {
                max: 2,
                window: Duration::from_secs(60),
            },
        ));
    }
}
