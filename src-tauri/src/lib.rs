mod commands;
mod context;
mod errors;
mod logger;
mod network;
mod runner;
mod settings;
mod system;
mod wrapper;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::block_application,
            commands::get_firewall_rules,
            commands::remove_firewall_rules,
            commands::open_windows_firewall,
            commands::create_restore_point,
            commands::get_firewall_stats,
            settings::load_settings,
            settings::save_settings,
            settings::reset_settings,
            settings::get_app_info,
            settings::open_logs_folder,
            settings::export_settings,
            settings::import_settings,
            network::get_network_processes,
            network::get_network_interfaces,
            system::get_system_stats,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
