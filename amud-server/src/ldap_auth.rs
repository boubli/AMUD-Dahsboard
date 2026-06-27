//! LDAP bind authentication (Homarr parity).

use ldap3::{LdapConnAsync, Scope, SearchEntry};

pub struct LdapSettings {
    pub enabled: bool,
    pub url: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub base_dn: String,
    pub user_filter: String,
}

pub fn ldap_settings_from_map(
    settings: &std::collections::HashMap<String, String>,
) -> LdapSettings {
    LdapSettings {
        enabled: settings
            .get("ldap_enabled")
            .map(|s| s == "1")
            .unwrap_or(false),
        url: settings.get("ldap_url").cloned().unwrap_or_default(),
        bind_dn: settings.get("ldap_bind_dn").cloned().unwrap_or_default(),
        bind_password: settings
            .get("ldap_bind_password")
            .cloned()
            .unwrap_or_default(),
        base_dn: settings.get("ldap_base_dn").cloned().unwrap_or_default(),
        user_filter: settings
            .get("ldap_user_filter")
            .cloned()
            .unwrap_or_else(|| "(uid={username})".to_string()),
    }
}

pub async fn ldap_authenticate(
    cfg: &LdapSettings,
    username: &str,
    password: &str,
) -> Result<(), String> {
    if !cfg.enabled || cfg.url.is_empty() || cfg.base_dn.is_empty() {
        return Err("LDAP not configured".into());
    }
    let (conn, mut ldap) = LdapConnAsync::new(cfg.url.as_str())
        .await
        .map_err(|e| format!("LDAP connect: {e}"))?;
    ldap3::drive!(conn);
    if !cfg.bind_dn.is_empty() {
        ldap.simple_bind(&cfg.bind_dn, &cfg.bind_password)
            .await
            .map_err(|e| format!("LDAP service bind: {e}"))?
            .success()
            .map_err(|e| format!("LDAP service bind failed: {e}"))?;
    }
    let filter = cfg.user_filter.replace("{username}", username);
    let (rs, _) = ldap
        .search(&cfg.base_dn, Scope::Subtree, &filter, vec!["dn"])
        .await
        .map_err(|e| format!("LDAP search: {e}"))?
        .success()
        .map_err(|e| format!("LDAP search failed: {e}"))?;
    let entry = rs
        .into_iter()
        .next()
        .map(SearchEntry::construct)
        .map(|e| e.dn)
        .filter(|dn| !dn.is_empty())
        .ok_or_else(|| "LDAP user not found".to_string())?;
    ldap.simple_bind(&entry, password)
        .await
        .map_err(|e| format!("LDAP user bind: {e}"))?
        .success()
        .map_err(|e| format!("LDAP auth failed: {e}"))?;
    Ok(())
}
