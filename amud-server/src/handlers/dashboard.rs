use super::imports::*;
use super::share::share_session_from_headers;
use crate::models::App;

#[derive(Clone, Copy)]
enum PageMode {
    Dashboard,
    Feeds,
}

pub async fn dashboard_handler(
    Extension(csp): Extension<CspNonce>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    render_page(PageMode::Dashboard, &csp.0, &headers, &state).await
}

pub async fn feeds_page_handler(
    Extension(csp): Extension<CspNonce>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    render_page(PageMode::Feeds, &csp.0, &headers, &state).await
}

async fn render_page(
    mode: PageMode,
    nonce: &str,
    headers: &HeaderMap,
    state: &Arc<AppState>,
) -> Html<String> {
    let session = get_session(headers, &state.sessions);
    let share_session = share_session_from_headers(headers, state);

    let settings = state.settings_cache.read().unwrap().clone();

    let mode_path = match mode {
        PageMode::Dashboard => "/",
        PageMode::Feeds => "/feeds",
    };
    if let Some(share) = &share_session {
        if !share
            .allowed_paths
            .split(',')
            .map(str::trim)
            .any(|p| p == mode_path)
        {
            return Html("<html><body><p>Share link does not allow this page.</p><a href=\"/\">Home</a></body></html>".to_string());
        }
    }

    let iframe_embeds_enabled = settings
        .get("iframe_embeds_enabled")
        .map(|s| s.as_str())
        .unwrap_or("0")
        == "1";
    let branding = branding_from_settings(&settings);
    let tagline = branding
        .tagline
        .as_deref()
        .unwrap_or("Homelab Operations Cockpit");
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
    let hide_telemetry = match mode {
        PageMode::Feeds => true,
        PageMode::Dashboard => match &session {
            None => telemetry_public != "1",
            Some(s) if s.role == "Guest" => telemetry_public != "1",
            _ => false,
        },
    };
    let logo_manifest = state.logo_manifest.clone();
    let custom_css = settings.get("custom_css").map(|s| s.as_str()).unwrap_or("");
    let csrf_token = csrf_token_for_session(headers, &state.sessions);
    let csrf_attr = escape_html(&csrf_token);

    let (db_categories, all_apps, wol_devices) = with_db(state.db.clone(), move |db| {
        let wol = match mode {
            PageMode::Dashboard => load_wol_devices_from_db(db),
            PageMode::Feeds => vec![],
        };
        (load_categories(db), load_apps_from_db(db), wol)
    })
    .await;

    // Filter apps based on page mode: dashboard hides RSS, feeds shows only RSS
    let mut apps: Vec<App> = match mode {
        PageMode::Dashboard => all_apps
            .into_iter()
            .filter(|a| a.integration_type != "rss")
            .collect(),
        PageMode::Feeds => all_apps
            .into_iter()
            .filter(|a| a.integration_type == "rss")
            .collect(),
    };

    let is_guest = !is_admin && is_guest_session(&session);
    if is_guest {
        if let Some(allowed) = parse_guest_visible_categories(&settings) {
            apps.retain(|app| {
                let cat = if app.category.is_empty() {
                    "General"
                } else {
                    app.category.as_str()
                };
                allowed.contains(cat)
            });
        }
        apps.retain(|app| app.guest_visible);
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
    let app_names_json = serde_json::to_string(
        &apps
            .iter()
            .map(|app| app.name.to_lowercase())
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".to_string());

    let feed_categories = match mode {
        PageMode::Feeds => with_db(state.db.clone(), load_feed_categories_json).await,
        PageMode::Dashboard => Vec::new(),
    };

    let apps_html = match mode {
        PageMode::Feeds => render_feeds_grid(
            &apps,
            is_admin,
            &csrf_attr,
            &logo_manifest,
            &feed_categories,
        ),
        PageMode::Dashboard => render_apps_grid(
            &apps,
            is_admin,
            &csrf_token,
            &csrf_attr,
            &logo_manifest,
            iframe_embeds_enabled,
        ),
    };

    let wol_html = render_wol_devices(&wol_devices, is_admin, &csrf_attr);

    let kiosk_mode = settings
        .get("kiosk_mode")
        .map(|s| s.as_str())
        .unwrap_or("0")
        == "1";
    let auth_buttons = render_auth_buttons(&session, &csrf_attr, mode, kiosk_mode);
    let streams_html = match mode {
        PageMode::Dashboard => render_streams(
            &apps,
            &session,
            is_admin,
            &csrf_attr,
            logo_manifest.as_ref(),
        ),
        PageMode::Feeds => String::new(),
    };
    let category_tabs_html = match mode {
        PageMode::Feeds => render_feed_category_tabs(&apps, &feed_categories),
        PageMode::Dashboard => {
            let layout = sanitize_dashboard_layout(
                settings
                    .get("dashboard_layout")
                    .map(|s| s.as_str())
                    .unwrap_or("tabs"),
            );
            if layout == "sections" {
                render_category_sections(&apps)
            } else {
                render_category_tabs(&apps)
            }
        }
    };
    let widgets_html = match mode {
        PageMode::Dashboard => {
            let widgets = with_db(state.db.clone(), load_dashboard_widgets).await;
            super::widgets::render_dashboard_widgets(&widgets, is_guest)
        }
        PageMode::Feeds => String::new(),
    };
    let support_html = match mode {
        PageMode::Dashboard => render_support_section(&settings),
        PageMode::Feeds => String::new(),
    };
    let feed_hero_html = match mode {
        PageMode::Feeds if !apps.is_empty() => {
            r#"<section id="feed-hero" class="glass-panel feed-hero" hidden aria-live="polite"></section>"#
                .to_string()
        }
        _ => String::new(),
    };

    let root_css = build_root_css(&branding);

    let index_tmpl = include_str!("../../../ui/templates/index.html");
    let username = session
        .as_ref()
        .map(|s| s.username.as_str())
        .unwrap_or("guest");
    let proxmox_enabled = settings
        .get("enable_proxmox")
        .map(|s| s.as_str())
        .unwrap_or("0")
        == "1";

    let app_version = option_env!("GIT_TAG").unwrap_or(env!("CARGO_PKG_VERSION"));
    let app_version_formatted = if app_version.starts_with('v') {
        app_version.to_string()
    } else {
        format!("v{}", app_version)
    };

    let mut update_banner = String::new();
    let mut update_status_class = String::new();

    if is_admin {
        let cache = {
            let lock = crate::handlers::RELEASE_CACHE.read().unwrap();
            lock.clone()
        };

        if let Some(cached) = cache {
            if crate::handlers::semver_newer(&app_version_formatted, &cached.latest) {
                update_status_class = "update-available".to_string();
                update_banner = format!(
                    r#"<div id="update-banner" class="update-banner">
    <div class="update-banner-content">
        <span class="update-banner-dot animate-pulse"></span>
        <span class="update-banner-text">A new update is available! You are running <strong>{}</strong>, latest is <strong>{}</strong>.</span>
    </div>
    <div class="update-banner-actions">
        <a href="/admin/settings?tab=system" class="btn-update-banner">Go to System &rarr;</a>
        <button type="button" onclick="dismissUpdateBanner()" class="btn-update-dismiss">&times;</button>
    </div>
</div>"#,
                    escape_html(&app_version_formatted),
                    escape_html(&cached.latest)
                );
            }
        }
    }

    let safe_app_name = escape_html(&branding.app_name);
    let safe_tagline = escape_html(tagline);
    let safe_app_logo_css = safe_css_url(&branding.app_logo);
    let safe_accent = branding.accent_color.clone();
    let safe_glass_blur = branding
        .glass_blur
        .parse::<u16>()
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "16".to_string());
    let safe_glass_opacity = branding
        .glass_opacity
        .parse::<f64>()
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "0.45".to_string());
    let safe_bento_radius = branding
        .bento_radius
        .parse::<u16>()
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "16".to_string());
    let safe_weather_lat = safe_weather_coord(weather_lat);
    let safe_weather_lon = safe_weather_coord(weather_lon);
    let safe_csrf_meta = escape_html(&csrf_token);
    let safe_app_version = escape_html(&app_version_formatted);

    // Page mode variables
    let page_title_suffix = match mode {
        PageMode::Dashboard => "DASHBOARD",
        PageMode::Feeds => "FEEDS",
    };
    let feeds_nav = match mode {
        PageMode::Dashboard => r#"<a href="/feeds" class="glass-panel topbar-action"><i data-lucide="rss"></i> Feeds</a>"#.to_string(),
        PageMode::Feeds => r#"<a href="/" class="glass-panel topbar-action"><i data-lucide="arrow-left"></i> Dashboard</a>"#.to_string(),
    };
    let main_grid_class = match mode {
        PageMode::Feeds => "feeds-grid",
        PageMode::Dashboard => "bento-grid",
    };
    let body_page_class = match mode {
        PageMode::Feeds => "page-feeds",
        PageMode::Dashboard => "",
    };

    let mut result = index_tmpl
        .replace("/* ROOT_CSS */", &root_css)
        .replace("{{app_name}}", &safe_app_name)
        .replace("{{tagline}}", &safe_tagline)
        .replace("{{page_title_suffix}}", page_title_suffix)
        .replace("<!-- FEEDS_NAV -->", &feeds_nav)
        .replace("{{main_grid_class}}", main_grid_class)
        .replace("{{body_page_class}}", body_page_class)
        .replace(
            "{{proxmox_enabled}}",
            if proxmox_enabled { "true" } else { "false" },
        )
        .replace("{{custom_bg_url}}", &safe_css_url(&branding.custom_bg_url));

    if branding.app_logo.is_empty() {
        result = result.replace(
            "{{if app_logo}}style=\"background-image: url('{{app_logo}}');\"{{end}}",
            "",
        );
    } else {
        result = result
            .replace("{{if app_logo}}", "")
            .replace("{{app_logo}}", &safe_app_logo_css)
            .replace("{{end}}", "");
    }

    let result = result
        .replace("{{accent_color}}", &safe_accent)
        .replace("{{glass_blur_intensity}}", &safe_glass_blur)
        .replace("{{glass_opacity}}", &safe_glass_opacity)
        .replace("{{bento_radius}}", &safe_bento_radius)
        .replace("<!-- APPS_GRID -->", &apps_html)
        .replace("<!-- WOL_SECTION -->", &wol_html)
        .replace("<!-- STREAMS_ROW -->", &streams_html)
        .replace("<!-- FEED_HERO -->", &feed_hero_html)
        .replace("<!-- CATEGORY_TABS -->", &category_tabs_html)
        .replace("<!-- DASHBOARD_WIDGETS -->", &widgets_html)
        .replace("<!-- SUPPORT_SECTION -->", &support_html)
        .replace("<!-- AUTH_BUTTONS -->", &auth_buttons)
        .replace("{{username}}", &escape_html(username))
        .replace("{{weather_latitude}}", &safe_weather_lat)
        .replace("{{weather_longitude}}", &safe_weather_lon)
        .replace("<!-- CATEGORY_OPTIONS -->", &category_options_html)
        .replace("{{csrf_token}}", &safe_csrf_meta)
        .replace("{{telemetry_public}}", telemetry_public)
        .replace(
            "{{hide_telemetry}}",
            if hide_telemetry { "true" } else { "false" },
        )
        .replace("{{custom_css}}", custom_css)
        .replace("{{app_names_json}}", &app_names_json)
        .replace("{{is_admin}}", if is_admin { "true" } else { "false" })
        .replace(
            "{{admin_drag_script}}",
            if is_admin {
                r#"<script src="/static/drag.js"></script>"#
            } else {
                ""
            },
        )
        .replace("{{app_version}}", &safe_app_version)
        .replace(
            "{{update_status_class}}",
            &escape_html(&update_status_class),
        )
        .replace("{{update_banner}}", &update_banner);

    // Theme mode (light/dark)
    let theme_mode = &branding.theme_mode;
    let theme_scheduler_config = build_theme_scheduler_json(&settings, theme_mode);
    let result = result
        .replace("{{theme_mode}}", &escape_html(theme_mode))
        .replace("{{theme_scheduler_config}}", &theme_scheduler_config);

    // Video wallpaper support
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
    let result = result
        .replace(
            "{{video_bg_class}}",
            if is_video_bg { "has-video-bg" } else { "" },
        )
        .replace("{{video_bg_element}}", &video_bg_element);

    Html(apply_csp_nonce(result, nonce))
}

