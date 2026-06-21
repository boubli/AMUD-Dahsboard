use super::imports::*;

pub async fn settings_page_handler(
    Extension(csp): Extension<CspNonce>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = match require_admin_session(&headers, &state.sessions) {
        Ok(s) => s,
        Err(_resp) => return Html("<h1>Access Denied: Admins Only</h1>".to_string()),
    };

    let settings = state.settings_cache.read().unwrap().clone();
    let branding = branding_from_settings(&settings);
    let app_name = branding.app_name.as_str();
    let tagline = branding
        .tagline
        .as_deref()
        .unwrap_or("Homelab Operations Cockpit");
    let custom_bg_url = branding.custom_bg_url.as_str();
    let app_logo = branding.app_logo.as_str();
    let accent_color = branding.accent_color.as_str();
    let glass_blur = branding.glass_blur.as_str();
    let glass_opacity = branding.glass_opacity.as_str();
    let bento_radius = branding.bento_radius.as_str();
    let grid_columns = branding.grid_columns.as_deref().unwrap_or("3");
    let weather_latitude = settings
        .get("weather_latitude")
        .map(|s| s.as_str())
        .unwrap_or("");
    let weather_longitude = settings
        .get("weather_longitude")
        .map(|s| s.as_str())
        .unwrap_or("");
    let jellyfin_url = settings
        .get("jellyfin_url")
        .map(|s| s.as_str())
        .unwrap_or("");
    let plex_url = settings.get("plex_url").map(|s| s.as_str()).unwrap_or("");
    let donate_enabled = settings
        .get("donate_enabled")
        .map(|s| s.as_str())
        .unwrap_or("1");
    let custom_css = settings.get("custom_css").map(|s| s.as_str()).unwrap_or("");
    let ha_url = settings.get("ha_url").map(|s| s.as_str()).unwrap_or("");
    let ha_token_placeholder = secret_field_placeholder(
        secret_setting_configured(&settings, "ha_token"),
        "Paste Home Assistant long-lived token",
    );
    let pve_api_token_placeholder = secret_field_placeholder(
        secret_setting_configured(&settings, "pve_api_token"),
        "PVEAPIToken=root@pam!tokenid=xxxx-xxxx-xxxx-xxxx",
    );
    let jellyfin_api_key_placeholder = secret_field_placeholder(
        secret_setting_configured(&settings, "jellyfin_api_key"),
        "Paste API key",
    );
    let plex_token_placeholder = secret_field_placeholder(
        secret_setting_configured(&settings, "plex_token"),
        "Paste Plex token",
    );
    let csrf_token = csrf_token_for_session(&headers, &state.sessions);
    let root_css = build_root_css(&branding);

    let settings_tmpl = include_str!("../../../ui/templates/settings.html");
    let username = session.username.as_str();
    let app_version = option_env!("GIT_TAG").unwrap_or(env!("CARGO_PKG_VERSION"));
    let proxmox_enabled =
        std::env::var("AMUD_ENABLE_PROXMOX").unwrap_or_else(|_| "false".to_string()) == "true";

    let mut result = settings_tmpl
        .replace("/* ROOT_CSS */", &root_css)
        .replace("{{app_name}}", app_name)
        .replace("{{app_version}}", app_version)
        .replace(
            "{{proxmox_enabled}}",
            if proxmox_enabled { "true" } else { "false" },
        )
        .replace("{{tagline}}", tagline)
        .replace("{{custom_bg_url}}", custom_bg_url);

    if app_logo.is_empty() {
        result = result.replace(
            "{{if app_logo}}style=\"background-image: url('{{app_logo}}');\"{{end}}",
            "",
        );
    } else {
        result = result.replace("{{if app_logo}}", "").replace("{{end}}", "");
    }
    result = result.replace("{{app_logo}}", &escape_html(app_logo));

    let result = result
        .replace("{{accent_color}}", accent_color)
        .replace("{{glass_blur_intensity}}", glass_blur)
        .replace("{{glass_opacity}}", glass_opacity)
        .replace("{{bento_radius}}", bento_radius)
        .replace(
            "{{eq_grid_2}}",
            crate::templates::theme_eq_attr(grid_columns, "2"),
        )
        .replace(
            "{{eq_grid_3}}",
            crate::templates::theme_eq_attr(grid_columns, "3"),
        )
        .replace(
            "{{eq_grid_4}}",
            crate::templates::theme_eq_attr(grid_columns, "4"),
        )
        .replace(
            "{{eq_grid_5}}",
            crate::templates::theme_eq_attr(grid_columns, "5"),
        )
        .replace("{{weather_latitude}}", weather_latitude)
        .replace("{{weather_longitude}}", weather_longitude)
        .replace(
            "{{pve_api_token_placeholder}}",
            &escape_html(&pve_api_token_placeholder),
        )
        .replace("{{jellyfin_url}}", jellyfin_url)
        .replace(
            "{{jellyfin_api_key_placeholder}}",
            &escape_html(&jellyfin_api_key_placeholder),
        )
        .replace("{{plex_url}}", plex_url)
        .replace(
            "{{plex_token_placeholder}}",
            &escape_html(&plex_token_placeholder),
        )
        .replace("{{ha_url}}", ha_url)
        .replace(
            "{{ha_token_placeholder}}",
            &escape_html(&ha_token_placeholder),
        )
        .replace("{{custom_css}}", custom_css)
        .replace("{{csrf_token}}", &csrf_token)
        .replace("{{username}}", &escape_html(username))
        .replace(
            "{{eq_donate_on}}",
            if donate_enabled == "1" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_donate_off}}",
            if donate_enabled != "1" {
                "selected"
            } else {
                ""
            },
        );
    let telemetry_public = settings
        .get("telemetry_public")
        .map(|s| s.as_str())
        .unwrap_or("0");
    let accept_invalid = settings
        .get("accept_invalid_certs")
        .map(|s| s.as_str())
        .unwrap_or("0");
    let result = result
        .replace(
            "{{eq_telemetry_on}}",
            if telemetry_public == "1" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_telemetry_off}}",
            if telemetry_public != "1" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_accept_invalid_certs_on}}",
            if accept_invalid == "1" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_accept_invalid_certs_off}}",
            if accept_invalid != "1" {
                "selected"
            } else {
                ""
            },
        )
        .replace(
            "{{eq_theme_dark}}",
            crate::templates::theme_eq_attr(&branding.theme_mode, "dark"),
        )
        .replace(
            "{{eq_theme_light}}",
            crate::templates::theme_eq_attr(&branding.theme_mode, "light"),
        )
        .replace("{{theme_mode}}", &escape_html(&branding.theme_mode));

    Html(apply_csp_nonce(result, &csp.0))
}
