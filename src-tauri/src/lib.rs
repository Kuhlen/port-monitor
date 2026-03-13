mod commands;
mod state;
mod types;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::port::list_ports,
            commands::serial::connect_port,
            commands::serial::disconnect_port,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
