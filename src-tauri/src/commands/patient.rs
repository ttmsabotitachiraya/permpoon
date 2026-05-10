use crate::commands::db_config::DbConfig;
use crate::models::patient::{PatientInfo, PatientQuery};
use crate::models::service_item::{RecommendationItem, ServiceHistoryRow};
use chrono::{Datelike, Duration, NaiveDate};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Row;
use std::str::FromStr;
use tauri::Manager;

async fn mysql_pool(config: &DbConfig) -> Result<sqlx::MySqlPool, String> {
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.username, config.password, config.host, config.port, config.database
    );
    MySqlPoolOptions::new()
        .max_connections(3)
        .connect(&url)
        .await
        .map_err(|e| e.to_string())
}

fn calculate_age(dob: &str, on_date: &str) -> i32 {
    let birth = NaiveDate::parse_from_str(dob, "%Y-%m-%d").unwrap_or_default();
    let target = NaiveDate::parse_from_str(on_date, "%Y-%m-%d").unwrap_or_default();
    let mut age = target.year() - birth.year();
    if (target.month(), target.day()) < (birth.month(), birth.day()) {
        age -= 1;
    }
    age
}

fn row_to_patient(r: &sqlx::mysql::MySqlRow, process_date: &str) -> PatientInfo {
    let dob: String = r.try_get::<String, _>("dob").unwrap_or_default();
    let age = calculate_age(&dob, process_date);
    let sex_raw: String = r.try_get::<String, _>("sex").unwrap_or_default();
    let sex = match sex_raw.as_str() {
        "1" | "M" | "m" => "M".to_string(),
        "2" | "F" | "f" => "F".to_string(),
        other => other.to_string(),
    };
    PatientInfo {
        hn: r.try_get("hn").unwrap_or_default(),
        fname: r.try_get("fname").unwrap_or_default(),
        lname: r.try_get("lname").unwrap_or_default(),
        cid: r.try_get("cid").unwrap_or_default(),
        pttype: r.try_get("pttype").unwrap_or_default(),
        pttype_name: r.try_get("pttype_name").unwrap_or_default(),
        hipdata_code: r.try_get("hipdata_code").unwrap_or_default(),
        dob,
        sex,
        age,
    }
}

