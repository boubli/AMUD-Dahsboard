use crate::agent::pve_config_payload;
use crate::apps::{is_jellyfin_app, is_plex_app};
use crate::audit::{list_recent_audit, record_audit};
use crate::auth::{
    clear_failed_logins, csrf_forbidden_response, csrf_token_for_session, expired_session_cookie,
    generate_session_token, get_session, hash_password, login_rate_limited, now_epoch_secs,
    rate_limit_response, record_failed_login, require_admin_session, revoke_sessions_for_user,
    session_cookie, validate_csrf, verify_password, CspNonce,
};
use crate::db::{
    load_apps_from_db, refresh_settings_cache, secret_field_placeholder,
    secret_setting_configured, setting_value_or_existing, telemetry_public_enabled, with_db,
};
use crate::media::default_media_streams;
use crate::models::{
    ActionResult, App, AppState, FullTelemetry, MediaStream, NetworkTelemetry, PveTestResult,
    Session, Webhook,
};
use crate::settings::{
    allowed_setting_keys, sanitize_setting_url, setting_key_allowed, DONATION_LINKS,
    DONATION_MESSAGE, EXTRA_SETTING_KEYS, SECRET_SETTING_KEYS,
};
use crate::security::mask_webhook_url;
use crate::security::{client_ip, enforce_rate_limit, RateLimitConfig};
use crate::templates::{
    apply_theme_placeholders, branding_from_settings, build_root_css, escape_html, normalize_url,
    BrandingVars,
};
use crate::webhooks::send_webhook_notification;
use axum::{
    body::Body,
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        Extension, Multipart, Path, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use chrono;
use futures_util;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path as FilePath;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

fn apply_csp_nonce(html: String, nonce: &str) -> String {
    html.replace("{{csp_nonce}}", nonce)
}

fn check_api_rate_limit(
    state: &AppState,
    headers: &HeaderMap,
    bucket: &str,
    max: usize,
    window_secs: u64,
) -> Option<Response> {
    let key = format!("{}:{}", bucket, client_ip(headers));
    if !enforce_rate_limit(
        &state.api_rate_limits,
        &key,
        RateLimitConfig {
            max,
            window: Duration::from_secs(window_secs),
        },
    ) {
        Some(rate_limit_response())
    } else {
        None
    }
}

// Handlers
pub async fn dashboard_handler(
    Extension(csp): Extension<CspNonce>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);

    // Load Settings from in-memory cache (refreshed on save / startup)
    let settings = state.settings_cache.read().unwrap().clone();

    let branding = branding_from_settings(&settings);
    let app_name = branding.app_name.as_str();
    let tagline = branding.tagline.as_deref().unwrap_or("Homelab Operations Cockpit");
    let custom_bg_url = branding.custom_bg_url.as_str();
    let app_logo = branding.app_logo.as_str();
    let accent_color = branding.accent_color.as_str();
    let glass_blur = branding.glass_blur.as_str();
    let glass_opacity = branding.glass_opacity.as_str();
    let bento_radius = branding.bento_radius.as_str();
    let overlay_theme = branding.overlay_theme.as_str();
    let custom_overlay_color = branding.custom_overlay_color.as_str();
    let grid_columns = branding.grid_columns.as_deref().unwrap_or("3");
    let grid_columns_n: usize = grid_columns.parse().unwrap_or(3).clamp(2, 5);
    let weather_lat = settings
        .get("weather_latitude")
        .map(|s| s.as_str())
        .unwrap_or("");
    let weather_lon = settings
        .get("weather_longitude")
        .map(|s| s.as_str())
        .unwrap_or("");
    let is_admin = session.as_ref().map(|s| s.role == "Admin").unwrap_or(false);
    let telemetry_public = settings
        .get("telemetry_public")
        .map(|s| s.as_str())
        .unwrap_or("0");
    let csrf_token = csrf_token_for_session(&headers, &state.sessions);
    let csrf_attr = escape_html(&csrf_token);

    // Load Categories from DB for dropdown
    let mut db_categories = Vec::<(i64, String)>::new();
    {
        let db = state.db.lock().unwrap();
        let mut stmt = db
            .prepare("SELECT id, name FROM categories ORDER BY sort_order ASC, name ASC")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let id: i64 = row.get(0).unwrap();
            let name: String = row.get(1).unwrap();
            db_categories.push((id, name));
        }
    }
    let mut category_options_html = String::new();
    for (_id, cat_name) in &db_categories {
        category_options_html.push_str(&format!(
            r#"<option value="{}">{}</option>"#,
            escape_html(cat_name),
            escape_html(cat_name)
        ));
    }
    if category_options_html.is_empty() {
        category_options_html = r#"<option value="General">General</option>"#.to_string();
    }

    // Load Applications (non-blocking DB access)
    let apps_html;
    let apps = with_db(state.db.clone(), |db| load_apps_from_db(db)).await;

    if apps.is_empty() {
        apps_html = r#"
        <div class="glass-panel app-card" style="grid-column: span 3; text-align: center; padding: 3rem 1rem;">
            <p style="font-weight: 600; color: var(--text-secondary);">No services configured yet</p>
            <p style="font-size: 0.8rem; color: var(--text-muted); margin-top: 0.5rem;">Log in as Admin and click "Add App" to register your infrastructure.</p>
        </div>"#.to_string();
    } else {
        // Group cards into the configured number of dashboard columns.
        let mut cols = vec![String::new(); grid_columns_n];
        for (i, app) in apps.iter().enumerate() {
            let col_idx = i % grid_columns_n;

            // Resolve Built-in Brand Logo
            let lowercase_icon = app.icon.to_lowercase();
            let mut resolved_logo = String::new();
            if app.icon.starts_with("http") || app.icon.starts_with("/") {
                resolved_logo = app.icon.clone();
            } else if !lowercase_icon.is_empty() {
                let possible_paths = [
                    format!("ui/static/logos/{}.svg", lowercase_icon),
                    format!("ui/static/logos/{}.png", lowercase_icon),
                    format!("ui/static/logos/{}.jpg", lowercase_icon),
                    format!("ui/static/logos/{}.svg", lowercase_icon.replace(' ', "-")),
                    format!("ui/static/logos/{}.png", lowercase_icon.replace(' ', "-")),
                    format!("static/logos/{}.svg", lowercase_icon),
                    format!("static/logos/{}.png", lowercase_icon),
                ];
                for path in &possible_paths {
                    if std::path::Path::new(path).exists() {
                        let web_path = if path.starts_with("ui/") {
                            path["ui".len()..].to_string()
                        } else {
                            format!("/{}", path)
                        };
                        resolved_logo = web_path;
                        break;
                    }
                }
            }
            let brand_logo = if !resolved_logo.is_empty() {
                resolved_logo
            } else {
                match lowercase_icon.as_str() {
                    "plex" => "/static/logos/plex.svg".to_string(),
                    "jellyfin" => "/static/logos/jellyfin.svg".to_string(),
                    "proxmox" => "/static/logos/proxmox.svg".to_string(),
                    "portainer" => "/static/logos/portainer.svg".to_string(),
                    "home-assistant" | "homeassistant" => {
                        "/static/logos/home-assistant.svg".to_string()
                    }
                    "nextcloud" => "/static/logos/nextcloud.svg".to_string(),
                    "adguard" | "adguard-home" => "/static/logos/adguard-home.svg".to_string(),
                    "pihole" | "pi-hole" => "/static/logos/pi-hole.svg".to_string(),
                    "sonarr" => "/static/logos/sonarr.svg".to_string(),
                    "radarr" => "/static/logos/radarr.svg".to_string(),
                    "qbittorrent" => "/static/logos/qbittorrent.svg".to_string(),
                    "transmission" => "/static/logos/transmission.svg".to_string(),
                    "overseerr" => "/static/logos/overseerr.svg".to_string(),
                    "truenas" => "/static/logos/truenas.svg".to_string(),
                    "casaos" => "/static/logos/casaos.svg".to_string(),
                    _ => "/static/fallback.svg".to_string(),
                }
            };

            // Status indicator — populated by WebSocket after first paint
            let status_badge = r#"<span class="status-badge" style="background:rgba(255,255,255,0.05);color:var(--text-muted);border:1px solid var(--border-card);">CHECKING...</span>"#;

            // Category slug for filtering
            let cat_slug: String = app
                .category
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();

            // Build Sub-Metrics Grid
            let name_lower = app.name.to_lowercase();
            let sub_metrics = if session.is_some() {
                if name_lower.contains("proxmox") {
                    r#"
                    <div class="nested-metrics-grid cols-3">
                        <div class="metric-block">
                            <span class="metric-value">—</span>
                            <span class="metric-label">VMs</span>
                        </div>
                        <div class="metric-block">
                            <span class="metric-value">—</span>
                            <span class="metric-label">CPU</span>
                        </div>
                        <div class="metric-block">
                            <span class="metric-value">—</span>
                            <span class="metric-label">Mem</span>
                        </div>
                    </div>"#
                        .to_string()
                } else {
                    r#"
                    <div class="nested-metrics-grid">
                        <div class="metric-block">
                            <span class="metric-value">Bookmark</span>
                            <span class="metric-label">Type</span>
                        </div>
                        <div class="metric-block">
                            <span class="metric-value">Linked</span>
                            <span class="metric-label">Status</span>
                        </div>
                    </div>"#
                        .to_string()
                }
            } else {
                "".to_string()
            };

            let delete_btn = if is_admin {
                let app_json = serde_json::to_string(&app).unwrap_or_default();
                let escaped_json = app_json
                    .replace('&', "&amp;")
                    .replace('"', "&quot;")
                    .replace('\'', "&#39;");
                format!(
                    r#"
                    <div style="display: inline-flex; align-items: center; gap: 0.25rem;">
                        <button type="button" class="btn-edit-app" title="Edit application" data-app="{}" @click="editApp = JSON.parse($el.getAttribute('data-app')); editAppModalOpen = true;">
                            <i data-lucide="edit-2"></i>
                        </button>
                        <form action="/apps/delete" method="POST" style="margin: 0; display: inline-flex; align-items: center;">
                            <input type="hidden" name="id" value="{}">
                            <input type="hidden" name="csrf_token" value="{}">
                            <button type="submit" class="btn-delete-app" title="Delete application">
                                <i data-lucide="trash-2"></i>
                            </button>
                        </form>
                    </div>
                    "#,
                    escaped_json, app.id, csrf_attr
                )
            } else {
                "".to_string()
            };
            let ctrl_container = if is_admin {
                r#"
                <div class="container-controls" style="display: none; align-items: center; gap: 0.25rem;" data-id="" data-provider="">
                    <button type="button" class="btn-ctrl start" title="Start Container" @click="triggerContainerAction($el, 'start')">
                        <i data-lucide="circle-play" style="width:0.9rem; height:0.9rem;"></i>
                    </button>
                    <button type="button" class="btn-ctrl stop" title="Stop Container" @click="triggerContainerAction($el, 'stop')">
                        <i data-lucide="circle-stop" style="width:0.9rem; height:0.9rem;"></i>
                    </button>
                    <button type="button" class="btn-ctrl restart" title="Restart Container" @click="triggerContainerAction($el, 'restart')">
                        <i data-lucide="rotate-cw" style="width:0.9rem; height:0.9rem;"></i>
                    </button>
                </div>
                "#.to_string()
            } else {
                "".to_string()
            };

            let card = format!(
                r#"
                <div class="glass-panel app-card" data-app-name="{}" data-category="{}">
                    <div class="app-card-header">
                        <a href="{}" target="_blank" rel="noopener noreferrer" class="app-card-identity" style="text-decoration:none; color:inherit;">
                            <div class="app-card-icon">
                                <img src="{}" onerror="this.src='/static/fallback.svg'">
                            </div>
                            <div>
                                <h3 class="app-card-title">{}</h3>
                                <p class="app-card-desc">{}</p>
                            </div>
                        </a>
                        <div style="display: flex; align-items: center; gap: 0.5rem;" class="app-card-badges">
                            {}
                            {}
                            {}
                        </div>
                    </div>
                    {}
                </div>"#,
                escape_html(&name_lower),
                escape_html(&cat_slug),
                escape_html(&app.url),
                escape_html(&brand_logo),
                escape_html(&app.name),
                escape_html(&app.description),
                status_badge,
                ctrl_container,
                delete_btn,
                sub_metrics
            );
            cols[col_idx].push_str(&card);
        }

        apps_html = cols
            .into_iter()
            .map(|col| format!(r#"<div class="bento-column">{}</div>"#, col))
            .collect::<Vec<_>>()
            .join("");
    }

    // Auth actions buttons in topbar
    let auth_buttons = if let Some(ref sess) = session {
        let admin_settings_btn = if sess.role == "Admin" {
            r#"
            <button type="button" class="glass-panel btn-admin" @click="addAppModalOpen = true" style="padding:0.5rem 1rem; border-radius:8px; background:rgba(255,255,255,0.02); font-weight:600; cursor:pointer; font-size:0.82rem; display:inline-flex; align-items:center; gap:0.35rem; color:#fff; border:1px solid rgba(255,255,255,0.06);">
                <i data-lucide="plus" style="width:0.95rem; height:0.95rem;"></i> Add App
            </button>
            <a href="/admin/settings" class="glass-panel btn-admin" style="padding:0.5rem 1rem; border-radius:8px; background:rgba(255,255,255,0.02); font-weight:600; cursor:pointer; font-size:0.82rem; display:inline-flex; align-items:center; gap:0.35rem; color:#fff; border:1px solid rgba(255,255,255,0.06); text-decoration:none;">
                <i data-lucide="sliders-horizontal" style="width:0.95rem; height:0.95rem;"></i> Settings
            </a>
            "#
        } else {
            ""
        };
        format!(
            r#"
            {}
            <form action="/logout" method="POST" style="margin:0; display:inline-flex;">
                <input type="hidden" name="csrf_token" value="{}">
                <button type="submit" class="glass-panel" style="padding:0.5rem 1rem; border-radius:8px; font-weight:600; font-size:0.82rem; color:var(--text-secondary); border:1px solid rgba(255,255,255,0.06); display:inline-flex; align-items:center; gap:0.35rem; background:rgba(255,255,255,0.02); cursor:pointer;">
                    <i data-lucide="log-out" style="width:0.95rem; height:0.95rem;"></i> Sign Out ({})
                </button>
            </form>
            "#,
            admin_settings_btn, csrf_attr, escape_html(&sess.username)
        )
    } else {
        r#"
        <a href="/login" class="glass-panel" style="padding:0.5rem 1rem; border-radius:8px; font-weight:600; font-size:0.82rem; text-decoration:none; color:#fff; border:1px solid rgba(255,255,255,0.06); display:inline-flex; align-items:center; gap:0.35rem; background:var(--accent-glow);">
            <i data-lucide="key-round" style="width:0.95rem; height:0.95rem;"></i> Sign In
        </a>
        "#.to_string()
    };

    // Stream cards only when Plex / Jellyfin / Emby is registered in the app grid
    let has_plex = apps.iter().any(is_plex_app);
    let has_jellyfin = apps.iter().any(is_jellyfin_app);

    let mut streams_html = String::new();
    if session.is_some() && (has_plex || has_jellyfin) {
        let mut cards = String::new();
        if has_plex {
            cards.push_str(r#"
            <!-- Plex stream card -->
            <div class="glass-panel stream-card">
                <div class="stream-main">
                    <div class="stream-meta">
                        <div class="stream-icon">
                            <i data-lucide="play" style="color: #ff9900;"></i>
                        </div>
                        <div>
                            <h2 class="stream-text-title">Plex</h2>
                            <p class="stream-text-desc">Watch movies and TV shows.</p>
                        </div>
                    </div>
                    <span class="stream-status-badge" data-stream-app="plex" data-stream-service="plex">CHECKING...</span>
                </div>
                
                <div class="stream-player">
                    <div class="stream-controls-row">
                        <span class="stream-track-title" id="plex-track">No Active Streams</span>
                        <div style="display: flex; gap: 0.5rem; align-items: center;">
                            <button class="stream-play-btn"><i data-lucide="play" style="width:0.85rem; height:0.85rem;"></i></button>
                            <span id="plex-timer">-</span>
                        </div>
                    </div>
                    <div class="stream-progress-track">
                        <div class="stream-progress-fill" id="plex-progress" style="width: 0%;"></div>
                    </div>
                </div>
            </div>
            "#);
        }
        if has_jellyfin {
            cards.push_str(r#"
            <!-- Jellyfin/Emby stream card -->
            <div class="glass-panel stream-card">
                <div class="stream-main">
                    <div class="stream-meta">
                        <div class="stream-icon">
                            <i data-lucide="play-circle" style="color: #10b981;"></i>
                        </div>
                        <div>
                            <h2 class="stream-text-title">Jellyfin</h2>
                            <p class="stream-text-desc">Watch movies and TV shows.</p>
                        </div>
                    </div>
                    <span class="stream-status-badge" data-stream-app="jellyfin emby" data-stream-service="jellyfin">CHECKING...</span>
                </div>
                
                <div class="stream-player">
                    <div class="stream-controls-row">
                        <span class="stream-track-title" id="jellyfin-track" style="color: var(--text-muted);">No Active Streams</span>
                        <span id="jellyfin-timer">-</span>
                    </div>
                    <div class="stream-progress-track">
                        <div class="stream-progress-fill" id="jellyfin-progress" style="width: 0%;"></div>
                    </div>
                </div>
            </div>
            "#);
        }

        let cols_class = if has_plex && has_jellyfin {
            "streams-row"
        } else {
            "streams-row single-col"
        };
        streams_html = format!(r#"<section class="{}">{}</section>"#, cols_class, cards);
    }

    // Build category filter tabs HTML
    let mut categories = Vec::<String>::new();
    for app in apps.iter() {
        if !app.category.is_empty() && !categories.contains(&app.category) {
            categories.push(app.category.clone());
        }
    }

    let mut category_tabs_html = format!(
        r#"<button class="filter-tab active" @click="filterCategory('all', $el)">All <span class="filter-count">{}</span></button>"#,
        apps.len()
    );
    for cat in categories.iter() {
        let count = apps.iter().filter(|a| &a.category == cat).count();
        let cat_slug: String = cat
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        category_tabs_html.push_str(&format!(
            r#"<button class="filter-tab" @click="filterCategory('{}', $el)">{} <span class="filter-count">{}</span></button>"#,
            escape_html(&cat_slug), escape_html(cat), count
        ));
    }

    // Build Support / Donation card. The links are hardcoded to the AMUD author -
    // self-hosters can only enable or disable the card, not change the links.
    let donate_enabled = settings.get("donate_enabled").map(|s| s.as_str()).unwrap_or("1");
    let mut support_html = String::new();
    if donate_enabled == "1" {
        let mut links = String::new();
        for (url, label, icon) in DONATION_LINKS.iter() {
            links.push_str(&format!(
                r#"<a href="{}" target="_blank" rel="noopener noreferrer" class="support-link"><i data-lucide="{}" style="width:1rem; height:1rem;"></i> {}</a>"#,
                url, icon, label
            ));
        }
        support_html = format!(
            r#"<section class="support-section">
                <div class="glass-panel support-card">
                    <div class="support-head">
                        <i data-lucide="heart" style="color:var(--accent-color); width:1.2rem; height:1.2rem;"></i>
                        <h2>Support AMUD</h2>
                    </div>
                    <p class="support-msg">{}</p>
                    <div class="support-links">{}</div>
                </div>
            </section>"#,
            DONATION_MESSAGE, links
        );
    }

    // Build root_css style overrides
    let root_css = build_root_css(&branding);

    // Load templates
    let index_tmpl = include_str!("../../ui/templates/index.html");
    let username = session
        .as_ref()
        .map(|s| s.username.as_str())
        .unwrap_or("guest");
    let mut result = index_tmpl
        .replace("/* ROOT_CSS */", &root_css)
        .replace("{{app_name}}", app_name)
        .replace("{{tagline}}", tagline)
        .replace("{{custom_bg_url}}", custom_bg_url);

    if app_logo.is_empty() {
        result = result.replace("{{if app_logo}}style=\"background-image: url('{{app_logo}}');\"{{end}}", "");
    } else {
        result = result
            .replace("{{if app_logo}}", "")
            .replace("{{app_logo}}", app_logo)
            .replace("{{end}}", "");
    }

    let result = result
        .replace("{{accent_color}}", accent_color)
        .replace("{{glass_blur_intensity}}", glass_blur)
        .replace("{{glass_opacity}}", glass_opacity)
        .replace("{{bento_radius}}", bento_radius)
        .replace("<!-- APPS_GRID -->", &apps_html)
        .replace("<!-- STREAMS_ROW -->", &streams_html)
        .replace("<!-- CATEGORY_TABS -->", &category_tabs_html)
        .replace("<!-- SUPPORT_SECTION -->", &support_html)
        .replace("<!-- AUTH_BUTTONS -->", &auth_buttons)
        .replace("{{username}}", &escape_html(username))
        .replace("{{custom_overlay_color}}", custom_overlay_color)
        .replace("{{weather_latitude}}", weather_lat)
        .replace("{{weather_longitude}}", weather_lon)
        .replace("<!-- CATEGORY_OPTIONS -->", &category_options_html)
        .replace("{{csrf_token}}", &csrf_token)
        .replace("{{telemetry_public}}", telemetry_public)
        .replace(
            "{{is_admin}}",
            if is_admin { "true" } else { "false" },
        );
    let result = apply_theme_placeholders(result, overlay_theme);

    Html(apply_csp_nonce(result, &csp.0))
}

pub async fn login_page(
    Extension(csp): Extension<CspNonce>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let settings = state.settings_cache.read().unwrap().clone();
    let branding = branding_from_settings(&settings);
    let root_css = build_root_css(&BrandingVars {
        tagline: None,
        ..branding
    });

    let login_tmpl = include_str!("../../ui/templates/login.html");
    let result = login_tmpl.replace("/* ROOT_CSS */", &root_css);
    Html(apply_csp_nonce(result, &csp.0))
}

pub async fn login_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "login_ip", 30, 300) {
        return resp;
    }

    let username = form.get("username").cloned().unwrap_or_default().trim().to_string();
    let password = form.get("password").cloned().unwrap_or_default();

    if login_rate_limited(&state.login_attempts, &username) {
        return Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(axum::body::Body::from("Too many failed login attempts. Try again later."))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT password_hash, role FROM users WHERE username = ?")
        .unwrap();

    let auth_res = stmt.query_row(params![username.clone()], |row| {
        let pwhash: String = row.get(0).unwrap();
        let role: String = row.get(1).unwrap();
        let (verified, needs_rehash) = verify_password(&pwhash, &password);
        Ok((verified, needs_rehash, role))
    });

    if let Ok((true, needs_rehash, role)) = auth_res {
        if needs_rehash {
            let upgraded = hash_password(&password);
            db.execute(
                "UPDATE users SET password_hash = ? WHERE username = ?",
                params![upgraded, username],
            )
            .ok();
        }
        clear_failed_logins(&state.login_attempts, &username);
        let token = generate_session_token();
        let csrf_token = generate_session_token();

        state
            .sessions
            .write()
            .unwrap()
            .insert(
                token.clone(),
                Session {
                    username: username.clone(),
                    role: role.clone(),
                    expires_at_epoch: now_epoch_secs() + 86_400,
                    csrf_token,
                },
            );

        let cookie = session_cookie(&token);
        let must_change = db
            .query_row(
                "SELECT value FROM settings WHERE key = 'admin_must_change_password'",
                [],
                |row| row.get::<_, String>(0),
            )
            .map(|v| v == "1")
            .unwrap_or(false);
        let redirect_to = if must_change && role == "Admin" {
            "/admin/settings"
        } else {
            "/"
        };

        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::SET_COOKIE, cookie)
            .header(header::LOCATION, redirect_to)
            .body(axum::body::Body::empty())
            .unwrap()
    } else {
        // Keep missing-user and wrong-password timing closer by doing an Argon2id hash
        // even when no stored hash exists.
        let _ = hash_password(&password);
        record_failed_login(&state.login_attempts, &username);
        Redirect::to("/login").into_response()
    }
}

pub async fn logout_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    form: Option<Form<HashMap<String, String>>>,
) -> impl IntoResponse {
    if let Some(Form(form)) = form {
        if !validate_csrf(&headers, &state.sessions, Some(&form)) {
            return csrf_forbidden_response();
        }
    }
    if let Some(cookie_header) = headers.get("cookie").and_then(|c| c.to_str().ok()) {
        if let Some(token) = cookie_header
            .split(';')
            .map(|s| s.trim())
            .find(|s| s.starts_with("amud_session="))
            .map(|s| s["amud_session=".len()..].to_string())
        {
            state.sessions.write().unwrap().remove(&token);
        }
    }

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::SET_COOKIE, expired_session_cookie())
        .header(header::LOCATION, "/")
        .body(axum::body::Body::empty())
        .unwrap()
}

