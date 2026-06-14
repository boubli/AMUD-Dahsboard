use crate::settings::SECRET_SETTING_KEYS;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::OnceLock;

const ENC_PREFIX: &str = "enc:v1:";
static SECRETS_KEY: OnceLock<[u8; 32]> = OnceLock::new();

pub(crate) fn encrypted_setting_key(key: &str) -> bool {
    SECRET_SETTING_KEYS.contains(&key) || key == "agent_shared_secret"
}

pub fn init_secrets_key(db_path: &str) -> Result<(), String> {
    let key = load_or_create_key(db_path)?;
    SECRETS_KEY
        .set(key)
        .map_err(|_| "secrets key already initialized".to_string())
}

fn key_bytes() -> &'static [u8; 32] {
    SECRETS_KEY
        .get()
        .expect("AMUD secrets key not initialized — call init_secrets_key at startup")
}

pub(crate) fn secrets_key_path(db_path: &str) -> std::path::PathBuf {
    Path::new(db_path)
        .parent()
        .map(|p| p.join(".amud-secrets-key"))
        .unwrap_or_else(|| Path::new(".amud-secrets-key").to_path_buf())
}

fn load_or_create_key(db_path: &str) -> Result<[u8; 32], String> {
    if let Ok(raw) = std::env::var("AMUD_SECRETS_KEY") {
        if !raw.trim().is_empty() {
            return derive_key_material(raw.trim());
        }
    }

    let key_path = secrets_key_path(db_path);
    if key_path.is_file() {
        let bytes = std::fs::read_to_string(&key_path)
            .map_err(|e| format!("read secrets key file {}: {e}", key_path.display()))?;
        return parse_key_material(bytes.trim());
    }

    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).map_err(|e| format!("generate secrets key: {e}"))?;
    let encoded = URL_SAFE_NO_PAD.encode(key);
    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&key_path, format!("{encoded}\n"))
        .map_err(|e| format!("write secrets key file {}: {e}", key_path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600)).ok();
    }
    eprintln!(
        "AMUD SECURITY: Generated secrets encryption key at {}. Back up this file with your database, or set AMUD_SECRETS_KEY.",
        key_path.display()
    );
    Ok(key)
}

fn parse_key_material(raw: &str) -> Result<[u8; 32], String> {
    if let Ok(decoded) = URL_SAFE_NO_PAD.decode(raw) {
        if decoded.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&decoded);
            return Ok(key);
        }
    }
    if raw.len() == 64 && raw.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut key = [0u8; 32];
        hex::decode_to_slice(raw, &mut key).map_err(|e| format!("invalid hex secrets key: {e}"))?;
        return Ok(key);
    }
    derive_key_material(raw)
}

fn derive_key_material(raw: &str) -> Result<[u8; 32], String> {
    let hash = Sha256::digest(raw.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    Ok(key)
}

pub(crate) fn encrypt_setting_for_db(key: &str, value: &str) -> String {
    if value.is_empty() || !encrypted_setting_key(key) {
        return value.to_string();
    }
    encrypt_value(value).unwrap_or_else(|_| value.to_string())
}

pub(crate) fn decrypt_setting_from_db(key: &str, value: &str) -> String {
    if !encrypted_setting_key(key) {
        return value.to_string();
    }
    decrypt_value(value).unwrap_or_else(|_| String::new())
}

pub fn encrypt_value(plaintext: &str) -> Result<String, &'static str> {
    let cipher = ChaCha20Poly1305::new_from_slice(key_bytes()).map_err(|_| "cipher")?;
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes).map_err(|_| "rng")?;
    let nonce = Nonce::from(nonce_bytes);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| "encrypt")?;
    let mut packed = nonce_bytes.to_vec();
    packed.extend(ciphertext);
    Ok(format!("{ENC_PREFIX}{}", URL_SAFE_NO_PAD.encode(packed)))
}

pub fn decrypt_value(stored: &str) -> Result<String, &'static str> {
    if !stored.starts_with(ENC_PREFIX) {
        return Ok(stored.to_string());
    }
    let packed = URL_SAFE_NO_PAD
        .decode(&stored[ENC_PREFIX.len()..])
        .map_err(|_| "b64")?;
    if packed.len() <= 12 {
        return Err("short");
    }
    let (nonce_bytes, ciphertext) = packed.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = ChaCha20Poly1305::new_from_slice(key_bytes()).map_err(|_| "cipher")?;
    let plain = cipher.decrypt(nonce, ciphertext).map_err(|_| "decrypt")?;
    String::from_utf8(plain).map_err(|_| "utf8")
}

pub(crate) fn migrate_plaintext_secrets(db: &Connection) -> usize {
    let mut migrated = 0usize;
    let Ok(mut stmt) = db.prepare("SELECT key, value FROM settings") else {
        return migrated;
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }) else {
        return migrated;
    };
    for row in rows.flatten() {
        let (key, value) = row;
        if !encrypted_setting_key(&key) || value.is_empty() || value.starts_with(ENC_PREFIX) {
            continue;
        }
        let encrypted = encrypt_setting_for_db(&key, &value);
        if db
            .execute(
                "UPDATE settings SET value = ? WHERE key = ?",
                params![encrypted, key],
            )
            .is_ok()
        {
            migrated += 1;
        }
    }
    migrated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_at_rest_roundtrip_and_legacy() {
        init_secrets_key("data/test-amud.db").unwrap();
        let plain = "PVEAPIToken=root@pam!token=abc123";
        let enc = encrypt_value(plain).unwrap();
        assert!(enc.starts_with(ENC_PREFIX));
        assert_ne!(enc, plain);
        assert_eq!(decrypt_value(&enc).unwrap(), plain);
        assert_eq!(decrypt_value("legacy-token").unwrap(), "legacy-token");
    }
}