#[tauri::command]
pub async fn lookup_patient(
    config: DbConfig,
    query: PatientQuery,
    process_date: String,
) -> Result<Vec<PatientInfo>, String> {
    let pool = mysql_pool(&config).await?;
    let base_select = "SELECT p.hn, p.fname, p.lname, p.cid, p.pttype, p.birthday AS dob, p.sex,
                       pt.name AS pttype_name, pt.hipdata_code
                       FROM patient p
                       INNER JOIN pttype pt ON pt.pttype = p.pttype";
    let rows = if let Some(hn) = &query.hn {
        sqlx::query(&format!("{} WHERE p.hn = ? LIMIT 1", base_select))
            .bind(hn)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?
    } else if let Some(cid) = &query.cid {
        sqlx::query(&format!("{} WHERE p.cid = ? LIMIT 1", base_select))
            .bind(cid)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?
    } else if let (Some(fname), Some(lname)) = (&query.fname, &query.lname) {
        let fname_like = format!("%{}%", fname);
        let lname_like = format!("%{}%", lname);
        sqlx::query(&format!(
            "{} WHERE p.fname LIKE ? AND p.lname LIKE ? LIMIT 50",
            base_select
        ))
        .bind(&fname_like)
        .bind(&lname_like)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else if let Some(fname) = &query.fname {
        let fname_like = format!("%{}%", fname);
        sqlx::query(&format!("{} WHERE p.fname LIKE ? LIMIT 50", base_select))
            .bind(&fname_like)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?
    } else if let Some(lname) = &query.lname {
        let lname_like = format!("%{}%", lname);
        sqlx::query(&format!("{} WHERE p.lname LIKE ? LIMIT 50", base_select))
            .bind(&lname_like)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?
    } else {
        return Err("กรุณาระบุ HN, CID หรือ ชื่อ-นามสกุล".to_string());
    };
    pool.close().await;
    Ok(rows
        .iter()
        .map(|r| row_to_patient(r, &process_date))
        .collect())
}

#[tauri::command]
pub async fn get_service_history(
    config: DbConfig,
    hn: String,
    icode: String,
) -> Result<Vec<ServiceHistoryRow>, String> {
    let pool = mysql_pool(&config).await?;
    let rows = sqlx::query(
        "SELECT o.icode, DATE_FORMAT(v.vstdate, '%Y-%m-%d') as vstdate FROM opitemrece o JOIN ovst v ON v.vn = o.vn WHERE v.hn = ? AND o.icode = ? ORDER BY v.vstdate DESC",
    )
    .bind(&hn)
    .bind(&icode)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(rows
        .iter()
        .map(|r| ServiceHistoryRow {
            icode: r.try_get("icode").unwrap_or_default(),
            vstdate: r.try_get::<String, _>("vstdate").unwrap_or_default(),
        })
        .collect())
}

fn passes_freq(
    freq_type: &Option<String>,
    freq_value: &Option<i64>,
    history: &[ServiceHistoryRow],
    process_date: &str,
) -> bool {
    match freq_type.as_deref() {
        None | Some("") => true,
        Some("days") => {
            if let Some(last) = history.first() {
                let last_date =
                    NaiveDate::parse_from_str(&last.vstdate[..10], "%Y-%m-%d").unwrap_or_default();
                let target =
                    NaiveDate::parse_from_str(process_date, "%Y-%m-%d").unwrap_or_default();
                let diff = (target - last_date).num_days();
                diff >= freq_value.unwrap_or(0)
            } else {
                true
            }
        }
        Some("months") => {
            if let Some(last) = history.first() {
                let last_date =
                    NaiveDate::parse_from_str(&last.vstdate[..10], "%Y-%m-%d").unwrap_or_default();
                let target =
                    NaiveDate::parse_from_str(process_date, "%Y-%m-%d").unwrap_or_default();
                let months = (target.year() - last_date.year()) * 12 + target.month() as i32
                    - last_date.month() as i32;
                months >= freq_value.unwrap_or(0) as i32
            } else {
                true
            }
        }
        Some("years") => {
            if let Some(last) = history.first() {
                let last_date =
                    NaiveDate::parse_from_str(&last.vstdate[..10], "%Y-%m-%d").unwrap_or_default();
                let target =
                    NaiveDate::parse_from_str(process_date, "%Y-%m-%d").unwrap_or_default();
                let years = target.year() - last_date.year();
                years >= freq_value.unwrap_or(0) as i32
            } else {
                true
            }
        }
        Some("per_week") => {
            let target = NaiveDate::parse_from_str(process_date, "%Y-%m-%d").unwrap_or_default();
            let days_from_monday = target.weekday().num_days_from_monday() as i64;
            let monday = target - Duration::days(days_from_monday);
            let sunday = monday + Duration::days(6);
            let count = history
                .iter()
                .filter(|h| {
                    let d =
                        NaiveDate::parse_from_str(&h.vstdate[..10], "%Y-%m-%d").unwrap_or_default();
                    d >= monday && d <= sunday
                })
                .count() as i64;
            count < freq_value.unwrap_or(0)
        }
        Some("per_year") => {
            let target = NaiveDate::parse_from_str(process_date, "%Y-%m-%d").unwrap_or_default();
            let fy_start_year = if target.month() >= 10 {
                target.year()
            } else {
                target.year() - 1
            };
            let fiscal_start = NaiveDate::from_ymd_opt(fy_start_year, 10, 1).unwrap_or_default();
            let fiscal_end = NaiveDate::from_ymd_opt(fy_start_year + 1, 9, 30).unwrap_or_default();
            let count = history
                .iter()
                .filter(|h| {
                    let d =
                        NaiveDate::parse_from_str(&h.vstdate[..10], "%Y-%m-%d").unwrap_or_default();
                    d >= fiscal_start && d <= fiscal_end
                })
                .count() as i64;
            count < freq_value.unwrap_or(0)
        }
        Some("total_limit") => (history.len() as i64) < freq_value.unwrap_or(0),
        _ => true,
    }
}

#[tauri::command]
pub async fn get_recommendations(
    app_handle: tauri::AppHandle,
    config: DbConfig,
    hn: String,
    process_date: String,
) -> Result<Vec<RecommendationItem>, String> {
    // 1. Get patient
    let patient_list = lookup_patient(
        config.clone(),
        PatientQuery {
            hn: Some(hn.clone()),
            cid: None,
            fname: None,
            lname: None,
        },
        process_date.clone(),
    )
    .await?;
    let patient = patient_list
        .into_iter()
        .next()
        .ok_or("ไม่พบข้อมูลผู้ป่วย".to_string())?;

    // 2. Get already-received today from HOSxP (read-only)
    let mysql_pool_ref = mysql_pool(&config).await?;
    let today_rows = sqlx::query(
        "SELECT o.icode FROM opitemrece o JOIN ovst v ON v.vn = o.vn WHERE v.hn = ? AND v.vstdate = ?",
    )
    .bind(&hn)
    .bind(&process_date)
    .fetch_all(&mysql_pool_ref)
    .await
    .map_err(|e| e.to_string())?;
    mysql_pool_ref.close().await;
    let today_icodes: std::collections::HashSet<String> = today_rows
        .iter()
        .map(|r| r.get::<String, _>("icode"))
        .collect();

    // 3. Load icode_config from SQLite
    let data_dir = app_handle.path().app_data_dir().expect("app data dir");
    let db_path = format!("{}/setting.db", data_dir.to_string_lossy());
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", db_path))
        .map_err(|e| e.to_string())?;
    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .map_err(|e| e.to_string())?;

    let icode_rows = sqlx::query(
        "SELECT c.id, c.icode, c.service_name, c.department, c.is_enabled, c.age_min, c.age_max,
                c.gender_restrict, c.freq_type, c.freq_value,
                GROUP_CONCAT(m.pttype_group_id) as group_ids
         FROM icode_config c
         LEFT JOIN icode_pttype_map m ON m.icode_config_id = c.id
         WHERE c.is_enabled = 1
         GROUP BY c.id",
    )
    .fetch_all(&sqlite_pool)
    .await
    .map_err(|e| e.to_string())?;

    // get pttype_groups
    let pg_rows = sqlx::query("SELECT id, alias, hipdata_code FROM pttype_group")
        .fetch_all(&sqlite_pool)
        .await
        .map_err(|e| e.to_string())?;
    sqlite_pool.close().await;

    let pttype_groups: Vec<(i64, String, String)> = pg_rows
        .iter()
        .map(|r| {
            (
                r.get::<i64, _>("id"),
                r.get::<String, _>("alias"),
                r.get::<String, _>("hipdata_code"),
            )
        })
        .collect();

    // find matching pttype_group for this patient
    let patient_groups: Vec<&(i64, String, String)> = pttype_groups
        .iter()
        .filter(|(_, _, hc)| hc == &patient.hipdata_code)
        .collect();

    if patient_groups.is_empty() {
        return Ok(vec![]);
    }

    let mut results = vec![];

    for row in &icode_rows {
        let icode: String = row.get("icode");
        if today_icodes.contains(&icode) {
            continue;
        }

        let group_ids_str: Option<String> = row.get("group_ids");
        let group_ids: Vec<i64> = group_ids_str
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.parse::<i64>().ok())
            .collect();

        // check pttype match
        let matched_aliases: Vec<String> = patient_groups
            .iter()
            .filter(|(gid, _, _)| group_ids.contains(gid))
            .map(|(_, alias, _)| alias.clone())
            .collect();
        if matched_aliases.is_empty() {
            continue;
        }

        // check age
        let age_min: Option<i64> = row.get("age_min");
        let age_max: Option<i64> = row.get("age_max");
        if let Some(min) = age_min {
            if (patient.age as i64) < min {
                continue;
            }
        }
        if let Some(max) = age_max {
            if (patient.age as i64) > max {
                continue;
            }
        }

        // check gender
        let gender_restrict: Option<String> = row.get("gender_restrict");
        if let Some(g) = &gender_restrict {
            if !g.is_empty() && g != &patient.sex {
                continue;
            }
        }

        // check freq
        let freq_type: Option<String> = row.get("freq_type");
        let freq_value: Option<i64> = row.get("freq_value");
        if freq_type.is_some() {
            let history = get_service_history(config.clone(), hn.clone(), icode.clone()).await?;
            if !passes_freq(&freq_type, &freq_value, &history, &process_date) {
                continue;
            }
        }

        results.push(RecommendationItem {
            icode: icode.clone(),
            service_name: row.get("service_name"),
            department: row.get("department"),
            pttype_alias: matched_aliases,
        });
    }

    Ok(results)
}

