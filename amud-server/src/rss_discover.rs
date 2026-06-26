use crate::feed_icons::{auto_feed_icon_url, host_from_url};
use crate::integrations::extract_feed_icon;
use crate::security::{sanitize_rss_feed_url, url_allowed_for_rss_feed};
use feed_rs::parser;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

const MAX_HTML_BYTES: usize = 512 * 1024;
const COMMON_FEED_PATHS: &[&str] = &[
    "/feed",
    "/rss",
    "/rss.xml",
    "/feed.xml",
    "/atom.xml",
    "/index.xml",
    "/feeds/posts/default",
    "/blog/feed",
    "/news/rss.xml",
];

fn normalize_site_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    if !url_allowed_for_rss_feed(&with_scheme) {
        return None;
    }
    Some(with_scheme)
}

fn extract_feed_links_from_html(html: &str, base_url: &str) -> Vec<String> {
    let mut found = Vec::new();
    let lower = html.to_lowercase();
    let mut search_from = 0;
    while let Some(rel_pos) = lower[search_from..].find("rel=\"alternate\"") {
        let start = search_from + rel_pos.saturating_sub(200);
        let end = (search_from + rel_pos + 400).min(html.len());
        let chunk = &html[start..end];
        if chunk.to_lowercase().contains("application/rss+xml")
            || chunk.to_lowercase().contains("application/atom+xml")
        {
            if let Some(href) = extract_href_from_chunk(chunk) {
                if let Ok(resolved) = reqwest::Url::parse(base_url)
                    .and_then(|base| base.join(&href))
                    .map(|u| u.to_string())
                {
                    if sanitize_rss_feed_url(&resolved).is_some() {
                        found.push(resolved);
                    }
                }
            }
        }
        search_from = search_from + rel_pos + 1;
    }
    found
}

fn extract_href_from_chunk(chunk: &str) -> Option<String> {
    let lower = chunk.to_lowercase();
    let href_key = "href=\"";
    let start = lower.find(href_key)? + href_key.len();
    let rest = &chunk[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

async fn fetch_text(client: &Client, url: &str) -> Option<String> {
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    if bytes.len() > MAX_HTML_BYTES {
        return None;
    }
    String::from_utf8(bytes.to_vec()).ok()
}

async fn validate_feed(client: &Client, feed_url: &str) -> Option<(String, Option<String>)> {
    let resp = client.get(feed_url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    if bytes.len() > 2 * 1024 * 1024 {
        return None;
    }
    let feed = parser::parse(&bytes[..]).ok()?;
    let title = feed
        .title
        .as_ref()
        .map(|t| t.content.trim().to_string())
        .filter(|t| !t.is_empty());
    let icon = extract_feed_icon(&feed);
    Some((title.unwrap_or_else(|| host_from_url(feed_url)), icon))
}

pub async fn discover_rss_feed(site_url: &str) -> Option<Value> {
    let site_url = normalize_site_url(site_url)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("AMUD-Dashboard/1.5 RSS-Discover")
        .build()
        .ok()?;

    let mut candidates: Vec<String> = Vec::new();

    if let Some(html) = fetch_text(&client, &site_url).await {
        candidates.extend(extract_feed_links_from_html(&html, &site_url));
    }

    if let Ok(base) = reqwest::Url::parse(&site_url) {
        for path in COMMON_FEED_PATHS {
            if let Ok(joined) = base.join(path) {
                let url = joined.to_string();
                if sanitize_rss_feed_url(&url).is_some() {
                    candidates.push(url);
                }
            }
        }
    }

    candidates.sort();
    candidates.dedup();

    for feed_url in candidates {
        if let Some((title, feed_icon)) = validate_feed(&client, &feed_url).await {
            let icon_url = feed_icon.unwrap_or_else(|| auto_feed_icon_url(&site_url, &feed_url));
            return Some(json!({
                "feed_url": feed_url,
                "site_url": site_url,
                "title": title,
                "icon_url": icon_url,
            }));
        }
    }

    None
}