fn is_guest_session(session: &Option<Session>) -> bool {
    match session {
        None => true,
        Some(s) => s.role == "Guest",
    }
}

fn filled_cpu_ram_cells() -> &'static str {
    r#"<div class="metric-block" data-lxc-cpu><span class="metric-value">—</span><span class="metric-label">CPU</span></div><div class="metric-block" data-lxc-ram><span class="metric-value">—</span><span class="metric-label">RAM</span></div>"#
}

fn filled_loading_grid() -> &'static str {
    r#"<div class="metric-block"><span class="metric-value">—</span><span class="metric-label">…</span></div><div class="metric-block"><span class="metric-value">—</span><span class="metric-label">…</span></div><div class="metric-block"><span class="metric-value">—</span><span class="metric-label">…</span></div><div class="metric-block"><span class="metric-value">—</span><span class="metric-label">…</span></div><div class="metric-block"><span class="metric-value">—</span><span class="metric-label">…</span></div><div class="metric-block"><span class="metric-value">—</span><span class="metric-label">…</span></div>"#
}

fn build_filled_integration_widget(_app_id: i64, _csrf_token: &str, show_cpu_ram: bool) -> String {
    let cpu_ram = if show_cpu_ram {
        filled_cpu_ram_cells()
    } else {
        ""
    };
    format!(
        r#"
                <div class="integration-widget integration-widget--filled">
                    <div class="integration-metrics-grid app-card-metrics-fallback" x-show="!integrationData || !integrationData.type">{loading}</div>
                    <div x-show="integrationData && integrationData.type">
                    <template x-if="integrationData.type === 'pihole' || integrationData.type === 'adguard'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.ads_blocked_today ?? '—'"></span><span class="metric-label">Blocked</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.dns_queries_today ?? '—'"></span><span class="metric-label">Queries</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.ads_percentage_today ?? integrationData.avg_processing_time ?? '—'"></span><span class="metric-label" x-text="integrationData.type === 'pihole' ? 'Block %' : 'Avg time'"></span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.domains_being_blocked ?? integrationData.status ?? '—'"></span><span class="metric-label" x-text="integrationData.type === 'pihole' ? 'Domains' : 'Status'"></span></div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'radarr'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.queue_size ?? '—'"></span><span class="metric-label">Queue</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.missing ?? '—'"></span><span class="metric-label">Missing</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.library_count ?? '—'"></span><span class="metric-label">Movies</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.disk_free ?? '—'"></span><span class="metric-label">Disk free</span></div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'sonarr'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.queue_size ?? '—'"></span><span class="metric-label">Queue</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.missing ?? '—'"></span><span class="metric-label">Missing</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.series_count ?? '—'"></span><span class="metric-label">Series</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.episode_count ?? '—'"></span><span class="metric-label">Episodes</span></div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'overseerr' || integrationData.type === 'jellyseerr'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.pending_requests ?? '—'"></span><span class="metric-label">Pending</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.approved_requests ?? '—'"></span><span class="metric-label">Approved</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.processing_requests ?? '—'"></span><span class="metric-label">Processing</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.total_requests ?? '—'"></span><span class="metric-label">Total</span></div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'prowlarr'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value"><span x-text="integrationData.indexers_enabled ?? '—'"></span>/<span x-text="integrationData.indexers_total ?? '—'"></span></span><span class="metric-label">Indexers</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.queue_size ?? '—'"></span><span class="metric-label">Queue</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.failed_indexers ?? '—'"></span><span class="metric-label">Failed</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.version ?? '—'"></span><span class="metric-label">Version</span></div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'uptime_kuma'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.monitors_up ?? '—'"></span><span class="metric-label">Up</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.monitors_down ?? '—'"></span><span class="metric-label">Down</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.monitors_total ?? '—'"></span><span class="metric-label">Total</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.maintenance ?? '—'"></span><span class="metric-label">Maint.</span></div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'cloudflare_tunnel'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value" style="font-size:0.75rem;text-transform:capitalize;" x-text="integrationData.tunnel_status ?? '—'"></span><span class="metric-label">Status</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.connections ?? '—'"></span><span class="metric-label">Connections</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.colo_count ?? '—'"></span><span class="metric-label">Colos</span></div>
                            <div class="metric-block"><span class="metric-value" style="font-size:0.7rem;" x-text="integrationData.tunnel_name ?? '—'"></span><span class="metric-label">Tunnel</span></div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'peanut'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value"><span x-text="integrationData.battery_percent ?? '—'"></span>%</span><span class="metric-label">Battery</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.ups_load ?? '—'"></span><span class="metric-label">Load</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.battery_runtime ?? '—'"></span><span class="metric-label">Runtime</span></div>
                            <div class="metric-block"><span class="metric-value" style="font-size:0.75rem;" x-text="integrationData.ups_status ?? '—'"></span><span class="metric-label">UPS</span></div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'qbittorrent'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.download_speed ?? '—'"></span><span class="metric-label">Download</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.upload_speed ?? '—'"></span><span class="metric-label">Upload</span></div>
                            <div class="metric-block"><span class="metric-value"><span x-text="integrationData.active_downloads ?? '—'"></span>↓ <span x-text="integrationData.seeding ?? '—'"></span>↑</span><span class="metric-label">Active</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.free_disk ?? '—'"></span><span class="metric-label">Free disk</span></div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'bazarr'">
                        <div class="integration-metrics-grid" data-lxc-metrics>
                            {cpu_ram}
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.missing_episodes ?? '—'"></span><span class="metric-label">Ep. missing</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.missing_movies ?? '—'"></span><span class="metric-label">Mov. missing</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.version ?? '—'"></span><span class="metric-label">Version</span></div>
                            <div class="metric-block"><span class="metric-value" x-text="integrationData.health ?? '—'"></span><span class="metric-label">Health</span></div>
                        </div>
                    </template>
                    </div>
                </div>"#,
        loading = filled_loading_grid(),
        cpu_ram = cpu_ram,
    )
}