/// Searches patients who visited a specific department on a given date
#[tauri::command]
pub async fn search_patients_by_department(
    config: DbConfig,
    depcodes: Vec<String>,
    process_date: String,
) -> Result<Vec<PatientInfo>, String> {
    let pool = mysql_pool(&config).await?;

    let rows = if depcodes.is_empty() {
        sqlx::query(
            "SELECT DISTINCT p.hn, p.fname, p.lname, p.cid, p.pttype, p.birthday AS dob, p.sex,
                    pt.name AS pttype_name, pt.hipdata_code
             FROM ovst v
             JOIN patient p ON p.hn = v.hn
             JOIN pttype pt ON pt.pttype = p.pttype
             WHERE DATE(v.vstdate) = DATE(?)
             ORDER BY p.fname, p.lname
             LIMIT 500",
        )
        .bind(&process_date)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        // สร้าง placeholders สำหรับ IN clause
        let placeholders = depcodes.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT DISTINCT p.hn, p.fname, p.lname, p.cid, p.pttype, p.birthday AS dob, p.sex,
                    pt.name AS pttype_name, pt.hipdata_code
             FROM ovst v
             JOIN patient p ON p.hn = v.hn
             JOIN pttype pt ON pt.pttype = p.pttype
             WHERE DATE(v.vstdate) = DATE(?) AND v.depcode IN ({})
             ORDER BY p.fname, p.lname
             LIMIT 500",
            placeholders
        );
        let mut q = sqlx::query(&sql).bind(&process_date);
        for code in &depcodes {
            q = q.bind(code);
        }
        q.fetch_all(&pool).await.map_err(|e| e.to_string())?
    };

    pool.close().await;
    Ok(rows
        .iter()
        .map(|r| row_to_patient(r, &process_date))
        .collect())
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PatientWithRecs {
    pub hn: String,
    pub fname: String,
    pub lname: String,
    pub cid: String,
    pub pttype: String,
    pub pttype_name: String,
    pub hipdata_code: String,
    pub dob: String,
    pub sex: String,
    pub age: i32,
    pub recommendations: Vec<RecommendationItem>,
}

