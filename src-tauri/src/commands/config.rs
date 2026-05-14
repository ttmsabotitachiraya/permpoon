//! Application settings stored in `setting.db` (SQLite).
//!
//! This module owns the SQLite connection helpers and all commands that
//! read/write pttype groups, icode configs, and generic key-value app config.
//! Database connection credentials are **not** stored here; see
//! [`super::db_config`] and `db.json`.

use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Row;
use std::str::FromStr;
use tauri::Manager;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns the path of `setting.db` inside the application data directory.
pub(crate) fn get_setting_db_path(app: &tauri::AppHandle) -> String {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data directory");
    std::fs::create_dir_all(&dir).ok();
    dir.join("setting.db").to_string_lossy().into_owned()
}

/// Opens (or creates) a SQLite pool at `db_path`.
pub(crate) async fn get_pool(db_path: &str) -> Result<sqlx::SqlitePool, String> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{db_path}?mode=rwc"))
        .map_err(|e| e.to_string())?;
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .map_err(|e| e.to_string())
}

/// Ensures `column` exists in `table`; adds it with `definition` if absent.
async fn ensure_column(
    pool: &sqlx::SqlitePool,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), String> {
    let pragma = format!("PRAGMA table_info({table})");
    let rows = sqlx::query(&pragma)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    let exists = rows
        .iter()
        .any(|row| row.get::<String, _>("name") == column);

    if !exists {
        let alter = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        sqlx::query(&alter)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Initialisation ────────────────────────────────────────────────────────────

/// Initialises the `setting.db` SQLite database.
///
/// Migrates the old `permpoon_config.sqlite` file if it still exists, and
/// migrates any MySQL credentials stored in the old `app_config` table into
/// `db.json` (encrypted).
pub async fn init_sqlite(app: &tauri::AppHandle) -> Result<(), String> {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data directory");
    std::fs::create_dir_all(&dir).ok();

    // ── File migration: permpoon_config.sqlite → setting.db ───────────────────
    let old_path = dir.join("permpoon_config.sqlite");
    let new_path = dir.join("setting.db");
    if old_path.exists() && !new_path.exists() {
        std::fs::rename(&old_path, &new_path)
            .map_err(|e| format!("migration rename failed: {e}"))?;
    }

    let db_path = get_setting_db_path(app);
    let pool = get_pool(&db_path).await?;

    // ── DDL ───────────────────────────────────────────────────────────────────
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS app_config (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pttype_group (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            alias        TEXT    NOT NULL UNIQUE,
            hipdata_code TEXT    NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS icode_config (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            icode           TEXT    NOT NULL UNIQUE,
            service_name    TEXT    NOT NULL,
            is_enabled      INTEGER DEFAULT 1,
            age_min         INTEGER,
            age_max         INTEGER,
            gender_restrict TEXT,
            freq_type       TEXT,
            freq_value      INTEGER,
            department      TEXT,
            created_at      TEXT    DEFAULT (datetime('now')),
            updated_at      TEXT    DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS icode_pttype_map (
            icode_config_id INTEGER NOT NULL
                REFERENCES icode_config(id) ON DELETE CASCADE,
            pttype_group_id INTEGER NOT NULL
                REFERENCES pttype_group(id)  ON DELETE CASCADE,
            PRIMARY KEY (icode_config_id, pttype_group_id)
        )",
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    ensure_column(&pool, "icode_config", "department", "TEXT").await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS department_config (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            depcode         TEXT    NOT NULL UNIQUE,
            department      TEXT    NOT NULL,
            is_enabled      INTEGER DEFAULT 1,
            created_at      TEXT    DEFAULT (datetime('now')),
            updated_at      TEXT    DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // ── Credential migration: app_config → db.json ────────────────────────────
    let db_json = super::db_config::db_json_path(app);
    if !db_json.exists() {
        migrate_credentials_to_db_json(app, &pool).await;
    }

    pool.close().await;
    Ok(())
}

/// Reads any MySQL credentials stored in the old `app_config` rows and saves
/// them to `db.json` (with encrypted username/password).
async fn migrate_credentials_to_db_json(app: &tauri::AppHandle, pool: &sqlx::SqlitePool) {
    let read = |k: &'static str| async move {
        sqlx::query("SELECT value FROM app_config WHERE key = ?")
            .bind(k)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .map(|r| r.get::<String, _>("value"))
    };

    let host = read("mysql_host")
        .await
        .unwrap_or_else(|| "127.0.0.1".into());
    let port: u16 = read("mysql_port")
        .await
        .and_then(|v| v.parse().ok())
        .unwrap_or(3306);
    let database = read("mysql_database").await.unwrap_or_else(|| "hos".into());
    let username = read("mysql_username").await.unwrap_or_default();
    let password = read("mysql_password").await.unwrap_or_default();

    // Only migrate if we have at least a non-default host or non-empty credentials
    if username.is_empty() && password.is_empty() && host == "127.0.0.1" {
        return;
    }

    let cfg = super::db_config::DbConfig {
        host,
        port,
        database,
        username,
        password,
    };
    // Ignore errors: best-effort migration
    let _ = super::db_config::save_db_config(app.clone(), cfg);
}

// ── Generic key-value config ──────────────────────────────────────────────────

/// Stores a key-value pair in the `app_config` table.
#[tauri::command]
pub async fn save_config(app: tauri::AppHandle, key: String, value: String) -> Result<(), String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    sqlx::query("INSERT OR REPLACE INTO app_config (key, value) VALUES (?, ?)")
        .bind(&key)
        .bind(&value)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(())
}

/// Retrieves a single key-value entry from `app_config`.
#[tauri::command]
pub async fn get_config(app: tauri::AppHandle, key: String) -> Result<Option<String>, String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    let row = sqlx::query("SELECT value FROM app_config WHERE key = ?")
        .bind(&key)
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(row.map(|r| r.get::<String, _>("value")))
}

/// Returns all key-value entries from `app_config` as a JSON object.
#[tauri::command]
pub async fn get_all_config(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    let rows = sqlx::query("SELECT key, value FROM app_config")
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    let mut map = serde_json::Map::new();
    for row in rows {
        map.insert(
            row.get::<String, _>("key"),
            serde_json::Value::String(row.get::<String, _>("value")),
        );
    }
    Ok(serde_json::Value::Object(map))
}

// ── Pttype group commands ─────────────────────────────────────────────────────

/// Adds a new pttype group, ignoring duplicates.
#[tauri::command]
pub async fn add_pttype_group(
    app: tauri::AppHandle,
    alias: String,
    hipdata_code: String,
) -> Result<i64, String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    let result =
        sqlx::query("INSERT OR IGNORE INTO pttype_group (alias, hipdata_code) VALUES (?, ?)")
            .bind(&alias)
            .bind(&hipdata_code)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(result.last_insert_rowid())
}

/// Removes a pttype group and all its icode mappings.
#[tauri::command]
pub async fn remove_pttype_group(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    sqlx::query("DELETE FROM icode_pttype_map WHERE pttype_group_id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM pttype_group WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(())
}

/// Returns all pttype groups with their associated icode counts.
#[tauri::command]
pub async fn get_all_pttype_groups(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    let rows = sqlx::query(
        "SELECT g.id, g.alias, g.hipdata_code,
                COUNT(m.icode_config_id) AS icode_count
         FROM pttype_group g
         LEFT JOIN icode_pttype_map m ON m.pttype_group_id = g.id
         GROUP BY g.id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;
    pool.close().await;
    let arr: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id":           r.get::<i64,    _>("id"),
                "alias":        r.get::<String, _>("alias"),
                "hipdata_code": r.get::<String, _>("hipdata_code"),
                "icode_count":  r.get::<i64,    _>("icode_count"),
            })
        })
        .collect();
    Ok(serde_json::Value::Array(arr))
}

