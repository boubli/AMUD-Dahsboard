use std::collections::HashMap;

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

pub(crate) fn accent_glow_from_hex(accent_color: &str) -> String {
    if accent_color.starts_with('#') && accent_color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&accent_color[1..3], 16),
            u8::from_str_radix(&accent_color[3..5], 16),
            u8::from_str_radix(&accent_color[5..7], 16),
        ) {
            return format!("rgba({}, {}, {}, 0.15)", r, g, b);
        }
    }
    "rgba(56, 189, 248, 0.15)".to_string()
}

pub(crate) fn theme_eq_attr(current: &str, option: &str) -> &'static str {
    if current.eq_ignore_ascii_case(option) {
        "selected"
    } else {
        ""
    }
}

pub(crate) struct BrandingVars {
    pub app_name: String,
    pub tagline: Option<String>,
    pub custom_bg_url: String,
    pub app_logo: String,
    pub accent_color: String,
    pub glass_blur: String,
    pub glass_opacity: String,
    pub bento_radius: String,
    pub grid_columns: Option<String>,
    pub overlay_theme: String,
    pub custom_overlay_color: String,
}

pub(crate) fn branding_from_settings(settings: &HashMap<String, String>) -> BrandingVars {
    let raw_bg = settings
        .get("custom_bg_url")
        .map(|s| s.as_str())
        .unwrap_or("/static/wallpaper.png");
    let sanitized_bg = crate::settings::sanitize_setting_url(raw_bg);
    let custom_bg_url = if !sanitized_bg.is_empty() {
        sanitized_bg
    } else if raw_bg.is_empty()
        || raw_bg == "https://raw.githubusercontent.com/youssef-boubli/assets/main/dashboard-bg.jpg"
    {
        "/static/wallpaper.png".to_string()
    } else {
        raw_bg.to_string()
    };

    let grid_columns = settings
        .get("grid_columns")
        .or_else(|| settings.get("app_grid_columns"))
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|cols| (2..=5).contains(cols))
        .map(|cols| cols.to_string());

    BrandingVars {
        app_name: settings
            .get("app_name")
            .cloned()
            .unwrap_or_else(|| "AMUD".to_string()),
        tagline: settings.get("tagline").cloned(),
        custom_bg_url,
        app_logo: settings.get("app_logo").cloned().unwrap_or_default(),
        accent_color: settings
            .get("accent_color")
            .cloned()
            .unwrap_or_else(|| "#cf6427".to_string()),
        glass_blur: settings
            .get("glass_blur_intensity")
            .cloned()
            .unwrap_or_else(|| "16".to_string()),
        glass_opacity: settings
            .get("glass_opacity")
            .cloned()
            .unwrap_or_else(|| "0.45".to_string()),
        bento_radius: settings
            .get("bento_radius")
            .cloned()
            .unwrap_or_else(|| "16".to_string()),
        grid_columns,
        overlay_theme: settings
            .get("overlay_theme")
            .cloned()
            .unwrap_or_else(|| "cyber".to_string()),
        custom_overlay_color: settings
            .get("custom_overlay_color")
            .cloned()
            .unwrap_or_else(|| "#1a1a2e".to_string()),
    }
}

pub(crate) fn build_root_css(vars: &BrandingVars) -> String {
    let bg_url_style = if vars.custom_bg_url.is_empty() {
        String::new()
    } else {
        format!("--brand-bg-image: url('{}');", vars.custom_bg_url)
    };
    let logo_url_style = if vars.app_logo.is_empty() {
        String::new()
    } else {
        format!("--brand-logo-url: url('{}');", vars.app_logo)
    };
    let opacity_f: f64 = vars.glass_opacity.parse().unwrap_or(0.45);
    let accent_glow = accent_glow_from_hex(&vars.accent_color);
    let overlay_gradient =
        get_overlay_gradient(&vars.overlay_theme, Some(&vars.custom_overlay_color));
    let tagline = vars.tagline.as_deref().unwrap_or("");
    let grid_columns = vars.grid_columns.as_deref().unwrap_or("3");

    format!(
        r#"
            {}
            {}
            --brand-title: "{}";
            --brand-slogan: "{}";
            --accent-color: {};
            --accent-glow: {};
            --glass-blur-intensity: {}px;
            --glass-opacity: {};
            --radius-xl: {}px;
            --grid-cols: {};
            --bg-card: rgba(15, 20, 25, {});
            --brand-overlay-gradient: {};
        "#,
        bg_url_style,
        logo_url_style,
        vars.app_name,
        tagline,
        vars.accent_color,
        accent_glow,
        vars.glass_blur,
        vars.glass_opacity,
        vars.bento_radius,
        grid_columns,
        opacity_f,
        overlay_gradient
    )
}

pub(crate) fn apply_theme_placeholders(html: String, overlay_theme: &str) -> String {
    html.replace("{{eq_cyber}}", theme_eq_attr(overlay_theme, "cyber"))
        .replace("{{eq_aurora}}", theme_eq_attr(overlay_theme, "aurora"))
        .replace("{{eq_crimson}}", theme_eq_attr(overlay_theme, "crimson"))
        .replace("{{eq_sunset}}", theme_eq_attr(overlay_theme, "sunset"))
        .replace("{{eq_obsidian}}", theme_eq_attr(overlay_theme, "obsidian"))
        .replace("{{eq_custom}}", theme_eq_attr(overlay_theme, "custom"))
}
