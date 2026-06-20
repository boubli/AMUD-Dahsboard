use crate::models::App;

pub(crate) fn contains_service_token(haystack: &str, token: &str) -> bool {
    let token = token.to_lowercase();
    let haystack = haystack.to_lowercase();
    if haystack == token {
        return true;
    }
    haystack
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|part| part == token)
}

fn app_field_matches_service(field: &str, tokens: &[&str]) -> bool {
    tokens.iter().any(|t| contains_service_token(field, t))
}

pub(crate) fn is_plex_app(app: &App) -> bool {
    const TOKENS: &[&str] = &["plex"];
    app_field_matches_service(&app.name, TOKENS)
        || app_field_matches_service(&app.url, TOKENS)
        || app_field_matches_service(&app.icon, TOKENS)
}

pub(crate) fn is_jellyfin_app(app: &App) -> bool {
    const TOKENS: &[&str] = &["jellyfin", "emby"];
    app_field_matches_service(&app.name, TOKENS)
        || app_field_matches_service(&app.url, TOKENS)
        || app_field_matches_service(&app.icon, TOKENS)
}