// ── Icode config commands ─────────────────────────────────────────────────────

/// Input payload for [`save_icode_config`].
#[derive(Debug, Deserialize, Serialize)]
pub struct IcodeConfigInput {
    /// HOSxP item code.
    pub icode: String,
    /// Human-readable service name.
    pub service_name: String,
    /// Whether this rule is currently active.
    pub is_enabled: bool,
    /// Minimum patient age (inclusive), or `None` for no restriction.
    pub age_min: Option<i64>,
    /// Maximum patient age (inclusive), or `None` for no restriction.
    pub age_max: Option<i64>,
    /// Gender restriction (`"M"`, `"F"`), or `None` for no restriction.
    pub gender_restrict: Option<String>,
    /// Frequency type (`"days"`, `"months"`, etc.), or `None`.
    pub freq_type: Option<String>,
    /// Numeric threshold for the frequency rule, or `None`.
    pub freq_value: Option<i64>,
    /// List of pttype group IDs this rule applies to.
    pub pttype_group_ids: Vec<i64>,
    /// Optional department / clinic name.
    pub department: Option<String>,
}

/// Upserts an icode configuration record and rebuilds its pttype-group mappings.
#[tauri::command]
pub async fn save_icode_config(
    app: tauri::AppHandle,
    input: IcodeConfigInput,
) -> Result<i64, String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    let is_enabled: i64 = i64::from(input.is_enabled);
    let result = sqlx::query(
        "INSERT INTO icode_config
             (icode, service_name, is_enabled, age_min, age_max,
              gender_restrict, freq_type, freq_value, department)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(icode) DO UPDATE SET
             service_name    = excluded.service_name,
             is_enabled      = excluded.is_enabled,
             age_min         = excluded.age_min,
             age_max         = excluded.age_max,
             gender_restrict = excluded.gender_restrict,
             freq_type       = excluded.freq_type,
             freq_value      = excluded.freq_value,
             department      = excluded.department,
             updated_at      = datetime('now')",
    )
    .bind(&input.icode)
    .bind(&input.service_name)
    .bind(is_enabled)
    .bind(input.age_min)
    .bind(input.age_max)
    .bind(&input.gender_restrict)
    .bind(&input.freq_type)
    .bind(input.freq_value)
    .bind(&input.department)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();
    let actual_id: i64 = if id == 0 {
        sqlx::query("SELECT id FROM icode_config WHERE icode = ?")
            .bind(&input.icode)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?
            .get("id")
    } else {
        id
    };

    // Rebuild pttype mappings
    sqlx::query("DELETE FROM icode_pttype_map WHERE icode_config_id = ?")
        .bind(actual_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    for gid in &input.pttype_group_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO icode_pttype_map (icode_config_id, pttype_group_id)
             VALUES (?, ?)",
        )
        .bind(actual_id)
        .bind(gid)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    pool.close().await;
    Ok(actual_id)
}

