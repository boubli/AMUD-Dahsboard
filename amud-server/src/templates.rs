use std::collections::HashMap;

fn escape_css_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' | '"' | '\'' => {
                out.push('\\');
                out.push(c);
            }
            '\n' | '\r' => {}
            _ => out.push(c),
        }
    }
    out
}

pub(crate) fn safe_css_url(raw: &str) -> String {
    let sanitized = crate::settings::sanitize_setting_url(raw);
    if sanitized.is_empty() {
        return String::new();
    }
    escape_css_string(&sanitized)
}

pub(crate) fn safe_accent_hex(raw: &str) -> String {
    if raw.starts_with('#')
        && raw.len() == 7
        && (1..7).all(|i| raw.as_bytes()[i].is_ascii_hexdigit())
    {
        return raw.to_string();
    }
    "#cf6427".to_string()
}

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

pub(crate) const DEFAULT_OVERLAY_GRADIENT: &str =
    "linear-gradient(135deg, rgba(8, 10, 18, 0.85) 0%, rgba(15, 10, 25, 0.8) 100%)";

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
    pub wallpaper_overlay_strength: String,
    pub bento_radius: String,
    pub grid_columns: Option<String>,
    pub theme_mode: String,
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
        app_logo: settings
            .get("app_logo")
            .map(|s| crate::settings::sanitize_setting_url(s))
            .filter(|s| !s.is_empty())
            .unwrap_or_default(),
        accent_color: safe_accent_hex(
            settings
                .get("accent_color")
                .map(|s| s.as_str())
                .unwrap_or("#cf6427"),
        ),
        glass_blur: settings
            .get("glass_blur_intensity")
            .cloned()
            .unwrap_or_else(|| "16".to_string()),
        glass_opacity: settings
            .get("glass_opacity")
            .cloned()
            .unwrap_or_else(|| "0.45".to_string()),
        wallpaper_overlay_strength: settings
            .get("wallpaper_overlay_strength")
            .map(|s| crate::settings::sanitize_wallpaper_overlay_strength(s))
            .unwrap_or_else(|| "0.85".to_string()),
        bento_radius: settings
            .get("bento_radius")
            .cloned()
            .unwrap_or_else(|| "16".to_string()),
        grid_columns,
        theme_mode: settings
            .get("theme_mode")
            .cloned()
            .unwrap_or_else(|| "dark".to_string()),
    }
}

pub(crate) fn build_root_css(vars: &BrandingVars) -> String {
    let bg_url = safe_css_url(&vars.custom_bg_url);
    let bg_url_style = if bg_url.is_empty() {
        String::new()
    } else {
        format!("--brand-bg-image: url('{}');", bg_url)
    };
    let logo_url = safe_css_url(&vars.app_logo);
    let logo_url_style = if logo_url.is_empty() {
        String::new()
    } else {
        format!("--brand-logo-url: url('{}');", logo_url)
    };
    let opacity_f: f64 = vars.glass_opacity.parse().unwrap_or(0.45);
    let overlay_strength = vars.wallpaper_overlay_strength.as_str();
    let accent_glow = accent_glow_from_hex(&vars.accent_color);
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
            --wallpaper-overlay-strength: {};
            --radius-xl: {}px;
            --grid-cols: {};
            --bento-row-height: 8.75rem;
            --bg-card: rgba(var(--theme-card-r, 15), var(--theme-card-g, 20), var(--theme-card-b, 25), {});
            --brand-overlay-gradient: {};
        "#,
        bg_url_style,
        logo_url_style,
        escape_css_string(&vars.app_name),
        escape_css_string(tagline),
        safe_accent_hex(&vars.accent_color),
        accent_glow,
        vars.glass_blur,
        vars.glass_opacity,
        overlay_strength,
        vars.bento_radius,
        grid_columns,
        opacity_f,
        DEFAULT_OVERLAY_GRADIENT
    )
}

pub(crate) struct BrandingRenderOptions<'a> {
    pub branding: &'a BrandingVars,
    pub custom_css: &'a str,
    pub default_tagline: &'a str,
}

