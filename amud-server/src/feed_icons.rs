use std::collections::HashMap;

/// Resolve the logo path for an RSS feed card or settings preview.
pub(crate) fn resolve_feed_logo(
    icon: &str,
    name: &str,
    site_url: &str,
    feed_url: &str,
    manifest: &HashMap<String, String>,
) -> String {
    if icon.starts_with("http") || icon.starts_with('/') {
        return icon.to_string();
    }
    let key = icon.trim().to_lowercase();
    if !key.is_empty() && key != "rss" {
        if let Some(path) = lookup_preset_logo(&key) {
            return path;
        }
        if let Some(path) = manifest.get(&key) {
            return path.clone();
        }
        let dashed = key.replace(' ', "-");
        if let Some(path) = manifest.get(&dashed) {
            return path.clone();
        }
    }
    for url in [site_url, feed_url] {
        if let Some(path) = logo_for_url(url) {
            return path;
        }
    }
    if let Some(path) = logo_for_text(name) {
        return path;
    }
    "/static/feeds/icons/rss.svg".to_string()
}

/// Guess a preset icon key when the admin leaves icon blank.
pub(crate) fn guess_feed_icon_key(name: &str, site_url: &str, feed_url: &str) -> String {
    for url in [site_url, feed_url] {
        if let Some(key) = preset_key_for_url(url) {
            return key;
        }
    }
    if let Some(key) = preset_key_for_text(name) {
        return key;
    }
    "rss".to_string()
}

pub(crate) fn host_from_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let host = without_scheme.split('/').next().unwrap_or("");
    host.strip_prefix("www.").unwrap_or(host).to_lowercase()
}

fn lookup_preset_logo(key: &str) -> Option<String> {
    PRESET_LOGOS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, path)| path.to_string())
}

fn logo_for_url(url: &str) -> Option<String> {
    preset_key_for_url(url).and_then(|key| lookup_preset_logo(&key))
}

fn logo_for_text(text: &str) -> Option<String> {
    preset_key_for_text(text).and_then(|key| lookup_preset_logo(&key))
}

fn preset_key_for_url(url: &str) -> Option<String> {
    let host = host_from_url(url);
    if host.is_empty() {
        return None;
    }
    DOMAIN_PRESETS
        .iter()
        .find(|(domains, _)| {
            domains
                .iter()
                .any(|d| host == *d || host.ends_with(&format!(".{d}")))
        })
        .map(|(_, key)| (*key).to_string())
}

fn preset_key_for_text(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    NAME_PRESETS
        .iter()
        .find(|(needle, _)| lower.contains(needle))
        .map(|(_, key)| (*key).to_string())
}

const PRESET_LOGOS: &[(&str, &str)] = &[
    ("rss", "/static/feeds/icons/rss.svg"),
    ("bbc", "/static/feeds/icons/bbc.svg"),
    ("cnn", "/static/feeds/icons/cnn.svg"),
    ("reuters", "/static/feeds/icons/reuters.svg"),
    ("nytimes", "/static/feeds/icons/nytimes.svg"),
    ("guardian", "/static/feeds/icons/guardian.svg"),
    ("techcrunch", "/static/feeds/icons/techcrunch.svg"),
    ("the-verge", "/static/feeds/icons/the-verge.svg"),
    ("verge", "/static/feeds/icons/the-verge.svg"),
    ("espn", "/static/feeds/icons/espn.svg"),
    ("bloomberg", "/static/feeds/icons/bloomberg.svg"),
    ("hackernews", "/static/feeds/icons/hackernews.svg"),
    ("hn", "/static/feeds/icons/hackernews.svg"),
    ("ars-technica", "/static/feeds/icons/ars-technica.svg"),
    ("ars", "/static/feeds/icons/ars-technica.svg"),
    ("substack", "/static/feeds/icons/substack.svg"),
    ("fox-news", "/static/feeds/icons/fox-news.svg"),
    ("ap-news", "/static/feeds/icons/ap-news.svg"),
    ("wired", "/static/feeds/icons/wired.svg"),
    ("engadget", "/static/feeds/icons/engadget.svg"),
    ("politico", "/static/feeds/icons/politico.svg"),
    ("inoreader", "/static/logos/inoreader.svg"),
    ("reddit", "/static/logos/reddit.svg"),
    ("nasa", "/static/logos/nasa.svg"),
    ("youtube", "/static/logos/youtube.svg"),
    ("github", "/static/logos/github.svg"),
    ("x", "/static/logos/x.svg"),
    ("twitter", "/static/logos/twitter.svg"),
    ("medium", "/static/logos/medium-dark.svg"),
    ("google", "/static/logos/google.svg"),
    ("apple", "/static/logos/apple.svg"),
    ("netflix", "/static/logos/netflix.svg"),
];

const DOMAIN_PRESETS: &[(&[&str], &str)] = &[
    (&["feeds.bbci.co.uk", "bbc.co.uk", "bbc.com"], "bbc"),
    (&["cnn.com"], "cnn"),
    (&["reuters.com"], "reuters"),
    (&["nytimes.com"], "nytimes"),
    (&["theguardian.com", "guardian.co.uk"], "guardian"),
    (&["techcrunch.com"], "techcrunch"),
    (&["theverge.com"], "the-verge"),
    (&["espn.com"], "espn"),
    (&["bloomberg.com"], "bloomberg"),
    (&["news.ycombinator.com", "hnrss.org"], "hackernews"),
    (&["arstechnica.com"], "ars-technica"),
    (&["substack.com"], "substack"),
    (&["foxnews.com"], "fox-news"),
    (&["apnews.com", "ap.org"], "ap-news"),
    (&["wired.com"], "wired"),
    (&["engadget.com"], "engadget"),
    (&["politico.com"], "politico"),
    (&["reddit.com"], "reddit"),
    (&["nasa.gov"], "nasa"),
    (&["youtube.com"], "youtube"),
    (&["github.com", "github.blog"], "github"),
    (&["x.com", "twitter.com"], "x"),
    (&["medium.com"], "medium"),
    (&["inoreader.com"], "inoreader"),
];

const NAME_PRESETS: &[(&str, &str)] = &[
    ("bbc", "bbc"),
    ("cnn", "cnn"),
    ("reuters", "reuters"),
    ("new york times", "nytimes"),
    ("nytimes", "nytimes"),
    ("guardian", "guardian"),
    ("techcrunch", "techcrunch"),
    ("the verge", "the-verge"),
    ("verge", "the-verge"),
    ("espn", "espn"),
    ("bloomberg", "bloomberg"),
    ("hacker news", "hackernews"),
    ("y combinator", "hackernews"),
    ("ars technica", "ars-technica"),
    ("substack", "substack"),
    ("fox news", "fox-news"),
    ("associated press", "ap-news"),
    ("wired", "wired"),
    ("engadget", "engadget"),
    ("politico", "politico"),
    ("reddit", "reddit"),
    ("nasa", "nasa"),
    ("youtube", "youtube"),
    ("github", "github"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_bbc_from_feed_url() {
        let manifest = HashMap::new();
        let logo = resolve_feed_logo(
            "rss",
            "BBC News",
            "",
            "https://feeds.bbci.co.uk/news/rss.xml",
            &manifest,
        );
        assert!(logo.contains("bbc"));
    }

    #[test]
    fn guesses_hackernews_key() {
        assert_eq!(
            guess_feed_icon_key("HN", "", "https://hnrss.org/frontpage"),
            "hackernews"
        );
    }
}
