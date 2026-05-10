mod commands;
mod crypto;
mod models;

use commands::config::init_sqlite;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                init_sqlite(&handle).await.expect("SQLite init failed");
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // DB connection config (db.json)
            commands::db_config::save_db_config,
            commands::db_config::load_db_config,
            // MySQL queries
            commands::db::test_db_connection,
            commands::db::search_pttype_by_hipdata,
            commands::db::get_icode_name,
            // App config (setting.db – generic key-value)
            commands::config::save_config,
            commands::config::get_config,
            commands::config::get_all_config,
            // Pttype groups
            commands::config::add_pttype_group,
            commands::config::remove_pttype_group,
            commands::config::get_all_pttype_groups,
            // Icode configs
            commands::config::save_icode_config,
            commands::config::toggle_icode_enabled,
            commands::config::remove_icode_config,
            commands::config::get_all_icode_configs,
            // Department configs
            commands::config::save_department_config,
            commands::config::get_all_department_configs,
            commands::config::toggle_department_enabled,
            commands::config::remove_department_config,
            // Patient / recommendations
            commands::patient::lookup_patient,
            commands::patient::get_service_history,
            commands::patient::get_recommendations,
            commands::db::search_department_by_depcode,
            commands::patient::search_patients_by_department,
            commands::patient::search_patients_with_recommendations,
            // Export / import
            commands::settings_io::export_settings,
            commands::settings_io::import_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
