use base64::Engine;
use serde::Deserialize;
use std::fs;
use tauri::Manager;

#[derive(Debug, Deserialize)]
pub struct TempPdfOpenInput {
    pub file_name: String,
    pub pdf_base64: String,
}

fn sanitize_file_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();

    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() {
        "recommendation-slip.pdf".to_string()
    } else if trimmed.to_ascii_lowercase().ends_with(".pdf") {
        trimmed.to_string()
    } else {
        format!("{trimmed}.pdf")
    }
}

fn cleanup_old_temp_pdfs(dir: &std::path::Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let now = std::time::SystemTime::now();
    let max_age = std::time::Duration::from_secs(60 * 60 * 24);

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with("recommendation-slip-") || !name.ends_with(".pdf") {
            continue;
        }

        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let Ok(age) = now.duration_since(modified) else {
            continue;
        };
        if age > max_age {
            let _ = fs::remove_file(path);
        }
    }
}

#[tauri::command]
pub async fn open_temp_pdf_with_viewer(
    app: tauri::AppHandle,
    input: TempPdfOpenInput,
) -> Result<String, String> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("print-preview");

    fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    cleanup_old_temp_pdfs(&cache_dir);

    let file_name = sanitize_file_name(&input.file_name);
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S-%3f");
    let output_path = cache_dir.join(format!("recommendation-slip-{timestamp}-{file_name}"));

    let pdf_bytes = base64::engine::general_purpose::STANDARD
        .decode(input.pdf_base64.as_bytes())
        .map_err(|e| e.to_string())?;

    fs::write(&output_path, pdf_bytes).map_err(|e| e.to_string())?;

    let output_str = output_path.to_string_lossy().into_owned();
    tauri_plugin_opener::open_path(&output_str, None::<&str>).map_err(|e| e.to_string())?;

    Ok(output_str)
}