/// Toggles the `is_enabled` flag of an icode config.
#[tauri::command]
pub async fn toggle_icode_enabled(
    app: tauri::AppHandle,
    id: i64,
    is_enabled: bool,
) -> Result<(), String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    sqlx::query(
        "UPDATE icode_config
         SET is_enabled = ?, updated_at = datetime('now')
         WHERE id = ?",
    )
    .bind(i64::from(is_enabled))
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(())
}

/// Removes an icode config and all its pttype-group mappings.
#[tauri::command]
pub async fn remove_icode_config(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    // Delete child mappings first (FK safety)
    sqlx::query("DELETE FROM icode_pttype_map WHERE icode_config_id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM icode_config WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(())
}

/// Returns all icode configs with their linked pttype-group IDs.
#[tauri::command]
pub async fn get_all_icode_configs(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    let rows = sqlx::query(
        "SELECT c.id, c.icode, c.service_name, c.department,
                c.is_enabled, c.age_min, c.age_max,
                c.gender_restrict, c.freq_type, c.freq_value,
                GROUP_CONCAT(m.pttype_group_id) AS group_ids
         FROM icode_config c
         LEFT JOIN icode_pttype_map m ON m.icode_config_id = c.id
         GROUP BY c.id
         ORDER BY c.department, c.service_name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;
    pool.close().await;

    let arr: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let ids_str: Option<String> = r.get("group_ids");
            let group_ids: Vec<i64> = ids_str
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| s.parse::<i64>().ok())
                .collect();
            serde_json::json!({
                "id":             r.get::<i64,           _>("id"),
                "icode":          r.get::<String,        _>("icode"),
                "service_name":   r.get::<String,        _>("service_name"),
                "department":     r.get::<Option<String>,_>("department"),
                "is_enabled":     r.get::<i64,           _>("is_enabled") == 1,
                "age_min":        r.get::<Option<i64>,   _>("age_min"),
                "age_max":        r.get::<Option<i64>,   _>("age_max"),
                "gender_restrict":r.get::<Option<String>,_>("gender_restrict"),
                "freq_type":      r.get::<Option<String>,_>("freq_type"),
                "freq_value":     r.get::<Option<i64>,   _>("freq_value"),
                "pttype_group_ids": group_ids,
            })
        })
        .collect();
    Ok(serde_json::Value::Array(arr))
}

