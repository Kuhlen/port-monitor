pub mod commands;
pub mod infra;
pub mod state;

use tauri::Manager;

use state::AppState;

// ONE command list for runtime and test. Never twin it: a twin list lets an
// unregistered command pass the test.
#[macro_export]
macro_rules! handler {
    () => {
        tauri::generate_handler![
            $crate::commands::list_ports,
            $crate::commands::connect_port,
            $crate::commands::disconnect_port,
            $crate::commands::check_update,
            $crate::commands::install_update,
        ]
    };
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .setup(|app| {
            app.manage(AppState::new(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(crate::handler!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
