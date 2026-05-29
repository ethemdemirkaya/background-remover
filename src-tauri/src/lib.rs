mod commands;
mod compose;
mod inference;
mod state;
mod error;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_image,
            commands::auto_remove,
            commands::smart_select,
            commands::export_image,
            commands::clear_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