// WS upgrades handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    let is_guest = session.is_none();
    if is_guest {
        let public = {
            let db = state.db.lock().unwrap();
            telemetry_public_enabled(&db)
        };
        if !public {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("WebSocket requires authentication"))
                .unwrap();
        }
    }
    ws.on_upgrade(move |socket| handle_ws_session(socket, state, is_guest))
}

async fn handle_ws_session(mut socket: WebSocket, state: Arc<AppState>, is_guest: bool) {
    let rx_stream = state.latest_telemetry.clone();

    loop {
        // Stream telemetry packet every 3 seconds
        let system_metrics = if is_guest {
            crate::models::AgentTelemetry::default()
        } else {
            rx_stream.read().unwrap().clone()
        };
        let network = if is_guest {
            crate::models::NetworkTelemetry::default()
        } else {
            system_metrics.network.clone().unwrap_or_default()
        };
        let streams = if is_guest {
            HashMap::new()
        } else {
            state.media_streams.read().unwrap().clone()
        };
        let app_statuses = state.app_statuses.read().unwrap().clone();
        let agent_connected = if is_guest {
            false
        } else {
            *state.agent_connected.read().unwrap()
        };

        let payload = FullTelemetry {
            system: system_metrics,
            network,
            streams,
            app_statuses,
            agent_connected,
        };

        if let Ok(msg) = serde_json::to_string(&payload) {
            if socket.send(WsMessage::Text(msg)).await.is_err() {
                break;
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

// Settings Handler
pub async fn settings_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "settings", 20, 60) {
        return resp.into_response();
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => return Redirect::to("/admin/settings").into_response(),
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    let db = state.db.lock().unwrap();
    let mut new_token = None;
    let mut changed_keys = 0usize;
    for (key, val) in form {
        if key == "csrf_token"
            || key == "new_password"
            || key == "old_password"
            || key == "repeat_password"
            || key == "new_username"
        {
            continue;
        }
        if !setting_key_allowed(&key) {
            continue;
        }
        let value = if key == "custom_bg_url" || key == "app_logo" {
            sanitize_setting_url(&val)
        } else if SECRET_SETTING_KEYS.contains(&key.as_str()) {
            match setting_value_or_existing(&db, &key, &val) {
                Some(v) => v,
                None => continue,
            }
        } else {
            val
        };
        if key == "pve_api_token" {
            new_token = Some(value.clone());
        }
        db.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )
        .ok();
        changed_keys += 1;
    }
    refresh_settings_cache(&db, &state.settings_cache);
    if changed_keys > 0 {
        record_audit(
            &db,
            &admin_user,
            "settings_update",
            "settings",
            &format!("{} keys updated", changed_keys),
            &headers,
        );
    }
    drop(db);

    if let Some(token) = new_token {
        let config_payload = pve_config_payload(&token);
        if let Ok(mut serialized) = serde_json::to_vec(&config_payload) {
            serialized.push(b'\n');
            if let Some(tx) = &*state.agent_command_tx.lock().unwrap() {
                let _ = tx.send(String::from_utf8_lossy(&serialized).into_owned());
            }
        }
    }
    Redirect::to("/admin/settings").into_response()
}

// Proxmox VE API Token connection tester handler
pub async fn test_proxmox_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return resp;
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let form_token = form.get("pve_api_token").cloned().unwrap_or_default();
    if !form_token.trim().is_empty() {
        {
            let db = state.db.lock().unwrap();
            db.execute(
                "INSERT INTO settings (key, value) VALUES ('pve_api_token', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = ?1",
                params![form_token.trim()],
            )
            .ok();
            refresh_settings_cache(&db, &state.settings_cache);
        }
        let config_payload = pve_config_payload(form_token.trim());
        if let Ok(mut serialized) = serde_json::to_vec(&config_payload) {
            serialized.push(b'\n');
            if let Some(tx) = &*state.agent_command_tx.lock().unwrap() {
                let _ = tx.send(String::from_utf8_lossy(&serialized).into_owned());
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    // Clear any previous test response
    *state.pve_test_response.write().unwrap() = None;

    // Agent tests its cached/env token — token is not sent in this command (SEC-015)
    let cmd = serde_json::json!({ "action": "test_pve" });

    let mut success = false;
    let mut error = None;

    if let Ok(mut serialized) = serde_json::to_vec(&cmd) {
        serialized.push(b'\n');

        let sent = {
            if let Some(tx) = &*state.agent_command_tx.lock().unwrap() {
                tx.send(String::from_utf8_lossy(&serialized).into_owned())
                    .is_ok()
            } else {
                false
            }
        };

        if sent {
            // Wait for response up to 5 seconds
            let start = std::time::Instant::now();
            while start.elapsed() < std::time::Duration::from_secs(5) {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if let Some(res) = state.pve_test_response.read().unwrap().as_ref() {
                    success = res.success;
                    error = res.error.clone();
                    break;
                }
            }
            if !success && error.is_none() {
                error = Some("Connection test timed out waiting for agent response".to_string());
            }
        } else {
            error = Some("AMUD Agent is offline or not connected".to_string());
        }
    } else {
        error = Some("Failed to serialize test command".to_string());
    }

    let body = serde_json::json!({
        "success": success,
        "error": error
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&body).unwrap(),
        ))
        .unwrap()
}

// Secure Credentials Update Handler
pub async fn credentials_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "credentials", 5, 900) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let sess = match session {
        Some(ref s) if s.role == "Admin" => s,
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };

    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let old_password = form.get("old_password").cloned().unwrap_or_default();
    let new_password = form.get("new_password").cloned().unwrap_or_default();
    let new_username = form.get("new_username").cloned().unwrap_or_default();

    if old_password.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Old password is required"}"#,
            ))
            .unwrap();
    }

    let db = state.db.lock().unwrap();

    // Verify old password matches the current user's password
    let stored_hash: Result<String, _> = db
        .prepare("SELECT password_hash FROM users WHERE username = ?")
        .unwrap()
        .query_row(params![sess.username], |row| row.get(0));

    let old_needs_rehash = match stored_hash {
        Ok(ref h) => {
            let (verified, needs_rehash) = verify_password(h, &old_password);
            if verified {
                needs_rehash
            } else {
                return Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"error":"Old password is incorrect"}"#,
                    ))
                    .unwrap();
            }
        }
        _ => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"Old password is incorrect"}"#,
                ))
                .unwrap();
        }
    };

    // Update username if provided (check uniqueness first)
    let mut actual_username = sess.username.clone();
    let old_username = sess.username.clone();
    if !new_username.is_empty() && new_username != sess.username {
        let count: i64 = db
            .prepare("SELECT COUNT(*) FROM users WHERE username = ?")
            .unwrap()
            .query_row(params![new_username], |row| row.get(0))
            .unwrap_or(0);

        if count > 0 {
            return Response::builder()
                .status(StatusCode::CONFLICT)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"error":"Username is already taken"}"#,
                ))
                .unwrap();
        }

        if db
            .execute(
                "UPDATE users SET username = ? WHERE username = ?",
                params![new_username, sess.username],
            )
            .is_ok()
        {
            actual_username = new_username.clone();
        }
    }

    // Update password if provided
    if !new_password.is_empty() || old_needs_rehash {
        let new_hash = hash_password(if new_password.is_empty() {
            &old_password
        } else {
            &new_password
        });
        db.execute(
            "UPDATE users SET password_hash = ? WHERE username = ?",
            params![new_hash, actual_username],
        )
        .ok();
        db.execute(
            "INSERT INTO settings (key, value) VALUES ('admin_must_change_password', '0')
             ON CONFLICT(key) DO UPDATE SET value = '0'",
            [],
        )
        .ok();
        refresh_settings_cache(&db, &state.settings_cache);
        record_audit(
            &db,
            &actual_username,
            "credentials_change",
            &actual_username,
            if actual_username != old_username {
                "username and/or password updated"
            } else {
                "password updated"
            },
            &headers,
        );
    }
    drop(db);

    revoke_sessions_for_user(&state.sessions, &old_username);
    if actual_username != old_username {
        revoke_sessions_for_user(&state.sessions, &actual_username);
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"success":true}"#))
        .unwrap()
}