/// Searches patients with their recommendations in a single optimized query
#[tauri::command]
pub async fn search_patients_with_recommendations(
    app_handle: tauri::AppHandle,
    config: DbConfig,
    depcodes: Vec<String>,
    process_date: String,
) -> Result<Vec<PatientWithRecs>, String> {
    let mysql_pool_ref = mysql_pool(&config).await?;

    // 1. Get patients first
    let patient_rows = if depcodes.is_empty() {
        sqlx::query(
            "SELECT p.hn, p.fname, p.lname, p.cid, p.pttype, p.birthday AS dob, p.sex,
                    pt.name AS pttype_name, pt.hipdata_code
             FROM ovst v
             JOIN patient p ON p.hn = v.hn
             JOIN pttype pt ON pt.pttype = p.pttype
             WHERE DATE(v.vstdate) = DATE(?)
             ORDER BY p.fname, p.lname
             LIMIT 500",
        )
        .bind(&process_date)
        .fetch_all(&mysql_pool_ref)
        .await
        .map_err(|e| e.to_string())?
    } else {
        let placeholders = depcodes.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT p.hn, p.fname, p.lname, p.cid, p.pttype, p.birthday AS dob, p.sex,
                    pt.name AS pttype_name, pt.hipdata_code
             FROM ovst v
             JOIN patient p ON p.hn = v.hn
             JOIN pttype pt ON pt.pttype = p.pttype
             WHERE DATE(v.vstdate) = DATE(?) AND v.depcode IN ({})
             ORDER BY p.fname, p.lname
             LIMIT 500",
            placeholders
        );
        let mut q = sqlx::query(&sql).bind(&process_date);
        for code in &depcodes {
            q = q.bind(code);
        }
        q.fetch_all(&mysql_pool_ref)
            .await
            .map_err(|e| e.to_string())?
    };

    if patient_rows.is_empty() {
        mysql_pool_ref.close().await;
        return Ok(vec![]);
    }

    // 2. Get all HNs and their today icodes in one query
    let hns: Vec<String> = patient_rows.iter().map(|r| r.get::<String, _>("hn")).collect();
    let hn_list = hns.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    let today_icodes_sql = format!(
        "SELECT v.hn, o.icode FROM opitemrece o JOIN ovst v ON v.vn = o.vn WHERE v.hn IN ({}) AND DATE(v.vstdate) = DATE(?)",
        hn_list
    );
    let mut today_q = sqlx::query(&today_icodes_sql);
    for hn in &hns {
        today_q = today_q.bind(hn);
    }
    today_q = today_q.bind(&process_date);
    let today_rows = today_q
        .fetch_all(&mysql_pool_ref)
        .await
        .map_err(|e| e.to_string())?;

    // Group today's icodes by hn
    let mut today_icodes_map: std::collections::HashMap<String, std::collections::HashSet<String>> = std::collections::HashMap::new();
    for row in &today_rows {
        let hn: String = row.get("hn");
        let icode: String = row.get("icode");
        today_icodes_map.entry(hn).or_default().insert(icode);
    }

    mysql_pool_ref.close().await;

    // 3. Load icode_config from SQLite (only enabled ones)
    let data_dir = app_handle.path().app_data_dir().expect("app data dir");
    let db_path = format!("{}/setting.db", data_dir.to_string_lossy());
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", db_path))
        .map_err(|e| e.to_string())?;
    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .map_err(|e| e.to_string())?;

    let icode_rows = sqlx::query(
        "SELECT c.id, c.icode, c.service_name, c.department, c.is_enabled, c.age_min, c.age_max,
                c.gender_restrict, c.freq_type, c.freq_value,
                GROUP_CONCAT(m.pttype_group_id) as group_ids
         FROM icode_config c
         LEFT JOIN icode_pttype_map m ON m.icode_config_id = c.id
         WHERE c.is_enabled = 1
         GROUP BY c.id",
    )
    .fetch_all(&sqlite_pool)
    .await
    .map_err(|e| e.to_string())?;

    let pg_rows = sqlx::query("SELECT id, alias, hipdata_code FROM pttype_group")
        .fetch_all(&sqlite_pool)
        .await
        .map_err(|e| e.to_string())?;

    // Map: hipdata_code -> group id (เช่น "U" -> ["1","2"])
    let mut pttype_group_ids_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    // Map: id -> alias (สำหรับแสดงผล)
    let mut group_id_to_alias: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for row in &pg_rows {
        let id: String = row.get::<i32, _>("id").to_string();
        let alias: String = row.get("alias");
        let hipdata_code: String = row.get("hipdata_code");
        pttype_group_ids_map
            .entry(hipdata_code.clone())
            .or_default()
            .push(id.clone());
        group_id_to_alias.insert(id, alias);
    }

    sqlite_pool.close().await;

    // 4. Build result
    let mut result: Vec<PatientWithRecs> = Vec::new();

    for row in &patient_rows {
        let hn: String = row.get("hn");
        let fname: String = row.get("fname");
        let lname: String = row.get("lname");
        let cid: String = row.get("cid");
        let pttype: String = row.get("pttype");
        let pttype_name: String = row.get("pttype_name");
        let hipdata_code: String = row.get("hipdata_code");
        let dob: String = row.try_get("dob").unwrap_or_default();
        let sex_raw: String = row.try_get("sex").unwrap_or_default();
        let sex = match sex_raw.as_str() {
            "1" | "M" | "m" => "M".to_string(),
            "2" | "F" | "f" => "F".to_string(),
            other => other.to_string(),
        };
        let age = calculate_age(&dob, &process_date);

        let today_icodes = today_icodes_map.get(&hn).cloned().unwrap_or_default();
        let pttype_group_ids = pttype_group_ids_map.get(&hipdata_code).cloned().unwrap_or_default();

        // ถ้าไม่มี group ให้ใช้ pttype โดยตรง (ไม่ filter group)
        let patient_has_group = !pttype_group_ids.is_empty();

        let mut recommendations: Vec<RecommendationItem> = Vec::new();

        for icode_row in &icode_rows {
            let icode: String = icode_row.get("icode");

            // Skip if already received today
            if today_icodes.contains(&icode) {
                continue;
            }

            let service_name: String = icode_row.get("service_name");
            let department: Option<String> = icode_row.get("department");
            let age_min: Option<i32> = icode_row.get("age_min");
            let age_max: Option<i32> = icode_row.get("age_max");
            let gender_restrict: Option<String> = icode_row.get("gender_restrict");
            let freq_type: Option<String> = icode_row.get("freq_type");
            let group_ids_str: Option<String> = icode_row.get("group_ids");

            // Age filter
            if let Some(min) = age_min {
                if age < min {
                    continue;
                }
            }
            if let Some(max) = age_max {
                if age > max {
                    continue;
                }
            }

            // Gender filter
            if let Some(gr) = &gender_restrict {
                if !gr.is_empty() {
                    let gr_upper = gr.to_uppercase();
                    if gr_upper == "M" && sex != "M" {
                        continue;
                    }
                    if gr_upper == "F" && sex != "F" {
                        continue;
                    }
                }
            }

            // Group filter: ถ้า icode มี group และ ผู้ป่วยมี group ให้เช็ค
            if let Some(gids) = &group_ids_str {
                if !gids.is_empty() {
                    // ถ้าผู้ป่วยไม่มี group ที่กำหนด ให้ข้าม (ยกเว้น icode ที่ไม่กำหนด group)
                    if patient_has_group {
                        let has_group = pttype_group_ids.iter().any(|gid| gids.contains(gid));
                        if !has_group {
                            continue;
                        }
                    }
                }
            }

            // ถ้าไม่มี group filter และ patient ไม่มี group ในระบบ ให้แสดง

            // Freq check (simplified - skip for now as it requires historical data)
            if let Some(ft) = &freq_type {
                if ft == "once" || ft == "yearly" || ft == "monthly" {
                    // Would need historical check - skip for speed
                }
            }

            let aliases: Vec<String> = pttype_group_ids
                .iter()
                .filter_map(|gid| group_id_to_alias.get(gid).cloned())
                .collect();

            recommendations.push(RecommendationItem {
                icode,
                service_name,
                department,
                pttype_alias: aliases,
            });
        }

        result.push(PatientWithRecs {
            hn,
            fname,
            lname,
            cid,
            pttype,
            pttype_name,
            hipdata_code,
            dob,
            sex,
            age,
            recommendations,
        });
    }

    Ok(result)
}