/// Shared placeholders for guest-facing pages (login, etc.) that should match dashboard branding.
pub(crate) fn apply_shared_branding(mut html: String, opts: &BrandingRenderOptions<'_>) -> String {
    let branding = opts.branding;
    let root_css = build_root_css(branding);
    let safe_app_name = escape_html(&branding.app_name);
    let tagline = branding
        .tagline
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(opts.default_tagline);
    let safe_tagline = escape_html(tagline);
    let safe_accent = escape_html(&branding.accent_color);
    let safe_theme = escape_html(&branding.theme_mode);
    let safe_app_logo_css = safe_css_url(&branding.app_logo);
    let favicon_url = if branding.app_logo.is_empty() {
        "/static/AMUD-logo.png".to_string()
    } else {
        escape_html(&branding.app_logo)
    };

    html = html.replace("/* ROOT_CSS */", &root_css);
    html = html
        .replace("{{app_name}}", &safe_app_name)
        .replace("{{tagline}}", &safe_tagline)
        .replace("{{accent_color}}", &safe_accent)
        .replace("{{theme_mode}}", &safe_theme)
        .replace("{{custom_css}}", opts.custom_css)
        .replace("{{favicon_url}}", &favicon_url);

    if branding.app_logo.is_empty() {
        html = html.replace(
            "{{if app_logo}}style=\"background-image: url('{{app_logo}}');\"{{end}}",
            "",
        );
    } else {
        html = html
            .replace("{{if app_logo}}", "")
            .replace("{{app_logo}}", &safe_app_logo_css)
            .replace("{{end}}", "");
    }

    let bg_url = &branding.custom_bg_url;
    let is_video_bg = bg_url.ends_with(".mp4") || bg_url.ends_with(".webm");
    let video_bg_element = if is_video_bg {
        format!(
            r#"<video class="video-bg" autoplay muted loop playsinline><source src="{}" type="video/{}"></video>"#,
            escape_html(bg_url),
            if bg_url.ends_with(".webm") {
                "webm"
            } else {
                "mp4"
            }
        )
    } else {
        String::new()
    };
    html.replace(
        "{{video_bg_class}}",
        if is_video_bg { "has-video-bg" } else { "" },
    )
    .replace("{{video_bg_element}}", &video_bg_element)
}

/// JSON config for client-side theme scheduler (`theme-scheduler.js`).
pub(crate) fn build_theme_scheduler_json(
    settings: &std::collections::HashMap<String, String>,
    base_mode: &str,
) -> String {
    use crate::settings::{sanitize_theme_scheduler, sanitize_time_hhmm};
    use serde_json::json;

    let scheduler = sanitize_theme_scheduler(
        settings
            .get("theme_scheduler")
            .map(|s| s.as_str())
            .unwrap_or("off"),
    );
    let light_at = sanitize_time_hhmm(
        settings
            .get("theme_light_at")
            .map(|s| s.as_str())
            .unwrap_or("07:00"),
        "07:00",
    );
    let dark_at = sanitize_time_hhmm(
        settings
            .get("theme_dark_at")
            .map(|s| s.as_str())
            .unwrap_or("19:00"),
        "19:00",
    );
    let lat = settings
        .get("weather_latitude")
        .map(|s| s.as_str())
        .unwrap_or("");
    let lon = settings
        .get("weather_longitude")
        .map(|s| s.as_str())
        .unwrap_or("");

    serde_json::to_string(&json!({
        "scheduler": scheduler,
        "lightAt": light_at,
        "darkAt": dark_at,
        "baseMode": if base_mode == "light" { "light" } else { "dark" },
        "lat": lat,
        "lon": lon,
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_root_css_uses_default_overlay() {
        let vars = branding_from_settings(&HashMap::new());
        let css = build_root_css(&vars);
        assert!(css.contains(DEFAULT_OVERLAY_GRADIENT));
        assert!(!css.contains("overlay_theme"));
    }

    #[test]
    fn apply_shared_branding_injects_theme_logo_and_custom_css() {
        let mut settings = HashMap::new();
        settings.insert("app_name".to_string(), "My Lab".to_string());
        settings.insert("accent_color".to_string(), "#ff00aa".to_string());
        settings.insert("theme_mode".to_string(), "light".to_string());
        settings.insert(
            "app_logo".to_string(),
            "/static/custom-logo.png".to_string(),
        );
        settings.insert(
            "custom_css".to_string(),
            ".btn-primary { background: hotpink; }".to_string(),
        );
        let branding = branding_from_settings(&settings);
        let html = apply_shared_branding(
            r#"<html data-theme="{{theme_mode}}"><style>:root { /* ROOT_CSS */ }</style><style id="x">{{custom_css}}</style><div class="brand-logo" {{if app_logo}}style="background-image: url('{{app_logo}}');"{{end}}></div>"#.to_string(),
            &BrandingRenderOptions {
                branding: &branding,
                custom_css: ".btn-primary { background: hotpink; }",
                default_tagline: "Sign in",
            },
        );
        assert!(html.contains(r#"data-theme="light""#));
        assert!(html.contains(".btn-primary { background: hotpink; }"));
        assert!(html.contains("url('/static/custom-logo.png')"));
        assert!(html.contains("--accent-color: #ff00aa"));
    }
}