// Categories CRUD Handlers
pub async fn list_categories_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT id, name, color, sort_order FROM categories ORDER BY sort_order ASC, name ASC",
        )
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut categories = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let id: i64 = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();
        let color: String = row.get(2).unwrap();
        let sort_order: i64 = row.get(3).unwrap();
        categories.push(serde_json::json!({
            "id": id,
            "name": name,
            "color": color,
            "sort_order": sort_order
        }));
    }
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&categories).unwrap(),
        ))
        .unwrap()
}

pub async fn add_category_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let name = form.get("name").cloned().unwrap_or_default();
    let color = form
        .get("color")
        .cloned()
        .unwrap_or_else(|| "#64748b".to_string());
    let sort_order = form
        .get("sort_order")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);

    if name.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Category name is required"}"#,
            ))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    match db.execute(
        "INSERT INTO categories (name, color, sort_order) VALUES (?, ?, ?)",
        params![name, color, sort_order],
    ) {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::CONFLICT)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Category already exists"}"#,
            ))
            .unwrap(),
    }
}

pub async fn delete_category_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let db = state.db.lock().unwrap();
            db.execute("DELETE FROM categories WHERE id = ?", params![id])
                .ok();
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"success":true}"#))
        .unwrap()
}

pub async fn edit_category_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let name = form.get("name").cloned().unwrap_or_default();
            let color = form
                .get("color")
                .cloned()
                .unwrap_or_else(|| "#64748b".to_string());
            let sort_order = form
                .get("sort_order")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(0);
            if !name.is_empty() {
                let db = state.db.lock().unwrap();
                db.execute(
                    "UPDATE categories SET name = ?, color = ?, sort_order = ? WHERE id = ?",
                    params![name, color, sort_order, id],
                )
                .ok();
            }
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"success":true}"#))
        .unwrap()
}