// ── Department config commands ─────────────────────────────────────────────────

/// Input payload for saving department config.
#[derive(Debug, Deserialize, Serialize)]
pub struct DepartmentConfigInput {
    /// HOSxP department code (depcode from kskdepartment).
    pub depcode: String,
    /// Department name from kskdepartment.
    pub department: String,
    /// Whether this department is enabled for search.
    pub is_enabled: bool,
}

/// Upserts a department configuration record.
#[tauri::command]
pub async fn save_department_config(
    app: tauri::AppHandle,
    input: DepartmentConfigInput,
) -> Result<i64, String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    let is_enabled: i64 = i64::from(input.is_enabled);
    let result = sqlx::query(
        "INSERT INTO department_config (depcode, department, is_enabled)
         VALUES (?, ?, ?)
         ON CONFLICT(depcode) DO UPDATE SET
             department = excluded.department,
             is_enabled = excluded.is_enabled,
             updated_at = datetime('now')",
    )
    .bind(&input.depcode)
    .bind(&input.department)
    .bind(is_enabled)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let id = result.last_insert_rowid();
    let actual_id: i64 = if id == 0 {
        sqlx::query("SELECT id FROM department_config WHERE depcode = ?")
            .bind(&input.depcode)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?
            .get("id")
    } else {
        id
    };

    pool.close().await;
    Ok(actual_id)
}

/// Toggles the `is_enabled` flag of a department config.
#[tauri::command]
pub async fn toggle_department_enabled(
    app: tauri::AppHandle,
    id: i64,
    is_enabled: bool,
) -> Result<(), String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    sqlx::query(
        "UPDATE department_config
         SET is_enabled = ?, updated_at = datetime('now')
         WHERE id = ?",
    )
    .bind(i64::from(is_enabled))
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(())
}

/// Removes a department config.
#[tauri::command]
pub async fn remove_department_config(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    sqlx::query("DELETE FROM department_config WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(())
}

/// Returns all department configs.
#[tauri::command]
pub async fn get_all_department_configs(
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let pool = get_pool(&get_setting_db_path(&app)).await?;
    let rows = sqlx::query(
        "SELECT id, depcode, department, is_enabled
         FROM department_config
         ORDER BY department",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;
    pool.close().await;

    let arr: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id":          r.get::<i64,           _>("id"),
                "depcode":     r.get::<String,        _>("depcode"),
                "department":  r.get::<String,        _>("department"),
                "is_enabled":  r.get::<i64,           _>("is_enabled") == 1,
            })
        })
        .collect();
    Ok(serde_json::Value::Array(arr))
}
