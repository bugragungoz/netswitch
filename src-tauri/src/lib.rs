//! The netSwitch window: a monitor thread that measures once a second, and the commands the
//! window calls. All counting lives in `netswitch-core`; this crate only wires it to Tauri.

mod commands;
mod monitor;
mod settings;

use std::sync::Arc;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second netSwitch would fight the first over the traffic trace; show the first instead.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir().ok();
            let settings = settings::Store::open(config_dir);
            let shared = Arc::new(monitor::Shared::new(settings));
            monitor::start(app.handle().clone(), shared.clone());
            app.manage(shared);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::get_settings,
            commands::set_settings,
            commands::set_app_blocked,
            commands::unblock_application,
            commands::unblock_all,
            commands::backup_firewall,
            commands::reveal_program,
            commands::open_firewall_console,
        ])
        .run(tauri::generate_context!())
        .expect("netSwitch could not start");
}
