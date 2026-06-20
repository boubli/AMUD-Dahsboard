use super::imports::*;
use reqwest;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::Write;

#[derive(Serialize)]
struct VersionResponse {
    current: String,
    latest: String,
    update_available: bool,
    release_url: String,
    release_notes: String,
    release_date: String,
    deployment_type: String,
    agent_connected: bool,
}

#[derive(Deserialize, Clone)]
pub(crate) struct GitHubAsset {
    pub(crate) name: String,
    pub(crate) browser_download_url: String,
}

#[derive(Deserialize, Clone)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Clone)]
pub(crate) struct CachedRelease {
    pub(crate) latest: String,
    pub(crate) release_url: String,
    pub(crate) release_notes: String,
    pub(crate) release_date: String,
    pub(crate) fetched_at: std::time::Instant,
    pub(crate) assets: Vec<GitHubAsset>,
}

pub(crate) static RELEASE_CACHE: std::sync::LazyLock<std::sync::RwLock<Option<CachedRelease>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(None));

static UPDATE_IN_PROGRESS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

struct UpdateProgressGuard;

impl Drop for UpdateProgressGuard {
    fn drop(&mut self) {
        UPDATE_IN_PROGRESS.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

pub(crate) fn semver_newer(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    let c = parse(current);
    let l = parse(latest);
    for i in 0..c.len().max(l.len()) {
        let cv = c.get(i).copied().unwrap_or(0);
        let lv = l.get(i).copied().unwrap_or(0);
        if lv > cv {
            return true;
        }
        if cv > lv {
            return false;
        }
    }
    false
}

async fn fetch_latest_release() -> Result<CachedRelease, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/boubli/AMUD-Dashboard/releases/latest")
        .header("User-Agent", "AMUD-Dashboard")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned status {}", resp.status()));
    }

    let github_release: GitHubRelease = resp.json().await.map_err(|e| e.to_string())?;

    let latest = github_release.tag_name;
    let release_url = github_release.html_url;
    let release_notes_raw = github_release.body.unwrap_or_default();

    // Safely take the first 300 characters
    let release_notes = if release_notes_raw.chars().count() > 300 {
        let mut notes: String = release_notes_raw.chars().take(300).collect();
        notes.push_str("...");
        notes
    } else {
        release_notes_raw
    };

    let release_date_raw = github_release.published_at.unwrap_or_default();
    let release_date = if release_date_raw.len() >= 10 {
        release_date_raw[0..10].to_string()
    } else {
        release_date_raw
    };

    Ok(CachedRelease {
        latest,
        release_url,
        release_notes,
        release_date,
        fetched_at: std::time::Instant::now(),
        assets: github_release.assets,
    })
}

use axum::extract::Query;

#[derive(Deserialize)]
pub struct VersionQuery {
    refresh: Option<bool>,
}

pub async fn system_version_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Query(query): Query<VersionQuery>,
) -> impl IntoResponse {
    if let Some(resp) = check_api_rate_limit(&state, &headers, "system_version", 10, 60) {
        return resp.into_response();
    }

    let session = get_session(&headers, &state.sessions);
    if !session.as_ref().map(|s| s.role == "Admin").unwrap_or(false) {
        return api_json(StatusCode::FORBIDDEN, json!({"error": "Admin required"}));
    }

    let current = option_env!("GIT_TAG").unwrap_or(env!("CARGO_PKG_VERSION"));
    let current_version = if current.starts_with('v') {
        current.to_string()
    } else {
        format!("v{}", current)
    };

    let force_refresh = query.refresh.unwrap_or(false);
    let need_fetch = force_refresh || {
        let cache = RELEASE_CACHE.read().unwrap();
        match &*cache {
            Some(ref cached) => cached.fetched_at.elapsed() > Duration::from_secs(3600),
            None => true,
        }
    };

    if need_fetch {
        match fetch_latest_release().await {
            Ok(new_release) => {
                let mut cache = RELEASE_CACHE.write().unwrap();
                *cache = Some(new_release);
            }
            Err(e) => {
                eprintln!("Failed to fetch latest release: {}", e);
            }
        }
    }

    let (latest, release_url, release_notes, release_date) = {
        let cache = RELEASE_CACHE.read().unwrap();
        match &*cache {
            Some(ref cached) => (
                cached.latest.clone(),
                cached.release_url.clone(),
                cached.release_notes.clone(),
                cached.release_date.clone(),
            ),
            None => (
                current_version.clone(),
                "".to_string(),
                "No release notes available".to_string(),
                "".to_string(),
            ),
        }
    };

    let update_available = semver_newer(&current_version, &latest);

    let deployment_type = if std::env::var("AMUD_ENABLE_PROXMOX").unwrap_or_default() == "true" {
        "proxmox"
    } else if std::path::Path::new("/.dockerenv").exists() {
        "docker"
    } else {
        "native"
    };

    let agent_connected = *state.agent_connected.read().unwrap();

    api_json(
        StatusCode::OK,
        json!(VersionResponse {
            current: current_version,
            latest,
            update_available,
            release_url,
            release_notes,
            release_date,
            deployment_type: deployment_type.to_string(),
            agent_connected,
        }),
    )
}

