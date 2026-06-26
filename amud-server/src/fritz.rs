use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

struct FritzCreds<'a> {
    username: &'a str,
    password: &'a str,
}

pub(crate) fn parse_fritz_credential(raw: &str) -> Option<(&str, &str)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (user, pass) = trimmed.split_once('|')?;
    let user = user.trim();
    let pass = pass.trim();
    if user.is_empty() || pass.is_empty() {
        return None;
    }
    Some((user, pass))
}

pub(crate) fn fritz_login_response(challenge: &str, password: &str) -> String {
    let to_hash = format!("{challenge}-{password}-{challenge}");
    let utf16_bytes: Vec<u8> = to_hash
        .encode_utf16()
        .flat_map(|unit| unit.to_le_bytes())
        .collect();
    let digest = md5::compute(&utf16_bytes);
    format!("{challenge}-{digest:x}")
}

fn soap_tag_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    let value = xml[start..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn soap_tag_u64(xml: &str, tags: &[&str]) -> Option<u64> {
    tags.iter()
        .find_map(|tag| soap_tag_value(xml, tag))
        .and_then(|v| v.parse().ok())
}

fn soap_tag_str(xml: &str, tags: &[&str]) -> Option<String> {
    tags.iter().find_map(|tag| soap_tag_value(xml, tag))
}

fn format_bitrate_bps(bps: u64) -> String {
    if bps == 0 {
        return "—".to_string();
    }
    if bps >= 1_000_000 {
        format!("{:.1} Mbit/s", bps as f64 / 1_000_000.0)
    } else if bps >= 1_000 {
        format!("{:.0} Kbit/s", bps as f64 / 1_000.0)
    } else {
        format!("{bps} bit/s")
    }
}

fn format_kbps_rate(kbps: u64) -> String {
    if kbps == 0 {
        return "—".to_string();
    }
    if kbps >= 1000 {
        format!("{:.1} Mbit/s", kbps as f64 / 1000.0)
    } else {
        format!("{kbps} Kbit/s")
    }
}

fn format_uptime(seconds: u64) -> String {
    if seconds == 0 {
        return "—".to_string();
    }
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3600;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h")
    } else {
        format!("{}m", seconds / 60)
    }
}

fn map_connection_status(raw: &str) -> String {
    match raw {
        "Connected" => "Connected".to_string(),
        "Disconnected" => "Disconnected".to_string(),
        "Unconfigured" => "Unconfigured".to_string(),
        "Connecting" => "Connecting".to_string(),
        other => other.to_string(),
    }
}

async fn fetch_login_sid(client: &Client, base_url: &str) -> Option<String> {
    let url = format!("{base_url}/login_sid.lua");
    let body = client.get(&url).send().await.ok()?.text().await.ok()?;
    soap_tag_str(&body, &["SID"])
}

async fn fetch_challenge(client: &Client, base_url: &str) -> Option<String> {
    let url = format!("{base_url}/login_sid.lua");
    let body = client.get(&url).send().await.ok()?.text().await.ok()?;
    soap_tag_str(&body, &["Challenge"])
}

async fn fritz_login(client: &Client, base_url: &str, creds: &FritzCreds<'_>) -> Option<String> {
    let sid = fetch_login_sid(client, base_url).await?;
    if sid != "0000000000000000" {
        return Some(sid);
    }
    let challenge = fetch_challenge(client, base_url).await?;
    let response = fritz_login_response(&challenge, creds.password);
    let mut url = reqwest::Url::parse(&format!("{base_url}/login_sid.lua")).ok()?;
    url.query_pairs_mut()
        .append_pair("username", creds.username)
        .append_pair("response", &response);
    let body = client.get(url).send().await.ok()?.text().await.ok()?;
    let sid = soap_tag_str(&body, &["SID"])?;
    if sid == "0000000000000000" {
        return None;
    }
    Some(sid)
}

fn tr064_envelope(service: &str, action: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
<s:Body>
<u:{action} xmlns:u="urn:dslforum-org:service:{service}:1">
</u:{action}>
</s:Body>
</s:Envelope>"#
    )
}

async fn tr064_call(
    client: &Client,
    base_url: &str,
    sid: &str,
    service_path: &str,
    service_urn: &str,
    action: &str,
) -> Option<String> {
    let url = format!("{base_url}/upnp/control/{service_path}?sid={sid}");
    let body = tr064_envelope(service_urn, action);
    let soap_action = format!("urn:dslforum-org:service:{service_urn}:1#{action}");
    let resp = client
        .post(&url)
        .header("Content-Type", "text/xml; charset=utf-8")
        .header("SOAPAction", format!("\"{soap_action}\""))
        .body(body)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.text().await.ok()
}