// Add App Handler
pub async fn add_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return csrf_forbidden_response().into_response();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    let name = form.get("name").cloned().unwrap_or_default();
    let url = normalize_url(&form.get("url").cloned().unwrap_or_default());
    let icon = form.get("icon").cloned().unwrap_or_default();
    let category = form
        .get("category")
        .cloned()
        .unwrap_or_else(|| "General".to_string());
    let node_tag = form
        .get("node_tag")
        .cloned()
        .unwrap_or_else(|| "Local".to_string());
    let description = form.get("description").cloned().unwrap_or_default();

    if !name.is_empty() && !url.is_empty() {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO apps (name, url, icon, description, category, node_tag) VALUES (?, ?, ?, ?, ?, ?)",
            params![name, url, icon, description, category, node_tag],
        )
        .ok();
    }
    Redirect::to("/").into_response()
}

// Delete App Handler
pub async fn delete_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return csrf_forbidden_response().into_response();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let db = state.db.lock().unwrap();
            db.execute("DELETE FROM apps WHERE id = ?", params![id])
                .ok();
        }
    }
    Redirect::to("/").into_response()
}

// Edit App Handler
pub async fn edit_app_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return csrf_forbidden_response().into_response();
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    if let Some(id_str) = form.get("id") {
        if let Ok(id) = id_str.parse::<i64>() {
            let name = form.get("name").cloned().unwrap_or_default();
                let url = normalize_url(&form.get("url").cloned().unwrap_or_default());
                let icon = form.get("icon").cloned().unwrap_or_default();
                let category = form
                    .get("category")
                    .cloned()
                    .unwrap_or_else(|| "General".to_string());
                let node_tag = form
                    .get("node_tag")
                    .cloned()
                    .unwrap_or_else(|| "Local".to_string());
                let description = form.get("description").cloned().unwrap_or_default();

                if !name.is_empty() && !url.is_empty() {
                    let db = state.db.lock().unwrap();
                    db.execute(
                        "UPDATE apps SET name = ?, url = ?, icon = ?, description = ?, category = ?, node_tag = ? WHERE id = ?",
                        params![name, url, icon, description, category, node_tag, id],
                    )
                    .ok();
                }
            }
        }
    Redirect::to("/").into_response()
}

