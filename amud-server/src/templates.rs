// Escape user-controlled text before injecting it into HTML.
pub(crate) fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

// Force a safe scheme on user-supplied URLs to neutralize javascript:/data: vectors.
pub(crate) fn normalize_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lower = trimmed.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    }
}

pub(crate) fn get_overlay_gradient(theme: &str, custom_color: Option<&str>) -> String {
    match theme.to_lowercase().as_str() {
        "aurora" => "linear-gradient(135deg, rgba(4, 15, 15, 0.88) 0%, rgba(6, 24, 20, 0.82) 100%)"
            .to_string(),
        "crimson" => {
            "linear-gradient(135deg, rgba(18, 8, 8, 0.88) 0%, rgba(12, 10, 15, 0.82) 100%)"
                .to_string()
        }
        "obsidian" => {
            "linear-gradient(135deg, rgba(10, 10, 12, 0.92) 0%, rgba(15, 15, 18, 0.88) 100%)"
                .to_string()
        }
        "sunset" => "linear-gradient(135deg, rgba(20, 8, 12, 0.88) 0%, rgba(8, 10, 20, 0.82) 100%)"
            .to_string(),
        "custom" => {
            if let Some(hex) = custom_color {
                if hex.starts_with('#') && hex.len() == 7 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[1..3], 16),
                        u8::from_str_radix(&hex[3..5], 16),
                        u8::from_str_radix(&hex[5..7], 16),
                    ) {
                        return format!(
                            "linear-gradient(135deg, rgba({}, {}, {}, 0.88) 0%, rgba({}, {}, {}, 0.82) 100%)",
                            r / 2, g / 2, b / 2, r / 3, g / 3, b / 3
                        );
                    }
                }
            }
            "linear-gradient(135deg, rgba(8, 10, 18, 0.85) 0%, rgba(15, 10, 25, 0.8) 100%)"
                .to_string()
        }
        _ => "linear-gradient(135deg, rgba(8, 10, 18, 0.85) 0%, rgba(15, 10, 25, 0.8) 100%)"
            .to_string(),
    }
}