fn render_apps_grid(
    apps: &[App],
    is_admin: bool,
    csrf_token: &str,
    csrf_attr: &str,
    logo_manifest: &HashMap<String, String>,
    iframe_embeds_enabled: bool,
) -> String {
    if apps.is_empty() {
        return r#"
        <div class="glass-panel app-card app-card--empty">
            <p style="font-weight: 600; color: var(--text-secondary);">No services configured yet</p>
            <p style="font-size: 0.8rem; color: var(--text-muted); margin-top: 0.5rem;">Log in as Admin and click "Add App" to register your infrastructure.</p>
        </div>"#
            .to_string();
    }

    let mut cards_html = String::new();
    for app in apps.iter() {
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

        let cat_slug = category_slug(&app.category);

        let name_lower = app.name.to_lowercase();
        let use_filled = app.card_span == "1x2"
            && is_admin
            && !app.integration_type.is_empty()
            && app.integration_type != "rss";
        let sub_metrics = if is_admin {
            if name_lower.contains("home") && name_lower.contains("assistant") {
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
            } else if app.show_container_metrics && !use_filled {
                format!(
                    r#"
                <div class="nested-metrics-grid cols-2" data-lxc-metrics>
                    <div class="metric-block">
                        <span class="metric-value">—</span>
                        <span class="metric-label">CPU</span>
                    </div>
                    <div class="metric-block">
                        <span class="metric-value">—</span>
                        <span class="metric-label">RAM</span>
                    </div>
                </div>"#
                )
            } else {
                String::new()
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
            let edit_control = format!(
                r#"<button type="button" class="btn-edit-app" title="Edit application" data-app="{}" @click="editApp = JSON.parse($el.getAttribute('data-app')); window.editingAppOriginalName = (editApp.name || '').toLowerCase(); editAppModalOpen = true; setTimeout(checkDuplicateAppName, 0);"><i data-lucide="edit-2"></i></button>"#,
                escaped_json
            );
            format!(
                r#"
                <div style="display: inline-flex; align-items: center; gap: 0.25rem;">
                    {}
                    <form action="/apps/delete" method="POST" style="margin: 0; display: inline-flex; align-items: center;">
                        <input type="hidden" name="id" value="{}">
                        <input type="hidden" name="csrf_token" value="{}">
                        <button type="submit" class="btn-delete-app" title="Delete application">
                            <i data-lucide="trash-2"></i>
                        </button>
                    </form>
                </div>
                "#,
                edit_control, app.id, csrf_attr
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
        if is_admin && !app.integration_type.is_empty() {
            if use_filled {
                integration_widget =
                    build_filled_integration_widget(app.id, csrf_token, app.show_container_metrics);
            } else if app.integration_type == "rss" {
                integration_widget = format!(
                    r#"
                <div class="integration-widget integration-widget--always">
                    <div class="nested-metrics-grid app-card-metrics-fallback" x-show="!integrationData || !integrationData.type">
                        <div class="metric-block">
                            <span class="metric-value">—</span>
                            <span class="metric-label">Loading</span>
                        </div>
                    </div>
                    <div x-show="integrationData && integrationData.type">
                    <template x-if="integrationData.type === 'rss'">
                        <div class="rss-feed-list">
                            <template x-for="(entry, index) in integrationData.entries" :key="index">
                                <a x-show="entry.link" :href="entry.link" target="_blank" rel="noopener" class="rss-feed-item">
                                    <span class="rss-feed-title" x-text="entry.title"></span>
                                    <span class="rss-feed-date" x-text="entry.date"></span>
                                </a>
                                <div x-show="!entry.link" class="rss-feed-item rss-feed-item--text-only">
                                    <span class="rss-feed-title" x-text="entry.title"></span>
                                    <span class="rss-feed-date" x-text="entry.date"></span>
                                </div>
                            </template>
                            <div x-show="!integrationData.entries || integrationData.entries.length === 0" class="rss-feed-empty">
                                <span>No entries found</span>
                            </div>
                        </div>
                    </template>
                    </div>
                </div>"#
                );
            } else {
                let integration_class =
                    "integration-widget integration-widget--hover app-card-metrics-layer app-card-metrics-layer--integration";
                integration_widget = format!(
                    r#"
                <div class="{}">
                    <div class="nested-metrics-grid app-card-metrics-fallback" x-show="!integrationData || !integrationData.type">
                        <div class="metric-block">
                            <span class="metric-value">—</span>
                            <span class="metric-label">Loading</span>
                        </div>
                    </div>
                    <div x-show="integrationData && integrationData.type">
                    <template x-if="integrationData.type === 'pihole' || integrationData.type === 'adguard'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.ads_blocked_today"></span>
                                <span class="metric-label">Ads Blocked</span>
                            </div>
                            <div class="metric-block" style="flex-direction: row; justify-content: center; gap: 0.5rem; align-items: center;">
                                <span class="metric-value" style="font-size: 0.8rem; text-transform: uppercase;" x-text="integrationData.status"></span>
                                <button type="button" class="btn btn-secondary" style="padding: 0.2rem 0.5rem; font-size: 0.7rem; height: auto;" @click="fetch('/api/apps/{}/integration/action', {{ method: 'POST', headers: {{'Content-Type': 'application/json', 'X-CSRF-Token': '{}'}}, body: JSON.stringify({{action: 'disable'}}) }}).then(() => fetch('/api/apps/{}/integration').then(r => r.ok ? r.json() : null)).then(d => {{ if (d && d.type) integrationData = d }}).catch(() => {{}})">Disable</button>
                            </div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'radarr' || integrationData.type === 'sonarr'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.queue_size"></span>
                                <span class="metric-label">Queue</span>
                            </div>
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.missing"></span>
                                <span class="metric-label">Missing</span>
                            </div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'overseerr' || integrationData.type === 'jellyseerr'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.pending_requests"></span>
                                <span class="metric-label">Pending</span>
                            </div>
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.total_requests"></span>
                                <span class="metric-label">Total</span>
                            </div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'prowlarr'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value"><span x-text="integrationData.indexers_enabled"></span>/<span x-text="integrationData.indexers_total"></span></span>
                                <span class="metric-label">Indexers</span>
                            </div>
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.queue_size"></span>
                                <span class="metric-label">Queue</span>
                            </div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'uptime_kuma'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.monitors_up"></span>
                                <span class="metric-label">Up</span>
                            </div>
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.monitors_down"></span>
                                <span class="metric-label">Down</span>
                            </div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'cloudflare_tunnel'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value" style="font-size:0.8rem; text-transform:capitalize;" x-text="integrationData.tunnel_status"></span>
                                <span class="metric-label" x-text="integrationData.tunnel_name"></span>
                            </div>
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.connections"></span>
                                <span class="metric-label">Connections</span>
                            </div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'peanut'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value"><span x-text="integrationData.battery_percent"></span>%</span>
                                <span class="metric-label">Battery</span>
                            </div>
                            <div class="metric-block">
                                <span class="metric-value" style="font-size:0.75rem;" x-text="integrationData.ups_status"></span>
                                <span class="metric-label">UPS</span>
                            </div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'qbittorrent'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.download_speed"></span>
                                <span class="metric-label">Download</span>
                            </div>
                            <div class="metric-block">
                                <span class="metric-value"><span x-text="integrationData.active_downloads"></span>↓ <span x-text="integrationData.seeding"></span>↑</span>
                                <span class="metric-label">Active</span>
                            </div>
                        </div>
                    </template>
                    <template x-if="integrationData.type === 'bazarr'">
                        <div class="nested-metrics-grid cols-2">
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.missing_episodes"></span>
                                <span class="metric-label">Ep. missing</span>
                            </div>
                            <div class="metric-block">
                                <span class="metric-value" x-text="integrationData.missing_movies"></span>
                                <span class="metric-label">Mov. missing</span>
                            </div>
                        </div>
                    </template>
                    </div>
                </div>"#,
                    integration_class, app.id, csrf_token, app.id
                );
            }
        }

        let alpine_init = if is_admin && !app.integration_type.is_empty() {
            format!(
                r#"x-data="{{ integrationData: null }}" data-integration-refresh="{}" x-init="fetch('/api/apps/{}/integration').then(r => r.ok ? r.json() : null).then(d => {{ if (d && d.type) integrationData = d }}).catch(() => {{}})""#,
                app.id, app.id
            )
        } else {
            "".to_string()
        };

        let mut alias_tokens = vec![name_lower.clone()];
        if !lowercase_icon.is_empty() && lowercase_icon != name_lower {
            alias_tokens.push(lowercase_icon);
        }
        let url_lower = app.url.to_lowercase();
        for token in url_lower
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|t| t.len() >= 3)
        {
            if !alias_tokens.iter().any(|t| t == token) {
                alias_tokens.push(token.to_string());
            }
        }
        let container_aliases = alias_tokens.join(" ");
        let is_host_agent_app = alias_tokens
            .iter()
            .any(|t| t == "proxmox" || t == "pve" || t == "beszel" || t == "filebrowser");
        let guest_compact_class = if !is_admin {
            " app-card--guest-compact"
        } else {
            ""
        };

        let span_class = match app.card_span.as_str() {
            "2x1" => " span-2",
            "1x2" => " span-tall",
            _ => "",
        };

        let metrics_slot = if use_filled {
            format!(
                r#"<div class="app-card-metrics-slot app-card-metrics-slot--filled">{}</div>"#,
                integration_widget
            )
        } else if app.integration_type == "rss" {
            if sub_metrics.is_empty() {
                integration_widget.clone()
            } else {
                format!("{}{}", sub_metrics, integration_widget)
            }
        } else if sub_metrics.is_empty() && integration_widget.is_empty() {
            String::new()
        } else if integration_widget.is_empty() {
            format!(
                r#"<div class="app-card-metrics-slot">{}</div>"#,
                sub_metrics
            )
        } else if sub_metrics.is_empty() {
            format!(
                r#"<div class="app-card-metrics-slot app-card-metrics-slot--solo">{}</div>"#,
                integration_widget
            )
        } else {
            format!(
                r#"<div class="app-card-metrics-slot app-card-metrics-slot--dual">{}{}</div>"#,
                sub_metrics, integration_widget
            )
        };

        let open_url =
            if iframe_embeds_enabled && (app.embed_mode == "iframe" || app.embed_mode == "tab") {
                format!("/embed/{}", app.id)
            } else {
                app.url.clone()
            };
        let link_target = if app.embed_mode == "tab" && iframe_embeds_enabled {
            ""
        } else {
            r#" target="_blank" rel="noopener noreferrer""#
        };
        let embed_mode_attr = if iframe_embeds_enabled && !app.embed_mode.is_empty() {
            format!(r#" data-embed-mode="{}""#, escape_html(&app.embed_mode))
        } else {
            String::new()
        };

        let card = format!(
            r#"
            <div class="glass-panel app-card{}{}" data-app-name="{}" data-app-id="{}" data-category="{}" data-container-aliases="{}" data-host-agent-app="{}" data-show-container-metrics="{}" {}>
                <div class="app-card-header">
                    <a href="{}"{}{} class="app-card-identity app-card-open" style="text-decoration:none; color:inherit;">
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
            guest_compact_class,
            span_class,
            escape_html(&name_lower),
            app.id,
            escape_html(&cat_slug),
            escape_html(&container_aliases),
            if is_host_agent_app { "true" } else { "false" },
            if app.show_container_metrics {
                "true"
            } else {
                "false"
            },
            alpine_init,
            escape_html(&open_url),
            link_target,
            embed_mode_attr,
            escape_html(&brand_logo),
            escape_html(&app.name),
            escape_html(&app.description),
            status_badge,
            ctrl_container,
            delete_btn,
            metrics_slot
        );
        cards_html.push_str(&card);
    }

    cards_html
}

fn feed_category_meta(name: &str, feed_categories: &[serde_json::Value]) -> (String, String) {
    for cat in feed_categories {
        if cat.get("name").and_then(|v| v.as_str()) == Some(name) {
            let color = cat
                .get("color")
                .and_then(|v| v.as_str())
                .unwrap_or("#64748b")
                .to_string();
            let icon = cat
                .get("icon")
                .and_then(|v| v.as_str())
                .unwrap_or("rss")
                .to_string();
            return (color, icon);
        }
    }
    ("#64748b".to_string(), "rss".to_string())
}

fn render_feeds_grid(
    apps: &[App],
    is_admin: bool,
    csrf_attr: &str,
    logo_manifest: &HashMap<String, String>,
    feed_categories: &[serde_json::Value],
) -> String {
    if apps.is_empty() {
        return r#"
        <div class="glass-panel feed-card feed-card--empty">
            <p style="font-weight: 600; color: var(--text-secondary);">No RSS feeds configured yet</p>
            <p style="font-size: 0.8rem; color: var(--text-muted); margin-top: 0.5rem;">Admins: add feeds under <a href="/admin/settings?tab=rss" style="color:var(--accent-color);">Settings → RSS Feeds</a>.</p>
        </div>"#
            .to_string();
    }

    let mut cards_html = String::new();
    for app in apps.iter() {
        let feed_url =
            crate::secrets::decrypt_value(&app.api_key).unwrap_or_else(|_| app.api_key.clone());
        let feed_logo = resolve_feed_logo(&app.icon, &app.name, &app.url, &feed_url, logo_manifest);
        let cat_slug = category_slug(&app.category);
        let (cat_color, _cat_icon) = feed_category_meta(&app.category, feed_categories);
        let host = host_from_url(if app.url.is_empty() {
            &app.name
        } else {
            &app.url
        });
        let site_href = if app.url.is_empty() {
            "#".to_string()
        } else {
            escape_html(&app.url)
        };

        let admin_actions = if is_admin {
            format!(
                r#"
                <div class="feed-card-actions">
                    <a href="/admin/settings?tab=rss" class="btn-edit-app" title="Edit in RSS Feeds settings">
                        <i data-lucide="edit-2"></i>
                    </a>
                    <form action="/apps/delete" method="POST" style="margin:0; display:inline-flex;">
                        <input type="hidden" name="id" value="{}">
                        <input type="hidden" name="csrf_token" value="{}">
                        <button type="submit" class="btn-delete-app" title="Delete feed">
                            <i data-lucide="trash-2"></i>
                        </button>
                    </form>
                </div>"#,
                app.id, csrf_attr
            )
        } else {
            String::new()
        };

        let category_pill = format!(
            r#"<span class="feed-category-pill" style="--pill-color:{};">{}</span>"#,
            escape_html(&cat_color),
            escape_html(&app.category)
        );

        let card = format!(
            r#"
            <article class="glass-panel feed-card" data-app-id="{}" data-category="{}" data-app-name="{}" x-data="{{ integrationData: null }}" x-init="fetch('/api/apps/{}/integration').then(r => r.ok ? r.json() : null).then(d => {{ if (d && d.type) integrationData = d }}).catch(() => {{}})">
                <header class="feed-card-header">
                    <a href="{}" target="_blank" rel="noopener noreferrer" class="feed-card-identity">
                        <div class="feed-card-icon">
                            <img src="{}" alt="" onerror="this.src='/static/feeds/icons/rss.svg'">
                        </div>
                        <div class="feed-card-meta">
                            <h3 class="feed-card-title">{}</h3>
                            <span class="feed-card-host">{}</span>
                        </div>
                    </a>
                    <div class="feed-card-badges">
                        {}
                        {}
                    </div>
                </header>
                <div class="integration-widget feed-card-body" x-show="integrationData && integrationData.type === 'rss'">
                    <div class="rss-feed-list">
                        <template x-for="(entry, index) in integrationData.entries" :key="index">
                            <a x-show="entry.link" :href="entry.link" target="_blank" rel="noopener" class="rss-feed-item">
                                <span class="rss-feed-title" x-text="entry.title"></span>
                                <span class="rss-feed-date" x-text="entry.date"></span>
                            </a>
                            <div x-show="!entry.link" class="rss-feed-item rss-feed-item--text-only">
                                <span class="rss-feed-title" x-text="entry.title"></span>
                                <span class="rss-feed-date" x-text="entry.date"></span>
                            </div>
                        </template>
                        <div x-show="!integrationData || !integrationData.entries || integrationData.entries.length === 0" class="rss-feed-empty">
                            <span>Loading headlines…</span>
                        </div>
                    </div>
                </div>
                <footer class="feed-card-footer">
                    <a href="{}" target="_blank" rel="noopener noreferrer" class="feed-card-visit">Visit site <i data-lucide="external-link"></i></a>
                </footer>
            </article>"#,
            app.id,
            escape_html(&cat_slug),
            escape_html(&app.name.to_lowercase()),
            app.id,
            site_href,
            escape_html(&feed_logo),
            escape_html(&app.name),
            escape_html(if host.is_empty() { "news feed" } else { &host }),
            category_pill,
            admin_actions,
            site_href,
        );
        cards_html.push_str(&card);
    }

    cards_html
}

fn render_feed_category_tabs(apps: &[App], feed_categories: &[serde_json::Value]) -> String {
    let mut category_tabs_html = format!(
        r#"<button type="button" class="filter-tab feed-filter-tab active" @click="filterCategory('all', $el)"><i data-lucide="layers"></i> All <span class="filter-count">{}</span></button>"#,
        apps.len()
    );

    for cat in feed_categories {
        let name = cat.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let count = apps.iter().filter(|a| a.category == name).count();
        if count == 0 {
            continue;
        }
        let icon = cat.get("icon").and_then(|v| v.as_str()).unwrap_or("rss");
        let color = cat
            .get("color")
            .and_then(|v| v.as_str())
            .unwrap_or("#64748b");
        let cat_slug = category_slug(name);
        category_tabs_html.push_str(&format!(
            r#"<button type="button" class="filter-tab feed-filter-tab" style="--tab-accent: {};" @click="filterCategory('{}', $el)"><i data-lucide="{}"></i> {} <span class="filter-count">{}</span></button>"#,
            escape_html(color),
            escape_html(&cat_slug),
            escape_html(icon),
            escape_html(name),
            count
        ));
    }

    category_tabs_html
}

fn render_auth_buttons(
    session: &Option<Session>,
    csrf_attr: &str,
    mode: PageMode,
    kiosk_mode: bool,
) -> String {
    let status_link = r#"<a href="/status" class="glass-panel topbar-action"><i data-lucide="activity"></i> Status</a>"#;
    if kiosk_mode && session.is_none() {
        return status_link.to_string();
    }
    if let Some(ref sess) = session {
        let admin_settings_btn = if sess.role == "Admin" {
            match mode {
                PageMode::Feeds => {
                    r#"
            <a href="/admin/settings?tab=rss" class="glass-panel topbar-action btn-admin">
                <i data-lucide="rss"></i> Add RSS Feed
            </a>
            <a href="/admin/settings" class="glass-panel topbar-action btn-admin">
                <i data-lucide="sliders-horizontal"></i> Settings
            </a>
            "#
                }
                PageMode::Dashboard => {
                    r#"
            <button type="button" class="glass-panel topbar-action btn-admin" @click="addAppModalOpen = true; appIconUrl = ''; newApp = { integration_type: '', api_key: '', card_span: '1x1', show_container_metrics: true };">
                <i data-lucide="plus"></i> Add App
            </button>
            <a href="/admin/settings" class="glass-panel topbar-action btn-admin">
                <i data-lucide="sliders-horizontal"></i> Settings
            </a>
            "#
                }
            }
        } else {
            ""
        };
        format!(
            r#"
            {}
            <form action="/logout" method="POST" style="margin:0; display:inline-flex;">
                <input type="hidden" name="csrf_token" value="{}">
                <button type="submit" class="glass-panel topbar-action">
                    <i data-lucide="log-out"></i> Sign Out ({})
                </button>
            </form>
            "#,
            admin_settings_btn,
            csrf_attr,
            escape_html(&sess.username)
        )
    } else {
        r#"
        <a href="/login" class="glass-panel topbar-action topbar-action--accent">
            <i data-lucide="key-round"></i> Sign In
        </a>
        "#
        .to_string()
    }
}

fn category_slug(category: &str) -> String {
    category
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}

fn render_streams(
    apps: &[App],
    session: &Option<Session>,
    _is_admin: bool,
    _csrf_attr: &str,
    _logo_manifest: &HashMap<String, String>,
) -> String {
    if session.is_none() {
        return String::new();
    }

    let has_plex = apps.iter().any(is_plex_app);
    let has_jellyfin = apps.iter().any(is_jellyfin_app);

    if !has_plex && !has_jellyfin {
        return String::new();
    }

    let mut html = String::new();

    let mut media_cards = String::new();

    if has_plex {
        media_cards.push_str(r#"
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
                    <span class="status-badge status-checking stream-status-badge" data-stream-app="plex" data-stream-service="plex">CHECKING...</span>
                </div>
                
                <div class="stream-player">
                    <div class="stream-controls-row">
                        <span class="stream-track-title" id="plex-track">No Active Streams</span>
                        <div style="display: flex; gap: 0.5rem; align-items: center;">
                            <button type="button" class="stream-play-btn"><i data-lucide="play" style="width:0.85rem; height:0.85rem;"></i></button>
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
        media_cards.push_str(r#"
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
                    <span class="status-badge status-checking stream-status-badge" data-stream-app="jellyfin emby" data-stream-service="jellyfin">CHECKING...</span>
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

    if has_plex || has_jellyfin {
        let media_count = (has_plex as u8) + (has_jellyfin as u8);
        let cols_class = if media_count > 1 {
            "streams-row"
        } else {
            "streams-row single-col"
        };
        html.push_str(&format!(
            r#"<section class="{}" data-filter-section="media">{}</section>"#,
            cols_class, media_cards
        ));
    }

    html
}

#[allow(dead_code)]
fn render_proxmox_stream_card(
    app: &App,
    is_admin: bool,
    csrf_attr: &str,
    logo_manifest: &HashMap<String, String>,
) -> String {
    let lowercase_icon = app.icon.to_lowercase();
    let resolved_logo = resolve_logo_from_manifest(&app.icon, logo_manifest);
    let brand_logo = if !resolved_logo.is_empty() {
        resolved_logo
    } else if !lowercase_icon.is_empty() {
        fallback_brand_logo(&lowercase_icon)
    } else {
        "/static/logos/proxmox.svg".to_string()
    };

    let desc = if app.description.trim().is_empty() {
        "Hypervisor node — containers, CPU, and memory at a glance.".to_string()
    } else {
        app.description.clone()
    };

    let admin_actions = if is_admin {
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
            r#"<div style="display: inline-flex; align-items: center; gap: 0.25rem; margin-left: 0.35rem;">
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
            </div>"#,
            escaped_json, app.id, csrf_attr
        )
    } else {
        String::new()
    };

    format!(
        r#"
            <!-- Proxmox host stream card -->
            <div class="glass-panel stream-card" id="proxmox-host-card" data-proxmox-host="true">
                <div class="stream-main">
                    <a href="{url}" target="_blank" rel="noopener noreferrer" class="stream-meta" style="text-decoration:none; color:inherit;">
                        <div class="stream-icon stream-icon--logo">
                            <img src="{logo}" alt="" onerror="this.src='/static/logos/proxmox.svg'">
                        </div>
                        <div>
                            <h2 class="stream-text-title">{name}</h2>
                            <p class="stream-text-desc">{desc}</p>
                        </div>
                    </a>
                    <div style="display:flex; align-items:center; gap:0.35rem; flex-shrink:0;">
                        <span class="status-badge status-checking stream-status-badge" data-proxmox-badge title="Proxmox agent telemetry">CHECKING...</span>
                        {admin_actions}
                    </div>
                </div>
                <div class="stream-player">
                    <div class="stream-controls-row">
                        <span class="stream-track-title" id="proxmox-host-summary" style="color: var(--text-muted);">Waiting for agent telemetry…</span>
                        <span id="proxmox-host-node">—</span>
                    </div>
                    <div class="nested-metrics-grid cols-3 stream-host-metrics" id="proxmox-host-metrics">
                        <div class="metric-block">
                            <span class="metric-value">—</span>
                            <span class="metric-label">Containers</span>
                        </div>
                        <div class="metric-block">
                            <span class="metric-value">—</span>
                            <span class="metric-label">CPU</span>
                        </div>
                        <div class="metric-block">
                            <span class="metric-value">—</span>
                            <span class="metric-label">Mem</span>
                        </div>
                    </div>
                </div>
            </div>
        "#,
        url = escape_html(&app.url),
        logo = escape_html(&brand_logo),
        name = escape_html(&app.name),
        desc = escape_html(&desc),
        admin_actions = admin_actions,
    )
}

fn render_category_tabs(apps: &[App]) -> String {
    let mut categories = Vec::<String>::new();
    for app in apps.iter() {
        if !app.category.is_empty() && !categories.contains(&app.category) {
            categories.push(app.category.clone());
        }
    }

    let mut category_tabs_html = format!(
        r#"<button type="button" class="filter-tab active" @click="filterCategory('all', $el)">All <span class="filter-count">{}</span></button>"#,
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
            r#"<button type="button" class="filter-tab" @click="filterCategory('{}', $el)">{} <span class="filter-count">{}</span></button>"#,
            escape_html(&cat_slug), escape_html(cat), count
        ));
    }
    category_tabs_html
}

fn render_category_sections(apps: &[App]) -> String {
    let mut categories = Vec::<String>::new();
    for app in apps.iter() {
        if !app.category.is_empty() && !categories.contains(&app.category) {
            categories.push(app.category.clone());
        }
    }
    if categories.is_empty() {
        categories.push("General".to_string());
    }
    let mut html = String::from(r#"<div class="category-sections">"#);
    html.push_str(
        r#"<details class="category-section" open data-filter-section="all"><summary class="category-section-title">All apps</summary></details>"#,
    );
    for cat in &categories {
        let cat_slug = category_slug(cat);
        let count = apps.iter().filter(|a| &a.category == cat).count();
        html.push_str(&format!(
            r#"<details class="category-section" open data-filter-section="{}"><summary class="category-section-title">{} <span class="filter-count">{}</span></summary></details>"#,
            escape_html(&cat_slug),
            escape_html(cat),
            count
        ));
    }
    html.push_str("</div>");
    html
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
    _csrf_attr: &str,
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
            <div class="glass-panel wol-card" style="display:flex; align-items:center; justify-content:space-between; padding:0.75rem 1rem; border-radius:var(--radius-xl); background:rgba(255,255,255,0.01); border:1px solid rgba(255,255,255,0.04); min-width:240px; flex:1;">
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

fn safe_weather_coord(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.parse::<f64>().is_ok() {
        escape_html(trimmed)
    } else {
        String::new()
    }
}
