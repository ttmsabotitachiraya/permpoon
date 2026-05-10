//! Export and import of `setting.db` configuration sections.
//!
//! The frontend receives/sends a JSON string and handles the actual file I/O
//! (download / `<input type="file">`), so no filesystem plugin is required.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::Manager;

use super::config::get_pool;

// ── Export data model ─────────────────────────────────────────────────────────

/// A single exported pttype group row.
#[derive(Debug, Serialize, Deserialize)]
pub struct PttypeGroupExport {
    /// Short alias used inside the application.
    pub alias: String,
    /// HOSxP `hipdata_code` value.
    pub hipdata_code: String,
}

/// A single exported icode config row.
#[derive(Debug, Serialize, Deserialize)]
pub struct IcodeConfigExport {
    /// HOSxP item code.
    pub icode: String,
    /// Human-readable service name.
    pub service_name: String,
    /// Whether this rule is currently active.
    pub is_enabled: bool,
    /// Minimum patient age or `None`.
    pub age_min: Option<i64>,
    /// Maximum patient age or `None`.
    pub age_max: Option<i64>,
    /// Gender restriction or `None`.
    pub gender_restrict: Option<String>,
    /// Frequency rule type or `None`.
    pub freq_type: Option<String>,
    /// Frequency threshold or `None`.
    pub freq_value: Option<i64>,
    /// Department / clinic name or `None`.
    pub department: Option<String>,
    /// Aliases of linked pttype groups (portable across databases).
    pub pttype_group_aliases: Vec<String>,
}

/// Top-level export document.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    /// Schema version (currently `1`).
    pub version: u32,
    /// RFC-3339 timestamp of export.
    pub exported_at: String,
    /// Exported pttype groups (may be empty if not requested).
    pub pttype_groups: Vec<PttypeGroupExport>,
    /// Exported icode configs (may be empty if not requested).
    pub icode_configs: Vec<IcodeConfigExport>,
}

// ── Command input/output ──────────────────────────────────────────────────────

/// Controls which sections are included in an export.
#[derive(Debug, Deserialize)]
pub struct ExportOptions {
    /// `true` to include the pttype_groups section.
    pub include_pttype_groups: bool,
    /// `None` = export all icode configs.
    /// `Some([])` = export no icode configs.
    /// `Some([id, …])` = export only the listed rows.
    pub icode_ids: Option<Vec<i64>>,
}