// Multipart File Uploader
pub async fn upload_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "upload", 10, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = session
        .as_ref()
        .filter(|s| s.role == "Admin")
        .map(|s| s.username.clone());
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Forbidden"))
            .unwrap();
    }
    if !validate_csrf(&headers, &state.sessions, None) {
        return csrf_forbidden_response();
    }

    let mut url_path = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" {
            let filename_orig = field.file_name().unwrap_or("image.png").to_string();
            let ext = FilePath::new(&filename_orig)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
                .to_lowercase();

            if ext != "png"
                && ext != "jpg"
                && ext != "jpeg"
                && ext != "ico"
                && ext != "gif"
            {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from("Invalid file extension"))
                    .unwrap();
            }

            let bytes = match field.bytes().await {
                Ok(b) => b,
                Err(_) => {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(axum::body::Body::from("Failed reading image bytes"))
                        .unwrap();
                }
            };

            if bytes.len() > 5 * 1024 * 1024 {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from("File size exceeds 5MB limit"))
                    .unwrap();
            }

            fs::create_dir_all("data/uploads").ok();
            let nano = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let filename = format!("{}.{}", nano, ext);
            let filepath = format!("data/uploads/{}", filename);

            if fs::write(&filepath, bytes).is_ok() {
                url_path = format!("/uploads/{}", filename);
            }
        }
    }

    if url_path.is_empty() {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(axum::body::Body::from("No image uploaded"))
            .unwrap()
    } else {
        if let Some(user) = admin_user {
            let db = state.db.lock().unwrap();
            record_audit(&db, &user, "upload", "image", &url_path, &headers);
        }
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(format!(
                r#"{{"url":"{}"}}"#,
                url_path
            )))
            .unwrap()
    }
}

