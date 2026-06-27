//! ICS calendar feed parser for dashboard calendar widgets.

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

const MAX_EVENTS: usize = 8;

pub fn parse_ics_preview(ics: &str) -> Vec<Value> {
    let mut events = Vec::new();
    let mut in_event = false;
    let mut summary = String::new();
    let mut dtstart = String::new();
    for line in ics.lines() {
        let line = line.trim();
        if line == "BEGIN:VEVENT" {
            in_event = true;
            summary.clear();
            dtstart.clear();
            continue;
        }
        if line == "END:VEVENT" {
            in_event = false;
            if !summary.is_empty() {
                events.push(json!({
                    "title": summary,
                    "start": dtstart,
                }));
            }
            if events.len() >= MAX_EVENTS {
                break;
            }
            continue;
        }
        if !in_event {
            continue;
        }
        if let Some(rest) = line.strip_prefix("SUMMARY:") {
            summary = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("DTSTART") {
            if let Some((_, val)) = rest.split_once(':') {
                dtstart = val.to_string();
            }
        }
    }
    events
}

pub async fn fetch_ics_events(client: &reqwest::Client, url: &str) -> Option<Vec<Value>> {
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().await.ok()?;
    Some(parse_ics_preview(&body))
}

/// Fetch upcoming *arr calendar entries. Content: `sonarr|https://host:8989|apikey` per line.
pub async fn fetch_arr_calendar_lines(client: &reqwest::Client, content: &str) -> Vec<Value> {
    let mut events = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() < 3 {
            continue;
        }
        let kind = parts[0].trim();
        let base = parts[1].trim().trim_end_matches('/');
        let key = parts[2].trim();
        if let Some(mut rows) = fetch_arr_calendar(client, kind, base, key).await {
            events.append(&mut rows);
        }
        if events.len() >= MAX_EVENTS {
            break;
        }
    }
    events.truncate(MAX_EVENTS);
    events
}

pub async fn fetch_arr_calendar(
    client: &reqwest::Client,
    kind: &str,
    base_url: &str,
    api_key: &str,
) -> Option<Vec<Value>> {
    let path = match kind {
        "radarr" => "/api/v3/calendar",
        "lidarr" => "/api/v1/calendar",
        "readarr" => "/api/v1/calendar",
        _ => "/api/v3/calendar", // sonarr default
    };
    let start = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let end = (chrono::Utc::now() + chrono::Duration::days(14))
        .format("%Y-%m-%d")
        .to_string();
    let url = format!("{base_url}{path}?start={start}&end={end}&unmonitored=false");
    let resp = client
        .get(&url)
        .header("X-Api-Key", api_key)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Vec<Value> = resp.json().await.ok()?;
    let mut out = Vec::new();
    for item in body {
        let title = item
            .get("title")
            .or_else(|| item.get("series").and_then(|s| s.get("title")))
            .and_then(|v| v.as_str())
            .unwrap_or("—");
        let start = item
            .get("airDate")
            .or_else(|| item.get("releaseDate"))
            .and_then(|v| v.as_str())
            .unwrap_or("—");
        out.push(json!({ "title": title, "start": start }));
        if out.len() >= MAX_EVENTS {
            break;
        }
    }
    Some(out)
}

pub fn format_datetime_widget() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%A, %d %B %Y — %H:%M UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vevent() {
        let ics = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nSUMMARY:Test Show\nDTSTART:20260101T120000Z\nEND:VEVENT\nEND:VCALENDAR";
        let events = parse_ics_preview(ics);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["title"], "Test Show");
    }
}
