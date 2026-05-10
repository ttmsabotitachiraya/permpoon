//! Commands for connecting to the HOSxP MySQL database and querying data.

use crate::commands::db_config::DbConfig;
use crate::models::pttype::PttypeRow;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::Row;

/// Opens a MySQL connection pool using the provided [`DbConfig`].
async fn mysql_pool(config: &DbConfig) -> Result<sqlx::MySqlPool, String> {
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.username, config.password, config.host, config.port, config.database
    );
    MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|e| e.to_string())
}

/// Tests the MySQL connection by executing `SELECT 1`.
#[tauri::command]
pub async fn test_db_connection(config: DbConfig) -> Result<bool, String> {
    let pool = mysql_pool(&config).await?;
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(true)
}

/// Searches for pttype rows that match the given `hipdata_code`.
#[tauri::command]
pub async fn search_pttype_by_hipdata(
    config: DbConfig,
    hipdata_code: String,
) -> Result<Vec<PttypeRow>, String> {
    let pool = MySqlPoolOptions::new()
        .max_connections(2)
        .connect(&format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        ))
        .await
        .map_err(|e| e.to_string())?;
    let rows = sqlx::query("SELECT pttype, name FROM pttype WHERE hipdata_code = ?")
        .bind(&hipdata_code)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(rows
        .iter()
        .map(|r| PttypeRow {
            pttype: r.get::<String, _>("pttype"),
            name: r.get::<String, _>("name"),
        })
        .collect())
}

/// Returns the name of an item from `drugitems` or `nondrugitems`.
#[tauri::command]
pub async fn get_icode_name(config: DbConfig, icode: String) -> Result<Option<String>, String> {
    let pool = MySqlPoolOptions::new()
        .max_connections(2)
        .connect(&format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        ))
        .await
        .map_err(|e| e.to_string())?;
    let row = sqlx::query(
        "SELECT name FROM drugitems    WHERE icode = ?
         UNION ALL
         SELECT name FROM nondrugitems WHERE icode = ?
         LIMIT 1",
    )
    .bind(&icode)
    .bind(&icode)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(row.map(|r| r.get::<String, _>("name")))
}

/// Searches kskdepartment table by depcode
#[tauri::command]
pub async fn search_department_by_depcode(
    config: DbConfig,
    depcode: String,
) -> Result<Option<serde_json::Value>, String> {
    let pool = MySqlPoolOptions::new()
        .max_connections(2)
        .connect(&format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        ))
        .await
        .map_err(|e| e.to_string())?;
    let row =
        sqlx::query("SELECT depcode, department FROM kskdepartment WHERE depcode = ? LIMIT 1")
            .bind(&depcode)
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(row.map(|r| {
        serde_json::json!({
            "depcode": r.get::<String, _>("depcode"),
            "department": r.get::<String, _>("department"),
        })
    }))
}