// Action Trigger Handler for LXC / Docker containers
pub async fn app_action_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "container_action", 30, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session.as_ref() {
        Some(s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Unauthorized"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let provider = form.get("provider").cloned().unwrap_or_default();
    let id = form.get("id").cloned().unwrap_or_default();
    let action = form.get("action").cloned().unwrap_or_default();

    if provider.is_empty() || id.is_empty() || action.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Missing parameters"}"#))
            .unwrap();
    }

    let action_ok = match provider.as_str() {
        "lxc" => matches!(action.as_str(), "start" | "stop" | "restart" | "reboot" | "shutdown"),
        "docker" => matches!(action.as_str(), "start" | "stop" | "restart"),
        _ => false,
    };
    if !action_ok {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Invalid provider or action"}"#))
            .unwrap();
    }

    let request_id = generate_session_token();
    let cmd_value = serde_json::json!({
        "provider": provider,
        "id": id,
        "action": action,
        "request_id": request_id
    });
    let mut cmd = match serde_json::to_vec(&cmd_value) {
        Ok(v) => v,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"error":"Failed to encode command"}"#))
                .unwrap();
        }
    };
    cmd.push(b'\n');

    let agent_connected = *state.agent_connected.read().unwrap();
    let command_tx = state.agent_command_tx.lock().unwrap().clone();

    if !agent_connected {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Agent not connected"}"#))
            .unwrap();
    }

    let sent = if let Some(tx) = command_tx {
        tx.send(String::from_utf8_lossy(&cmd).into_owned()).is_ok()
    } else {
        false
    };

    if !sent {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"error":"Agent not connected"}"#))
            .unwrap();
    }

    {
        let db = state.db.lock().unwrap();
        record_audit(
            &db,
            &admin_user,
            "container_action",
            &format!("{}:{}", provider, id),
            &action,
            &headers,
        );
    }

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(12) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let result = {
            let mut results = state.action_results.write().unwrap();
            results.remove(&request_id)
        };
        if let Some(result) = result {
            if result.success {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"success":true}"#))
                    .unwrap();
            }
            let error = result.error.unwrap_or_else(|| "Action failed".to_string());
            let body = serde_json::json!({ "error": error });
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap();
        }
    }

    Response::builder()
        .status(StatusCode::GATEWAY_TIMEOUT)
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{"error":"Timed out waiting for agent to confirm action"}"#,
        ))
        .unwrap()
}