async fn download_file(
    client: &reqwest::Client,
    url: &str,
    dest_path: &std::path::Path,
) -> Result<String, String> {
    let mut resp = client
        .get(url)
        .header("User-Agent", "AMUD-Dashboard")
        .send()
        .await
        .map_err(|e| format!("Failed to download {}: {}", url, e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to download {}, status code: {}",
            url,
            resp.status()
        ));
    }

    let mut file = std::fs::File::create(dest_path)
        .map_err(|e| format!("Failed to create file at {:?}: {}", dest_path, e))?;

    let mut hasher = Sha256::new();

    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| format!("Error reading chunk: {}", e))?
    {
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk: {}", e))?;
        hasher.update(&chunk);
    }

    let hash_bytes = hasher.finalize();
    Ok(hex::encode(hash_bytes))
}

fn parse_checksums(content: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let hash = parts[0].to_string();
            let file = parts[1].trim_start_matches('*').to_string();
            map.insert(file, hash);
        }
    }
    map
}

async fn perform_update(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    admin_user: &str,
) -> Result<(), String> {
    let current = option_env!("GIT_TAG").unwrap_or(env!("CARGO_PKG_VERSION"));
    let current_version = if current.starts_with('v') {
        current.to_string()
    } else {
        format!("v{}", current)
    };

    let (latest_version, assets) = {
        let cache = RELEASE_CACHE.read().unwrap();
        match &*cache {
            Some(ref cached) => (cached.latest.clone(), cached.assets.clone()),
            None => {
                return Err("No cached release found. Please check for updates first.".to_string())
            }
        }
    };

    let server_asset = assets.iter().find(|a| a.name == "amud-server");
    let ui_asset = assets.iter().find(|a| a.name == "ui.tar.gz");
    let sums_asset = assets.iter().find(|a| a.name == "SHA256SUMS");

    let (server_url, ui_url, sums_url) = match (server_asset, ui_asset, sums_asset) {
        (Some(s), Some(u), Some(sm)) => (
            &s.browser_download_url,
            &u.browser_download_url,
            &sm.browser_download_url,
        ),
        _ => {
            return Err(
                "Required release assets (amud-server, ui.tar.gz, SHA256SUMS) are missing."
                    .to_string(),
            )
        }
    };

    // 1. Audit update started
    let headers_clone = headers.clone();
    let admin_user_clone = admin_user.to_string();
    let target = format!("{} -> {}", current_version, latest_version);
    with_db(state.db.clone(), move |db| {
        record_audit_blocking(
            db,
            &headers_clone,
            &admin_user_clone,
            "system_update_started",
            &target,
            "initiated by admin",
        );
    })
    .await;

    // Create temp directory inside workspace
    let tmp_dir = std::path::PathBuf::from("data/update_tmp");
    std::fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("Failed to create temporary directory: {}", e))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create reqwest client: {}", e))?;

    let server_path = tmp_dir.join("amud-server");
    let ui_path = tmp_dir.join("ui.tar.gz");
    let sums_path = tmp_dir.join("SHA256SUMS");

    // 2. Download files & calculate hash
    let server_hash = download_file(&client, server_url, &server_path).await?;
    let ui_hash = download_file(&client, ui_url, &ui_path).await?;
    let _ = download_file(&client, sums_url, &sums_path).await?;

    // 3. Verify checksums
    let sums_content = std::fs::read_to_string(&sums_path)
        .map_err(|e| format!("Failed to read SHA256SUMS file: {}", e))?;
    let checksum_map = parse_checksums(&sums_content);

    let expected_server_hash = checksum_map
        .get("amud-server")
        .ok_or_else(|| "amud-server missing from SHA256SUMS".to_string())?;
    let expected_ui_hash = checksum_map
        .get("ui.tar.gz")
        .ok_or_else(|| "ui.tar.gz missing from SHA256SUMS".to_string())?;

    if &server_hash != expected_server_hash {
        return Err(format!(
            "Checksum mismatch for amud-server. Expected {}, got {}",
            expected_server_hash, server_hash
        ));
    }
    if &ui_hash != expected_ui_hash {
        return Err(format!(
            "Checksum mismatch for ui.tar.gz. Expected {}, got {}",
            expected_ui_hash, ui_hash
        ));
    }

    // 4. Backup current binary
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    let backup_exe = current_exe.with_extension("bak");

    if backup_exe.exists() {
        let _ = std::fs::remove_file(&backup_exe);
    }
    std::fs::copy(&current_exe, &backup_exe)
        .map_err(|e| format!("Failed to backup current executable: {}", e))?;

    // 5. Replace current executable
    std::fs::rename(&server_path, &current_exe).map_err(|e| {
        // Rollback
        let _ = std::fs::copy(&backup_exe, &current_exe);
        format!("Failed to replace executable: {}", e)
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&current_exe) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&current_exe, perms);
        }
    }

    // 6. Extract UI assets
    let tar_status = std::process::Command::new("tar")
        .args(["-xzf", ui_path.to_str().unwrap(), "-C", "."])
        .status();

    match tar_status {
        Ok(status) if status.success() => {}
        Ok(status) => {
            // Rollback executable
            let _ = std::fs::copy(&backup_exe, &current_exe);
            return Err(format!(
                "tar extraction failed with exit code: {:?}",
                status.code()
            ));
        }
        Err(e) => {
            // Rollback executable
            let _ = std::fs::copy(&backup_exe, &current_exe);
            return Err(format!("Failed to run tar command: {}", e));
        }
    }

    // Clean up temporary files
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // 7. Audit update complete
    let headers_clone = headers.clone();
    let admin_user_clone = admin_user.to_string();
    let target = latest_version.clone();
    with_db(state.db.clone(), move |db| {
        record_audit_blocking(
            db,
            &headers_clone,
            &admin_user_clone,
            "system_update_complete",
            &target,
            "server binary + UI assets updated",
        );
    })
    .await;

    Ok(())
}