/// Summary returned after a successful import.
#[derive(Debug, Serialize)]
pub struct ImportSummary {
    /// Number of pttype groups that were newly inserted.
    pub pttype_groups_added: usize,
    /// Number of pttype groups that already existed (skipped).
    pub pttype_groups_skipped: usize,
    /// Number of icode configs that were newly inserted.
    pub icode_configs_added: usize,
    /// Number of icode configs that were updated (already existed).
    pub icode_configs_updated: usize,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn setting_db_path(app: &tauri::AppHandle) -> String {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data directory");
    dir.join("setting.db").to_string_lossy().into_owned()
}

fn aliases_from_str(raw: Option<String>) -> Vec<String> {
    raw.unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Exports selected sections from `setting.db` and returns a JSON string.
///
/// The caller (frontend) is responsible for writing the string to a file.
#[tauri::command]
pub async fn export_settings(
    app: tauri::AppHandle,
    options: ExportOptions,
) -> Result<String, String> {
    let pool = get_pool(&setting_db_path(&app)).await?;

    // ── pttype groups ─────────────────────────────────────────────────────────
    let pttype_groups: Vec<PttypeGroupExport> = if options.include_pttype_groups {
        sqlx::query("SELECT alias, hipdata_code FROM pttype_group ORDER BY alias")
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?
            .iter()
            .map(|r| PttypeGroupExport {
                alias: r.get("alias"),
                hipdata_code: r.get("hipdata_code"),
            })
            .collect()
    } else {
        vec![]
    };

    // ── icode configs ─────────────────────────────────────────────────────────
    const ICODE_SELECT: &str = "SELECT c.icode, c.service_name, c.is_enabled, c.age_min, c.age_max,
                c.gender_restrict, c.freq_type, c.freq_value, c.department,
                GROUP_CONCAT(g.alias) AS group_aliases
         FROM icode_config c
         LEFT JOIN icode_pttype_map m ON m.icode_config_id = c.id
         LEFT JOIN pttype_group     g ON g.id = m.pttype_group_id";

    let icode_configs: Vec<IcodeConfigExport> = match &options.icode_ids {
        // Export nothing
        Some(ids) if ids.is_empty() => vec![],

        // Export all
        None => {
            let q = format!("{ICODE_SELECT} GROUP BY c.id ORDER BY c.icode");
            sqlx::query(&q)
                .fetch_all(&pool)
                .await
                .map_err(|e| e.to_string())?
                .iter()
                .map(row_to_icode_export)
                .collect()
        }

        // Export specific rows
        Some(ids) => {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            let q = format!(
                "{ICODE_SELECT} WHERE c.id IN ({placeholders}) GROUP BY c.id ORDER BY c.icode"
            );
            let mut query = sqlx::query(&q);
            for id in ids {
                query = query.bind(*id);
            }
            query
                .fetch_all(&pool)
                .await
                .map_err(|e| e.to_string())?
                .iter()
                .map(row_to_icode_export)
                .collect()
        }
    };

    pool.close().await;

    let data = ExportData {
        version: 1,
        exported_at: Utc::now().to_rfc3339(),
        pttype_groups,
        icode_configs,
    };
    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

/// Imports settings from a JSON string produced by [`export_settings`].
///
/// Pttype groups are inserted when the alias does not already exist.
/// Icode configs are upserted (INSERT OR REPLACE) by `icode`.
/// Pttype-group aliases in the import file are resolved to IDs in the
/// current database; unresolved aliases are silently skipped.
#[tauri::command]
pub async fn import_settings(app: tauri::AppHandle, data: String) -> Result<ImportSummary, String> {
    let export: ExportData = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let pool = get_pool(&setting_db_path(&app)).await?;

    let mut summary = ImportSummary {
        pttype_groups_added: 0,
        pttype_groups_skipped: 0,
        icode_configs_added: 0,
        icode_configs_updated: 0,
    };

    // ── Import pttype groups ──────────────────────────────────────────────────
    for group in &export.pttype_groups {
        let result =
            sqlx::query("INSERT OR IGNORE INTO pttype_group (alias, hipdata_code) VALUES (?, ?)")
                .bind(&group.alias)
                .bind(&group.hipdata_code)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;

        if result.rows_affected() > 0 {
            summary.pttype_groups_added += 1;
        } else {
            summary.pttype_groups_skipped += 1;
        }
    }

    // Build alias → id map after inserting groups
    let pg_rows = sqlx::query("SELECT id, alias FROM pttype_group")
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let alias_to_id: std::collections::HashMap<String, i64> = pg_rows
        .iter()
        .map(|r| (r.get::<String, _>("alias"), r.get::<i64, _>("id")))
        .collect();

    // ── Import icode configs ──────────────────────────────────────────────────
    for cfg in &export.icode_configs {
        // Check if record already exists
        let exists: bool = sqlx::query("SELECT 1 FROM icode_config WHERE icode = ?")
            .bind(&cfg.icode)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?
            .is_some();

        let is_enabled: i64 = i64::from(cfg.is_enabled);
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
        .bind(&cfg.icode)
        .bind(&cfg.service_name)
        .bind(is_enabled)
        .bind(cfg.age_min)
        .bind(cfg.age_max)
        .bind(&cfg.gender_restrict)
        .bind(&cfg.freq_type)
        .bind(cfg.freq_value)
        .bind(&cfg.department)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        // Resolve actual row id
        let actual_id: i64 = if result.last_insert_rowid() == 0 {
            sqlx::query("SELECT id FROM icode_config WHERE icode = ?")
                .bind(&cfg.icode)
                .fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?
                .get("id")
        } else {
            result.last_insert_rowid()
        };

        // Rebuild pttype mappings (only for aliases that exist)
        sqlx::query("DELETE FROM icode_pttype_map WHERE icode_config_id = ?")
            .bind(actual_id)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;

        for alias in &cfg.pttype_group_aliases {
            if let Some(&gid) = alias_to_id.get(alias) {
                sqlx::query(
                    "INSERT OR IGNORE INTO icode_pttype_map
                     (icode_config_id, pttype_group_id) VALUES (?, ?)",
                )
                .bind(actual_id)
                .bind(gid)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        if exists {
            summary.icode_configs_updated += 1;
        } else {
            summary.icode_configs_added += 1;
        }
    }

    pool.close().await;
    Ok(summary)
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn row_to_icode_export(r: &sqlx::sqlite::SqliteRow) -> IcodeConfigExport {
    IcodeConfigExport {
        icode: r.get("icode"),
        service_name: r.get("service_name"),
        is_enabled: r.get::<i64, _>("is_enabled") == 1,
        age_min: r.get("age_min"),
        age_max: r.get("age_max"),
        gender_restrict: r.get("gender_restrict"),
        freq_type: r.get("freq_type"),
        freq_value: r.get("freq_value"),
        department: r.get("department"),
        pttype_group_aliases: aliases_from_str(r.get("group_aliases")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_data_roundtrip() {
        let data = ExportData {
            version: 1,
            exported_at: "2024-01-01T00:00:00Z".to_string(),
            pttype_groups: vec![PttypeGroupExport {
                alias: "UCS".to_string(),
                hipdata_code: "UCS".to_string(),
            }],
            icode_configs: vec![IcodeConfigExport {
                icode: "1234567".to_string(),
                service_name: "Test".to_string(),
                is_enabled: true,
                age_min: None,
                age_max: None,
                gender_restrict: None,
                freq_type: None,
                freq_value: None,
                department: None,
                pttype_group_aliases: vec!["UCS".to_string()],
            }],
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: ExportData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.pttype_groups.len(), 1);
        assert_eq!(parsed.icode_configs[0].icode, "1234567");
    }

    #[test]
    fn import_summary_fields() {
        let s = ImportSummary {
            pttype_groups_added: 2,
            pttype_groups_skipped: 1,
            icode_configs_added: 3,
            icode_configs_updated: 0,
        };
        assert_eq!(s.pttype_groups_added, 2);
    }

    #[test]
    fn aliases_from_str_empty() {
        assert!(aliases_from_str(None).is_empty());
        assert!(aliases_from_str(Some(String::new())).is_empty());
    }

    #[test]
    fn aliases_from_str_multiple() {
        let v = aliases_from_str(Some("UCS,OFC,WEL".to_string()));
        assert_eq!(v, ["UCS", "OFC", "WEL"]);
    }
}