pub async fn serve_upload_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    if get_session(&headers, &state.sessions).is_none() {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Forbidden"))
            .unwrap();
    }
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap();
    }
    let path = format!("data/uploads/{}", filename);
    match fs::read(&path) {
        Ok(bytes) => {
            let content_type = match filename.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "ico" => "image/x-icon",
                _ => "application/octet-stream",
            };
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .body(Body::from(bytes))
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap(),
    }
}

// Webhook API handlers
pub async fn list_webhooks_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT id, name, url, event_types, is_active FROM webhooks ORDER BY id DESC")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut list = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let id: i64 = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();
        let url: String = row.get(2).unwrap();
        let event_types: String = row.get(3).unwrap();
        let is_active: i32 = row.get(4).unwrap();

        let masked_url = mask_webhook_url(&url);

        list.push(serde_json::json!({
            "id": id,
            "name": name,
            "url": masked_url,
            "event_types": event_types,
            "is_active": is_active
        }));
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&list).unwrap(),
        ))
        .unwrap()
}

pub async fn add_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "webhook", 20, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let name = form.get("name").cloned().unwrap_or_default().trim().to_string();
    let url = form.get("url").cloned().unwrap_or_default().trim().to_string();
    let event_types = form.get("event_types").cloned().unwrap_or_default().trim().to_string();
    let is_active = form.get("is_active").and_then(|v| v.parse::<i32>().ok()).unwrap_or(1);

    if name.is_empty() || url.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Name and URL are required"}"#))
            .unwrap();
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"URL must start with http:// or https://"}"#))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    match db.execute(
        "INSERT INTO webhooks (name, url, event_types, is_active) VALUES (?, ?, ?, ?)",
        params![name, url, event_types, is_active],
    ) {
        Ok(_) => {
            record_audit(
                &db,
                &admin_user,
                "webhook_create",
                &name,
                "webhook added",
                &headers,
            );
            Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(format!(r#"{{"error":"Database error: {}"}}"#, e)))
            .unwrap(),
    }
}

pub async fn edit_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "webhook", 20, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let id_str = form.get("id").cloned().unwrap_or_default();
    let id = match id_str.parse::<i64>() {
        Ok(v) => v,
        Err(_) => return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Invalid Webhook ID"}"#))
            .unwrap()
    };

    let name = form.get("name").cloned().unwrap_or_default().trim().to_string();
    let url_input = form.get("url").cloned().unwrap_or_default().trim().to_string();
    let event_types = form.get("event_types").cloned().unwrap_or_default().trim().to_string();
    let is_active = form.get("is_active").and_then(|v| v.parse::<i32>().ok()).unwrap_or(1);

    if name.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Name is required"}"#))
            .unwrap();
    }

    let db = state.db.lock().unwrap();
    let url = if url_input.is_empty() {
        match db.query_row(
            "SELECT url FROM webhooks WHERE id = ?",
            params![id],
            |row| row.get::<_, String>(0),
        ) {
            Ok(existing) => existing,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(r#"{"error":"Webhook not found"}"#))
                    .unwrap();
            }
        }
    } else if !url_input.starts_with("http://") && !url_input.starts_with("https://") {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"URL must start with http:// or https://"}"#))
            .unwrap();
    } else {
        url_input
    };

    match db.execute(
        "UPDATE webhooks SET name = ?, url = ?, event_types = ?, is_active = ? WHERE id = ?",
        params![name, url, event_types, is_active, id],
    ) {
        Ok(_) => {
            record_audit(
                &db,
                &admin_user,
                "webhook_update",
                &format!("id:{}", id),
                &name,
                &headers,
            );
            Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(format!(r#"{{"error":"Database error: {}"}}"#, e)))
            .unwrap(),
    }
}

pub async fn delete_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "webhook", 20, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let id_str = form.get("id").cloned().unwrap_or_default();
    if let Ok(id) = id_str.parse::<i64>() {
        let db = state.db.lock().unwrap();
        db.execute("DELETE FROM webhooks WHERE id = ?", params![id]).ok();
        record_audit(
            &db,
            &admin_user,
            "webhook_delete",
            &format!("id:{}", id),
            "webhook removed",
            &headers,
        );
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(r#"{"success":true}"#))
        .unwrap()
}

pub async fn test_webhook_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "webhook_test", 5, 60) {
        return resp;
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
                .unwrap();
        }
    };
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }

    let id_str = form.get("id").cloned().unwrap_or_default();
    let id = match id_str.parse::<i64>() {
        Ok(v) => v,
        Err(_) => return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Invalid ID"}"#))
            .unwrap()
    };

    let webhook = {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT name, url FROM webhooks WHERE id = ?").unwrap();
        stmt.query_row(params![id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).ok()
    };

    if let Some((name, url)) = webhook {
        {
            let db = state.db.lock().unwrap();
            record_audit(
                &db,
                &admin_user,
                "webhook_test",
                &format!("id:{}", id),
                &name,
                &headers,
            );
        }
        tokio::spawn(async move {
            send_webhook_notification(
                url,
                name,
                "test",
                "Test Container",
                999,
                "running",
                "Docker",
            )
            .await;
        });

        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"success":true}"#))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Webhook not found"}"#))
            .unwrap()
    }
}

// User Management Handlers
pub async fn list_users_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);
    if !session.map(|s| s.role == "Admin").unwrap_or(false) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"error":"Forbidden"}"#))
            .unwrap();
    }

    let mut users = Vec::new();
    {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare("SELECT id, username, role FROM users").unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let id: i64 = row.get(0).unwrap();
            let username: String = row.get(1).unwrap();
            let role: String = row.get(2).unwrap();
            users.push(serde_json::json!({ "id": id, "username": username, "role": role }));
        }
    }
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&users).unwrap(),
        ))
        .unwrap()
}

#[derive(Deserialize)]
pub(crate) struct AddUserForm {
    username: String,
    password: Option<String>,
    role: String,
}

pub async fn add_user_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<AddUserForm>,
) -> impl IntoResponse {
    if let Some(_resp) = check_api_rate_limit(&state, &headers, "user_mgmt", 10, 60) {
        return (StatusCode::TOO_MANY_REQUESTS, "Too many requests".to_string());
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => return (StatusCode::FORBIDDEN, "Forbidden".to_string()),
    };
    if !validate_csrf(&headers, &state.sessions, None) {
        return (StatusCode::FORBIDDEN, "Forbidden".to_string());
    }

    let pass = form.password.unwrap_or_default();
    if pass.is_empty() {
        return (StatusCode::BAD_REQUEST, "Password is required for new users.".to_string());
    }
    let p_hash = hash_password(&pass);
    let db = state.db.lock().unwrap();
    match db.execute(
        "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
        params![form.username.trim(), p_hash, form.role],
    ) {
        Ok(_) => {
            record_audit(
                &db,
                &admin_user,
                "user_create",
                form.username.trim(),
                &form.role,
                &headers,
            );
            (StatusCode::OK, "User added".to_string())
        }
        Err(_) => (StatusCode::BAD_REQUEST, "Username already exists or invalid.".to_string()),
    }
}

