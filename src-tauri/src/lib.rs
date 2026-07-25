//! Tauri command layer — a thin bridge to `palbox-core`.
//!
//! Keep real logic in the core crate; commands here should just marshal arguments and
//! results across the UI boundary. This preserves the core/UI separation.

/// Returns the core engine version. Smoke test that the UI <-> core bridge is wired.
#[tauri::command]
fn core_version() -> String {
    palbox_core::version().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![core_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
