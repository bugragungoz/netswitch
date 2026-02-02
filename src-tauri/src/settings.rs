use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    // General
    pub auto_create_restore_point: bool,
    pub show_confirmation_dialogs: bool,
    pub include_subdirectories: bool,
    pub block_exe_by_default: bool,
    pub block_dll_by_default: bool,

    // Blocking Defaults
    pub default_excluded_keywords: Vec<String>,
    pub default_excluded_files: Vec<String>,
    pub custom_rule_prefix: String,

    // Performance
    pub enable_detailed_logging: bool,
    pub log_retention_days: u32,
    pub cache_firewall_rules: bool,

    // Advanced
    pub debug_mode: bool,
    pub check_for_updates: bool,
    pub run_at_startup: bool,
}

impl AppSettings {
    pub fn default_settings() -> Self {
        Self {
            auto_create_restore_point: false,
            show_confirmation_dialogs: true,
            include_subdirectories: true,
            block_exe_by_default: true,
            block_dll_by_default: true,
            default_excluded_keywords: vec![
                "uninstall".to_string(),
                "updater".to_string(),
                "helper".to_string(),
            ],
            default_excluded_files: vec![],
            custom_rule_prefix: "AppBlocker Rule -".to_string(),
            enable_detailed_logging: false,
            log_retention_days: 30,
            cache_firewall_rules: true,
            debug_mode: false,
            check_for_updates: true,
            run_at_startup: false,
        }
    }
}

fn get_settings_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(app_data)
        .join("AppInternetBlocker")
        .join("settings.json")
}

fn get_logs_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(app_data).join("AppInternetBlocker").join("logs")
}

#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    let path = get_settings_path();

    if !path.exists() {
        return Ok(AppSettings::default_settings());
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;
    let settings: AppSettings =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?;

    Ok(settings)
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path();

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }

    let content =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn reset_settings() -> Result<AppSettings, String> {
    let settings = AppSettings::default_settings();
    save_settings(settings.clone())?;
    Ok(settings)
}

#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub version: String,
    pub build_date: String,
    pub settings_path: String,
    pub logs_path: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_date: "2026-02-02".to_string(),
        settings_path: get_settings_path().to_string_lossy().to_string(),
        logs_path: get_logs_path().to_string_lossy().to_string(),
    }
}

#[tauri::command]
pub fn open_logs_folder() -> Result<(), String> {
    let path = get_logs_path();

    // Create directory if it doesn't exist
    fs::create_dir_all(&path).map_err(|e| format!("Failed to create logs directory: {}", e))?;

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open logs folder: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn export_settings() -> Result<String, String> {
    let settings = load_settings()?;
    serde_json::to_string_pretty(&settings).map_err(|e| format!("Failed to export settings: {}", e))
}

#[tauri::command]
pub fn import_settings(json_content: String) -> Result<AppSettings, String> {
    let settings: AppSettings =
        serde_json::from_str(&json_content).map_err(|e| format!("Failed to parse settings: {}", e))?;
    save_settings(settings.clone())?;
    Ok(settings)
}