#[derive(Deserialize)]
pub(crate) struct EditUserForm {
    id: i64,
    username: String,
    password: Option<String>,
    role: String,
}

pub async fn edit_user_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<EditUserForm>,
) -> impl IntoResponse {
    if let Some(_resp) = check_api_rate_limit(&state, &headers, "user_mgmt", 10, 60) {
        return (StatusCode::TOO_MANY_REQUESTS, "Too many requests".to_string());
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => return (StatusCode::FORBIDDEN, "Forbidden".to_string()),
    };
    if !validate_csrf(&headers, &state.sessions, None) {
        return (StatusCode::FORBIDDEN, "Forbidden".to_string());
    }

    let db = state.db.lock().unwrap();
    let result = if let Some(pass) = form.password.filter(|p| !p.trim().is_empty()) {
        let p_hash = hash_password(&pass);
        match db.execute(
            "UPDATE users SET username = ?, password_hash = ?, role = ? WHERE id = ?",
            params![form.username.trim(), p_hash, form.role, form.id],
        ) {
            Ok(_) => Ok("password and profile updated"),
            Err(_) => Err("Update failed."),
        }
    } else {
        match db.execute(
            "UPDATE users SET username = ?, role = ? WHERE id = ?",
            params![form.username.trim(), form.role, form.id],
        ) {
            Ok(_) => Ok("profile updated"),
            Err(_) => Err("Update failed."),
        }
    };
    if let Ok(details) = result {
        record_audit(
            &db,
            &admin_user,
            "user_update",
            form.username.trim(),
            details,
            &headers,
        );
        (StatusCode::OK, "User updated".to_string())
    } else {
        (StatusCode::BAD_REQUEST, "Update failed.".to_string())
    }
}

#[derive(Deserialize)]
pub(crate) struct DeleteUserForm {
    id: i64,
}

pub async fn delete_user_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<DeleteUserForm>,
) -> impl IntoResponse {
    if let Some(_resp) = check_api_rate_limit(&state, &headers, "user_mgmt", 10, 60) {
        return (StatusCode::TOO_MANY_REQUESTS, "Too many requests".to_string());
    }

    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => return (StatusCode::FORBIDDEN, "Forbidden".to_string()),
    };
    if !validate_csrf(&headers, &state.sessions, None) {
        return (StatusCode::FORBIDDEN, "Forbidden".to_string());
    }

    let db = state.db.lock().unwrap();
    db.execute("DELETE FROM users WHERE id = ?", params![form.id]).ok();
    record_audit(
        &db,
        &admin_user,
        "user_delete",
        &format!("id:{}", form.id),
        "user removed",
        &headers,
    );
    (StatusCode::OK, "Deleted".to_string())
}

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
    let tagline = branding.tagline.as_deref().unwrap_or("Homelab Operations Cockpit");
    let custom_bg_url = branding.custom_bg_url.as_str();
    let app_logo = branding.app_logo.as_str();
    let accent_color = branding.accent_color.as_str();
    let glass_blur = branding.glass_blur.as_str();
    let glass_opacity = branding.glass_opacity.as_str();
    let bento_radius = branding.bento_radius.as_str();
    let grid_columns = branding.grid_columns.as_deref().unwrap_or("3");
    let overlay_theme = branding.overlay_theme.as_str();
    let custom_overlay_color = branding.custom_overlay_color.as_str();
    let weather_latitude = settings.get("weather_latitude").map(|s| s.as_str()).unwrap_or("");
    let weather_longitude = settings.get("weather_longitude").map(|s| s.as_str()).unwrap_or("");
    let jellyfin_url = settings.get("jellyfin_url").map(|s| s.as_str()).unwrap_or("");
    let plex_url = settings.get("plex_url").map(|s| s.as_str()).unwrap_or("");
    let donate_enabled = settings.get("donate_enabled").map(|s| s.as_str()).unwrap_or("1");
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

    let settings_tmpl = include_str!("../../ui/templates/settings.html");
    let username = session.username.as_str();
    let mut result = settings_tmpl
        .replace("/* ROOT_CSS */", &root_css)
        .replace("{{app_name}}", app_name)
        .replace("{{tagline}}", tagline)
        .replace("{{custom_bg_url}}", custom_bg_url);

    if app_logo.is_empty() {
        result = result.replace("{{if app_logo}}style=\"background-image: url('{{app_logo}}');\"{{end}}", "");
    } else {
        result = result
            .replace("{{if app_logo}}", "")
            .replace("{{app_logo}}", app_logo)
            .replace("{{end}}", "");
    }

    let result = result
        .replace("{{accent_color}}", accent_color)
        .replace("{{glass_blur_intensity}}", glass_blur)
        .replace("{{glass_opacity}}", glass_opacity)
        .replace("{{bento_radius}}", bento_radius)
        .replace("{{eq_grid_2}}", crate::templates::theme_eq_attr(grid_columns, "2"))
        .replace("{{eq_grid_3}}", crate::templates::theme_eq_attr(grid_columns, "3"))
        .replace("{{eq_grid_4}}", crate::templates::theme_eq_attr(grid_columns, "4"))
        .replace("{{eq_grid_5}}", crate::templates::theme_eq_attr(grid_columns, "5"))
        .replace("{{weather_latitude}}", weather_latitude)
        .replace("{{weather_longitude}}", weather_longitude)
        .replace("{{pve_api_token_placeholder}}", &escape_html(&pve_api_token_placeholder))
        .replace("{{jellyfin_url}}", jellyfin_url)
        .replace("{{jellyfin_api_key_placeholder}}", &escape_html(&jellyfin_api_key_placeholder))
        .replace("{{plex_url}}", plex_url)
        .replace("{{plex_token_placeholder}}", &escape_html(&plex_token_placeholder))
        .replace("{{csrf_token}}", &csrf_token)
        .replace("{{username}}", &escape_html(username))
        .replace("{{custom_overlay_color}}", custom_overlay_color)
        .replace("{{eq_donate_on}}", if donate_enabled == "1" { "selected" } else { "" })
        .replace("{{eq_donate_off}}", if donate_enabled != "1" { "selected" } else { "" });
    let result = apply_theme_placeholders(result, overlay_theme);

    Html(apply_csp_nonce(result, &csp.0))
}

pub async fn list_audit_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return resp;
    }

    let db = state.db.lock().unwrap();
    let entries = list_recent_audit(&db, 200);
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())))
        .unwrap()
}
