use super::imports::*;

pub async fn list_widgets_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }
    let widgets = with_db(state.db.clone(), load_dashboard_widgets).await;
    api_json(StatusCode::OK, serde_json::json!({ "widgets": widgets }))
}

pub async fn add_widget_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }
    let title = form.get("title").cloned().unwrap_or_default();
    let widget_type = sanitize_widget_type(
        form.get("widget_type")
            .map(|s| s.as_str())
            .unwrap_or("note"),
    );
    let content = if widget_type == "html" {
        sanitize_custom_css(form.get("content").map(|s| s.as_str()).unwrap_or(""))
    } else {
        form.get("content").cloned().unwrap_or_default()
    };
    let guest_visible = parse_show_container_metrics(form.get("guest_visible").map(|s| s.as_str()));
    let grid_span = form
        .get("grid_span")
        .map(|s| sanitize_card_span(s))
        .unwrap_or_else(|| "1x1".to_string());
    if !title.is_empty() {
        with_db(state.db.clone(), move |db| {
            let sort_order = db
                .query_row(
                    "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM dashboard_widgets",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            let _ = db.execute(
                "INSERT INTO dashboard_widgets (widget_type, title, content, sort_order, guest_visible, grid_span) VALUES (?, ?, ?, ?, ?, ?)",
                params![widget_type, title, content, sort_order, guest_visible, grid_span],
            );
        })
        .await;
    }
    Redirect::to("/admin/settings?tab=widgets").into_response()
}

pub async fn delete_widget_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_session(&headers, &state.sessions) {
        return *resp;
    }
    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response();
    }
    if let Some(id) = form.get("id").and_then(|s| s.parse::<i64>().ok()) {
        with_db(state.db.clone(), move |db| {
            let _ = db.execute("DELETE FROM dashboard_widgets WHERE id = ?", params![id]);
        })
        .await;
    }
    Redirect::to("/admin/settings?tab=widgets").into_response()
}

pub(crate) fn render_dashboard_widgets(
    widgets: &[crate::models::DashboardWidget],
    is_guest: bool,
) -> String {
    let visible: Vec<_> = widgets
        .iter()
        .filter(|w| !is_guest || w.guest_visible)
        .collect();
    if visible.is_empty() {
        return String::new();
    }
    let mut html = String::from(r#"<section class="dashboard-widgets-row bento-grid">"#);
    for w in visible {
        let span_class = match w.grid_span.as_str() {
            "2x1" => " span-2",
            "1x2" => " span-tall",
            _ => "",
        };
        let body = match w.widget_type.as_str() {
            "html" => format!(r#"<div class="widget-html">{}</div>"#, w.content),
            "links" => {
                let mut links = String::new();
                for line in w.content.lines() {
                    let parts: Vec<&str> = line.splitn(2, '|').collect();
                    if parts.len() == 2 {
                        links.push_str(&format!(
                            r#"<a href="{}" target="_blank" rel="noopener" class="widget-link">{}</a>"#,
                            escape_html(parts[0].trim()),
                            escape_html(parts[1].trim())
                        ));
                    }
                }
                links
            }
            _ => format!(r#"<p class="widget-note">{}</p>"#, escape_html(&w.content)),
        };
        html.push_str(&format!(
            r#"<div class="glass-panel dashboard-widget{}"><h3 class="widget-title">{}</h3>{}</div>"#,
            span_class,
            escape_html(&w.title),
            body
        ));
    }
    html.push_str("</section>");
    html
}
