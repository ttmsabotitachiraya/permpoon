//! Database connection configuration stored in `db.json`.
//!
//! `username` and `password` are AES-256-GCM encrypted before being written to
//! disk.  All other fields are stored as plain JSON because they are not
//! considered sensitive.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

use crate::crypto;

// ── Public types ──────────────────────────────────────────────────────────────

/// MySQL connection parameters (plaintext in memory, never written to disk as-is).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DbConfig {
    /// MySQL host address.
    pub host: String,
    /// MySQL port number.
    pub port: u16,
    /// Database name.
    pub database: String,
    /// Plaintext username (decrypted on load, encrypted on save).
    pub username: String,
    /// Plaintext password (decrypted on load, encrypted on save).
    pub password: String,
}

// ── Private on-disk type ──────────────────────────────────────────────────────

/// The actual structure written to `db.json`.
/// `username` and `password` hold AES-256-GCM base64-encoded ciphertext.
#[derive(Debug, Deserialize, Serialize)]
struct DbConfigFile {
    host: String,
    port: u16,
    database: String,
    /// AES-256-GCM ciphertext, base64-encoded (`nonce || ct`).
    username: String,
    /// AES-256-GCM ciphertext, base64-encoded (`nonce || ct`).
    password: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns the path to `db.json` inside the app data directory.
pub fn db_json_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data directory");
    std::fs::create_dir_all(&dir).ok();
    dir.join("db.json")
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Saves the database configuration to `db.json`.
///
/// `username` and `password` are encrypted with AES-256-GCM before writing.
#[tauri::command]
pub fn save_db_config(app: tauri::AppHandle, config: DbConfig) -> Result<(), String> {
    let enc_user = crypto::encrypt(&config.username)?;
    let enc_pass = crypto::encrypt(&config.password)?;

    let on_disk = DbConfigFile {
        host: config.host,
        port: config.port,
        database: config.database,
        username: enc_user,
        password: enc_pass,
    };

    let json = serde_json::to_string_pretty(&on_disk).map_err(|e| e.to_string())?;
    std::fs::write(db_json_path(&app), json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Loads the database configuration from `db.json`, decrypting credentials.
///
/// Returns a default (empty) configuration when the file does not yet exist.
#[tauri::command]
pub fn load_db_config(app: tauri::AppHandle) -> Result<DbConfig, String> {
    let path = db_json_path(&app);
    if !path.exists() {
        return Ok(DbConfig {
            host: "127.0.0.1".to_string(),
            port: 3306,
            database: "hos".to_string(),
            username: String::new(),
            password: String::new(),
        });
    }

    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let on_disk: DbConfigFile = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    let username = crypto::decrypt(&on_disk.username).unwrap_or_default();
    let password = crypto::decrypt(&on_disk.password).unwrap_or_default();

    Ok(DbConfig {
        host: on_disk.host,
        port: on_disk.port,
        database: on_disk.database,
        username,
        password,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_config_clone_and_debug() {
        let c = DbConfig {
            host: "localhost".into(),
            port: 3306,
            database: "hos".into(),
            username: "root".into(),
            password: "secret".into(),
        };
        let c2 = c.clone();
        assert_eq!(c.host, c2.host);
        assert_eq!(c.username, c2.username);
        // Debug should not panic
        let _ = format!("{c:?}");
    }
}