pub async fn system_update_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    let session = get_session(&headers, &state.sessions);
    let admin_user = match session {
        Some(ref s) if s.role == "Admin" => s.username.clone(),
        _ => return api_json(StatusCode::FORBIDDEN, json!({"error": "Admin required"})),
    };

    if !validate_csrf(&headers, &state.sessions, Some(&form)) {
        return csrf_forbidden_response().into_response();
    }

    let deployment_type = if std::env::var("AMUD_ENABLE_PROXMOX").unwrap_or_default() == "true" {
        "proxmox"
    } else if std::path::Path::new("/.dockerenv").exists() {
        "docker"
    } else {
        "native"
    };

    if deployment_type == "docker" {
        return api_json(
            StatusCode::BAD_REQUEST,
            json!({"error": "Docker deployment cannot be auto-updated"}),
        );
    }

    if UPDATE_IN_PROGRESS.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return api_json(
            StatusCode::CONFLICT,
            json!({"error": "Update already in progress"}),
        );
    }

    let _guard = UpdateProgressGuard;

    match perform_update(&state, &headers, &admin_user).await {
        Ok(()) => {
            // Spawn delayed exit so process restarts
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                std::process::exit(0);
            });

            api_json(
                StatusCode::OK,
                json!({
                    "status": "ok",
                    "message": "Update applied successfully. Server restarting..."
                }),
            )
        }
        Err(err_msg) => {
            // Audit update failed
            let headers_clone = headers.clone();
            let admin_user_clone = admin_user.clone();
            let err_msg_clone = err_msg.clone();

            let latest_version = {
                let cache = RELEASE_CACHE.read().unwrap();
                cache.as_ref().map(|c| c.latest.clone()).unwrap_or_default()
            };

            with_db(state.db.clone(), move |db| {
                record_audit_blocking(
                    db,
                    &headers_clone,
                    &admin_user_clone,
                    "system_update_failed",
                    &latest_version,
                    &err_msg_clone,
                );
            })
            .await;

            api_json(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "status": "error",
                    "message": format!("Update failed: {}", err_msg)
                }),
            )
        }
    }
}

pub async fn health_handler() -> impl IntoResponse {
    api_json(StatusCode::OK, json!({"status": "UP"}))
}

pub async fn ready_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_ok = with_db(state.db.clone(), |conn| {
        conn.execute("SELECT 1", []).is_ok()
    })
    .await;

    if db_ok {
        api_json(StatusCode::OK, json!({"status": "READY"}))
    } else {
        api_json(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"status": "NOT_READY", "error": "Database query failed"}),
        )
    }
}
