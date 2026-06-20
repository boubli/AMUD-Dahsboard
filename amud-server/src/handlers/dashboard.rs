use super::imports::*;
use crate::models::App;

pub async fn dashboard_handler(
    Extension(csp): Extension<CspNonce>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let session = get_session(&headers, &state.sessions);

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
    let hide_telemetry = match &session {
        None => telemetry_public != "1",
        Some(s) if s.role == "Guest" => telemetry_public != "1",
        _ => false,
    };
    let logo_manifest = state.logo_manifest.clone();
    let custom_css = settings.get("custom_css").map(|s| s.as_str()).unwrap_or("");
    let csrf_token = csrf_token_for_session(&headers, &state.sessions);
    let csrf_attr = escape_html(&csrf_token);

    let db_categories = with_db(state.db.clone(), load_categories).await;
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

    let apps = with_db(state.db.clone(), load_apps_from_db).await;
    let wol_devices = with_db(state.db.clone(), load_wol_devices_from_db).await;
    let app_names_json = serde_json::to_string(
        &apps
            .iter()
            .map(|app| app.name.to_lowercase())
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".to_string());

    let apps_html = render_apps_grid(
        &apps,
        is_admin,
        &csrf_token,
        &csrf_attr,
        &session,
        grid_columns_n,
        &logo_manifest,
    );

    let wol_html = render_wol_devices(&wol_devices, is_admin, &csrf_attr);

    let auth_buttons = render_auth_buttons(&session, &csrf_attr);
    let streams_html = render_streams(&apps, &session);
    let category_tabs_html = render_category_tabs(&apps);
    let support_html = render_support_section(&settings);

    let root_css = build_root_css(&branding);

    let index_tmpl = include_str!("../../../ui/templates/index.html");
    let username = session
        .as_ref()
        .map(|s| s.username.as_str())
        .unwrap_or("guest");
    let proxmox_enabled =
        std::env::var("AMUD_ENABLE_PROXMOX").unwrap_or_else(|_| "false".to_string()) == "true";

    let mut result = index_tmpl
        .replace("/* ROOT_CSS */", &root_css)
        .replace("{{app_name}}", app_name)
        .replace("{{tagline}}", tagline)
        .replace(
            "{{proxmox_enabled}}",
            if proxmox_enabled { "true" } else { "false" },
        )
        .replace("{{custom_bg_url}}", custom_bg_url);

    if app_logo.is_empty() {
        result = result.replace(
            "{{if app_logo}}style=\"background-image: url('{{app_logo}}');\"{{end}}",
            "",
        );
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
        .replace("<!-- WOL_SECTION -->", &wol_html)
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
            "{{hide_telemetry}}",
            if hide_telemetry { "true" } else { "false" },
        )
        .replace("{{custom_css}}", custom_css)
        .replace("{{app_names_json}}", &app_names_json)
        .replace("{{is_admin}}", if is_admin { "true" } else { "false" });
    let result = apply_theme_placeholders(result, overlay_theme);

    Html(apply_csp_nonce(result, &csp.0))
}

fn render_apps_grid(
    apps: &[App],
    is_admin: bool,
    csrf_token: &str,
    csrf_attr: &str,
    session: &Option<Session>,
    grid_columns_n: usize,
    logo_manifest: &HashMap<String, String>,
) -> String {
    if apps.is_empty() {
        return r#"
        <div class="glass-panel app-card" style="grid-column: span 3; text-align: center; padding: 3rem 1rem;">
            <p style="font-weight: 600; color: var(--text-secondary);">No services configured yet</p>
            <p style="font-size: 0.8rem; color: var(--text-muted); margin-top: 0.5rem;">Log in as Admin and click "Add App" to register your infrastructure.</p>
        </div>"#.to_string();
    }

    let mut cols = vec![String::new(); grid_columns_n];
    for (i, app) in apps.iter().enumerate() {
        let col_idx = i % grid_columns_n;

        let lowercase_icon = app.icon.to_lowercase();
        let resolved_logo = resolve_logo_from_manifest(&app.icon, logo_manifest);
        let brand_logo = if !resolved_logo.is_empty() {
            resolved_logo
        } else if !lowercase_icon.is_empty() {
            fallback_brand_logo(&lowercase_icon)
        } else {
            "/static/fallback.svg".to_string()
        };

        let status_title = if is_admin {
            "Waiting for container or URL health status"
        } else {
            "Public availability check"
        };
        let status_badge = format!(
            r#"<span class="status-badge status-checking" title="{}" aria-label="{}" data-last-status="">CHECKING...</span>"#,
            status_title, status_title
        );

        let cat_slug: String = app
            .category
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();

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
            } else if name_lower.contains("home") && name_lower.contains("assistant") {
                r#"
                <div class="nested-metrics-grid cols-3" id="ha-metrics-grid">
                    <div class="metric-block">
                        <span class="metric-value" id="ha-lights">—</span>
                        <span class="metric-label">Lights</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value" id="ha-switches">—</span>
                        <span class="metric-label">Switches</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value" id="ha-temp">—</span>
                        <span class="metric-label">Avg Temp</span>
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
            let mut app_for_json = app.clone();
            if !app_for_json.api_key.is_empty() {
                app_for_json.api_key = "Configured — leave blank to keep unchanged".to_string();
            }
            let app_json = serde_json::to_string(&app_for_json).unwrap_or_default();
            let escaped_json = app_json
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('\'', "&#39;");
            format!(
                r#"
                <div style="display: inline-flex; align-items: center; gap: 0.25rem;">
                    <button type="button" class="btn-edit-app" title="Edit application" data-app="{}" @click="editApp = JSON.parse($el.getAttribute('data-app')); window.editingAppOriginalName = (editApp.name || '').toLowerCase(); editAppModalOpen = true; setTimeout(checkDuplicateAppName, 0);">
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

        let mut integration_widget = String::new();
        if !app.integration_type.is_empty() {
            integration_widget = format!(
                r#"
                <div class="integration-widget" x-show="integrationData">
                    <template x-if="integrationData.type === 'pihole' || integrationData.type === 'adguard'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.ads_blocked_today"></span>
                                <span class="metric-label">Ads Blocked</span>
                            </div>
                            <div class="metric-block" style="flex-direction: row; justify-content: center; gap: 0.5rem; align-items: center;">
                                <span class="metric-value" style="font-size: 0.8rem; text-transform: uppercase;" x-text="integrationData.status"></span>
                                <button class="btn btn-secondary" style="padding: 0.2rem 0.5rem; font-size: 0.7rem; height: auto;" @click="fetch('/api/apps/{}/integration/action', {{ method: 'POST', headers: {{'Content-Type': 'application/json', 'X-CSRF-Token': '{}'}}, body: JSON.stringify({{action: 'disable'}}) }}).then(() => fetch('/api/apps/{}/integration').then(r=>r.json()).then(d=>integrationData=d))">Disable</button>
                            </div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'radarr' || integrationData.type === 'sonarr'">
                        <div class="nested-metrics-grid">
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.queue_size"></span>
                                <span class="metric-label">Items in Queue</span>
                            </div>
                        </div>
                    </template>
                </div>"#,
                app.id, csrf_token, app.id
            );
        }

        let alpine_init = if !app.integration_type.is_empty() {
            format!(
                r#"x-data="{{ integrationData: null }}" x-init="fetch('/api/apps/{}/integration').then(r => r.json()).then(d => integrationData = d)""#,
                app.id
            )
        } else {
            "".to_string()
        };

        let card = format!(
            r#"
            <div class="glass-panel app-card" data-app-name="{}" data-category="{}" {}>
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
                {}
            </div>"#,
            escape_html(&name_lower),
            escape_html(&cat_slug),
            alpine_init,
            escape_html(&app.url),
            escape_html(&brand_logo),
            escape_html(&app.name),
            escape_html(&app.description),
            status_badge,
            ctrl_container,
            delete_btn,
            sub_metrics,
            integration_widget
        );
        cols[col_idx].push_str(&card);
    }

    cols.into_iter()
        .map(|col| format!(r#"<div class="bento-column">{}</div>"#, col))
        .collect::<Vec<_>>()
        .join("")
}

fn render_auth_buttons(session: &Option<Session>, csrf_attr: &str) -> String {
    if let Some(ref sess) = session {
        let admin_settings_btn = if sess.role == "Admin" {
            r#"
            <button type="button" class="glass-panel btn-admin" @click="addAppModalOpen = true; appIconUrl = ''; newApp = { integration_type: '', api_key: '' };" style="padding:0.5rem 1rem; border-radius:8px; background:rgba(255,255,255,0.02); font-weight:600; cursor:pointer; font-size:0.82rem; display:inline-flex; align-items:center; gap:0.35rem; color:#fff; border:1px solid rgba(255,255,255,0.06);">
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
            admin_settings_btn,
            csrf_attr,
            escape_html(&sess.username)
        )
    } else {
        r#"
        <a href="/login" class="glass-panel" style="padding:0.5rem 1rem; border-radius:8px; font-weight:600; font-size:0.82rem; text-decoration:none; color:#fff; border:1px solid rgba(255,255,255,0.06); display:inline-flex; align-items:center; gap:0.35rem; background:var(--accent-glow);">
            <i data-lucide="key-round" style="width:0.95rem; height:0.95rem;"></i> Sign In
        </a>
        "#.to_string()
    }
}

fn render_streams(apps: &[App], session: &Option<Session>) -> String {
    let has_plex = apps.iter().any(is_plex_app);
    let has_jellyfin = apps.iter().any(is_jellyfin_app);

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
        format!(r#"<section class="{}">{}</section>"#, cols_class, cards)
    } else {
        String::new()
    }
}

fn render_category_tabs(apps: &[App]) -> String {
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
    category_tabs_html
}

fn render_support_section(settings: &HashMap<String, String>) -> String {
    let donate_enabled = settings
        .get("donate_enabled")
        .map(|s| s.as_str())
        .unwrap_or("1");
    if donate_enabled == "1" {
        let mut links = String::new();
        for (url, label, icon) in DONATION_LINKS.iter() {
            links.push_str(&format!(
                r#"<a href="{}" target="_blank" rel="noopener noreferrer" class="support-link"><i data-lucide="{}" style="width:1rem; height:1rem;"></i> {}</a>"#,
                url, icon, label
            ));
        }
        format!(
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
        )
    } else {
        String::new()
    }
}

fn render_wol_devices(
    devices: &[crate::models::WolDevice],
    is_admin: bool,
    csrf_attr: &str,
) -> String {
    if devices.is_empty() {
        return String::new();
    }

    let mut cards = String::new();
    for dev in devices {
        let icon_name = if dev.icon.trim().is_empty() {
            "cpu"
        } else {
            &dev.icon
        };
        let ip_info = if dev.ip_address.trim().is_empty() {
            "".to_string()
        } else {
            format!(
                r#"<span class="wol-ip" style="font-size:0.75rem; color:var(--text-muted);">({})</span>"#,
                escape_html(&dev.ip_address)
            )
        };

        let action_btn = if is_admin {
            format!(
                r#"<button type="button" class="btn-wake-app" title="Wake Device" data-wake-id="{}" style="background:var(--accent-glow); border:1px solid rgba(255,255,255,0.06); color:#fff; cursor:pointer; padding:0.4rem 0.8rem; border-radius:6px; display:inline-flex; align-items:center; gap:0.35rem; font-size:0.78rem; font-weight:600;" @click="triggerWakeAction($el)">
                    <i data-lucide="power" style="width:0.9rem; height:0.9rem;"></i> Wake
                </button>"#,
                dev.id
            )
        } else {
            "".to_string()
        };

        cards.push_str(&format!(
            r#"
            <div class="glass-panel wol-card" style="display:flex; align-items:center; justify-content:space-between; padding:0.75rem 1rem; border-radius:var(--bento-radius); background:rgba(255,255,255,0.01); border:1px solid rgba(255,255,255,0.04); min-width:240px; flex:1;">
                <div style="display:flex; align-items:center; gap:0.75rem;">
                    <div class="wol-icon-wrapper" style="width:2.2rem; height:2.2rem; border-radius:8px; background:rgba(255,255,255,0.02); display:flex; align-items:center; justify-content:center; color:var(--accent-color);">
                        <i data-lucide="{}"></i>
                    </div>
                    <div style="display:flex; flex-direction:column;">
                        <span class="wol-name" style="font-weight:600; font-size:0.88rem; color:#fff;">{}</span>
                        <div style="display:flex; align-items:center; gap:0.3rem;">
                            <span class="wol-mac" style="font-size:0.72rem; color:var(--text-muted); font-family:monospace;">{}</span>
                            {}
                        </div>
                    </div>
                </div>
                {}
            </div>
            "#,
            escape_html(icon_name),
            escape_html(&dev.name),
            escape_html(&dev.mac_address),
            ip_info,
            action_btn
        ));
    }

    format!(
        r#"
        <section class="wol-section" style="margin-bottom:2rem;">
            <div style="display:flex; align-items:center; gap:0.5rem; margin-bottom:0.75rem;">
                <i data-lucide="zap" style="color:var(--accent-color); width:1.2rem; height:1.2rem;"></i>
                <h2 style="font-size:1.1rem; font-weight:700; color:#fff; margin:0;">Power Controls (Wake-on-LAN)</h2>
            </div>
            <div class="wol-grid" style="display:flex; flex-wrap:wrap; gap:0.75rem;">
                {}
            </div>
        </section>
        "#,
        cards
    )
}
