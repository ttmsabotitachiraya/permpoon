//! Database connection configuration stored in `db.json`.
//!
//! All persisted fields are AES-256-GCM encrypted before being written to disk.
//! The loader also supports migrating legacy files where only `username` and
//! `password` were encrypted.

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

// ── Private on-disk types ─────────────────────────────────────────────────────

/// The current structure written to `db.json`.
/// Every field stores AES-256-GCM ciphertext as base64(`nonce || ciphertext`).
#[derive(Debug, Deserialize, Serialize)]
struct DbConfigFile {
    host: String,
    port: String,
    database: String,
    username: String,
    password: String,
}

/// Legacy on-disk structure where only username/password were encrypted.
#[derive(Debug, Deserialize)]
struct LegacyDbConfigFile {
    host: String,
    port: u16,
    database: String,
    username: String,
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

fn default_db_config() -> DbConfig {
    DbConfig {
        host: "127.0.0.1".to_string(),
        port: 3306,
        database: "hos".to_string(),
        username: String::new(),
        password: String::new(),
    }
}

fn encrypt_db_config(config: &DbConfig) -> Result<DbConfigFile, String> {
    Ok(DbConfigFile {
        host: crypto::encrypt(&config.host)?,
        port: crypto::encrypt(&config.port.to_string())?,
        database: crypto::encrypt(&config.database)?,
        username: crypto::encrypt(&config.username)?,
        password: crypto::encrypt(&config.password)?,
    })
}

fn decrypt_current_db_config(on_disk: DbConfigFile) -> Result<DbConfig, String> {
    let port = crypto::decrypt(&on_disk.port)?
        .parse::<u16>()
        .map_err(|e| format!("Invalid decrypted port value: {e}"))?;

    Ok(DbConfig {
        host: crypto::decrypt(&on_disk.host)?,
        port,
        database: crypto::decrypt(&on_disk.database)?,
        username: crypto::decrypt(&on_disk.username)?,
        password: crypto::decrypt(&on_disk.password)?,
    })
}

fn decrypt_legacy_db_config(on_disk: LegacyDbConfigFile) -> Result<DbConfig, String> {
    Ok(DbConfig {
        host: on_disk.host,
        port: on_disk.port,
        database: on_disk.database,
        username: crypto::decrypt(&on_disk.username)?,
        password: crypto::decrypt(&on_disk.password)?,
    })
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Saves the database configuration to `db.json`.
///
/// All fields are encrypted with AES-256-GCM before writing.
#[tauri::command]
pub fn save_db_config(app: tauri::AppHandle, config: DbConfig) -> Result<(), String> {
    let on_disk = encrypt_db_config(&config)?;
    let json = serde_json::to_string_pretty(&on_disk).map_err(|e| e.to_string())?;
    std::fs::write(db_json_path(&app), json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Loads the database configuration from `db.json`, decrypting every field.
///
/// Returns a default (empty) configuration when the file does not yet exist.
/// Legacy partially encrypted files are transparently migrated on load.
#[tauri::command]
pub fn load_db_config(app: tauri::AppHandle) -> Result<DbConfig, String> {
    let path = db_json_path(&app);
    if !path.exists() {
        return Ok(default_db_config());
    }

    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let config = match serde_json::from_str::<DbConfigFile>(&raw) {
        Ok(on_disk) => decrypt_current_db_config(on_disk),
        Err(current_err) => match serde_json::from_str::<LegacyDbConfigFile>(&raw) {
            Ok(legacy) => {
                let config = decrypt_legacy_db_config(legacy)?;
                save_db_config(app.clone(), config.clone())?;
                Ok(config)
            }
            Err(legacy_err) => Err(format!(
                "Failed to parse db.json as current format ({current_err}) or legacy format ({legacy_err})"
            )),
        },
    }?;

    Ok(config)
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
        let _ = format!("{c:?}");
    }

    #[test]
    fn encrypt_and_decrypt_all_fields_roundtrip() {
        let original = DbConfig {
            host: "10.0.0.12".into(),
            port: 3307,
            database: "hospital".into(),
            username: "admin".into(),
            password: "super-secret".into(),
        };

        let encrypted = encrypt_db_config(&original).unwrap();
        assert_ne!(encrypted.host, original.host);
        assert_ne!(encrypted.port, original.port.to_string());
        assert_ne!(encrypted.database, original.database);
        assert_ne!(encrypted.username, original.username);
        assert_ne!(encrypted.password, original.password);

        let decrypted = decrypt_current_db_config(encrypted).unwrap();
        assert_eq!(decrypted.host, original.host);
        assert_eq!(decrypted.port, original.port);
        assert_eq!(decrypted.database, original.database);
        assert_eq!(decrypted.username, original.username);
        assert_eq!(decrypted.password, original.password);
    }

    #[test]
    fn decrypt_legacy_format_roundtrip() {
        let legacy = LegacyDbConfigFile {
            host: "127.0.0.1".into(),
            port: 3306,
            database: "hos".into(),
            username: crypto::encrypt("root").unwrap(),
            password: crypto::encrypt("pw").unwrap(),
        };

        let decrypted = decrypt_legacy_db_config(legacy).unwrap();
        assert_eq!(decrypted.host, "127.0.0.1");
        assert_eq!(decrypted.port, 3306);
        assert_eq!(decrypted.database, "hos");
        assert_eq!(decrypted.username, "root");
        assert_eq!(decrypted.password, "pw");
    }
}