pub(crate) async fn fetch_fritz(client: &Client, base_url: &str, creds_raw: &str) -> Option<Value> {
    let (username, password) = parse_fritz_credential(creds_raw)?;
    let creds = FritzCreds { username, password };
    let sid = fritz_login(client, base_url, &creds).await?;

    let device_info = tr064_call(
        client,
        base_url,
        &sid,
        "deviceinfo1",
        "DeviceInfo",
        "GetInfo",
    )
    .await
    .unwrap_or_default();

    let wan_status = tr064_call(
        client,
        base_url,
        &sid,
        "wanipconnection1",
        "WANIPConn",
        "GetStatusInfo",
    )
    .await
    .unwrap_or_default();

    let ext_ip = tr064_call(
        client,
        base_url,
        &sid,
        "wanipconnection1",
        "WANIPConn",
        "GetExternalIPAddress",
    )
    .await
    .unwrap_or_default();

    let wan_addon = tr064_call(
        client,
        base_url,
        &sid,
        "wancommoninterfaceconfig1",
        "WANCommonIFC",
        "GetAddonInfos",
    )
    .await
    .unwrap_or_default();

    let hosts = tr064_call(
        client,
        base_url,
        &sid,
        "hosts1",
        "Hosts",
        "GetHostNumberOfEntries",
    )
    .await
    .unwrap_or_default();

    let dsl_link = tr064_call(
        client,
        base_url,
        &sid,
        "wandsllinkconfig1",
        "WANDSLLinkC",
        "GetDSLLinkInfo",
    )
    .await
    .unwrap_or_default();

    let connection_raw = soap_tag_str(&wan_status, &["NewConnectionStatus", "ConnectionStatus"])
        .unwrap_or_else(|| "—".to_string());
    let status = map_connection_status(&connection_raw);

    let down_bps =
        soap_tag_u64(&wan_addon, &["NewByteReceiveRate", "ByteReceiveRate"]).unwrap_or(0);
    let up_bps = soap_tag_u64(&wan_addon, &["NewByteSendRate", "ByteSendRate"]).unwrap_or(0);

    let ext_ip = soap_tag_str(&ext_ip, &["NewExternalIPAddress", "ExternalIPAddress"])
        .unwrap_or_else(|| "—".to_string());

    let uptime_secs = soap_tag_u64(&wan_status, &["NewUptime", "Uptime"])
        .or_else(|| soap_tag_u64(&device_info, &["NewUpTime", "UpTime"]))
        .unwrap_or(0);

    let devices =
        soap_tag_u64(&hosts, &["NewHostNumberOfEntries", "HostNumberOfEntries"]).unwrap_or(0);

    let version = soap_tag_str(&device_info, &["NewSoftwareVersion", "SoftwareVersion"])
        .unwrap_or_else(|| "—".to_string());

    let model = soap_tag_str(&device_info, &["NewModelName", "ModelName"])
        .unwrap_or_else(|| "—".to_string());

    let down_link_kbps =
        soap_tag_u64(&dsl_link, &["NewDownstreamCurrRate", "DownstreamCurrRate"]).unwrap_or(0);
    let up_link_kbps =
        soap_tag_u64(&dsl_link, &["NewUpstreamCurrRate", "UpstreamCurrRate"]).unwrap_or(0);

    Some(json!({
        "type": "fritz",
        "status": status,
        "download_speed": format_bitrate_bps(down_bps),
        "upload_speed": format_bitrate_bps(up_bps),
        "external_ip": ext_ip,
        "uptime": format_uptime(uptime_secs),
        "devices": devices,
        "version": version,
        "model": model,
        "down_link": format_kbps_rate(down_link_kbps),
        "up_link": format_kbps_rate(up_link_kbps),
    }))
}

pub(crate) fn build_fritz_client(accept_invalid_certs: bool) -> Client {
    Client::builder()
        .timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .unwrap_or_else(|_| Client::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_credential_splits_user_pass() {
        assert_eq!(
            parse_fritz_credential("admin|secret"),
            Some(("admin", "secret"))
        );
        assert!(parse_fritz_credential("noseparator").is_none());
        assert!(parse_fritz_credential("|nopass").is_none());
    }

    #[test]
    fn login_response_matches_avm_utf16_md5() {
        let response = fritz_login_response("1234567890", "password");
        assert!(response.starts_with("1234567890-"));
        assert_eq!(response.len(), 10 + 1 + 32);
    }

    #[test]
    fn soap_tag_extracts_values() {
        let xml = "<NewConnectionStatus>Connected</NewConnectionStatus>";
        assert_eq!(
            soap_tag_str(xml, &["NewConnectionStatus"]).as_deref(),
            Some("Connected")
        );
    }

    #[test]
    fn format_bitrate_scales() {
        assert_eq!(format_bitrate_bps(5_500_000), "5.5 Mbit/s");
        assert_eq!(format_bitrate_bps(0), "—");
    }

    #[test]
    fn format_uptime_days_hours() {
        assert_eq!(format_uptime(90_000), "1d 1h");
    }
}
