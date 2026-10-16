//! The commands the window calls. Errors come back as `{ code, detail }`; the window translates
//! the code and shows the detail.

use crate::monitor::{Shared, State};
use crate::settings::Settings;
use serde::Serialize;
#[cfg(windows)]
use std::sync::atomic::Ordering;
use std::sync::Arc;
#[cfg(windows)]
use tauri::Manager;

#[derive(Debug, Serialize)]
pub struct CmdError {
    code: &'static str,
    detail: String,
}

impl CmdError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

type CmdResult<T> = Result<T, CmdError>;

#[tauri::command]
pub fn get_state(shared: tauri::State<'_, Arc<Shared>>) -> State {
    shared.state().clone()
}

#[tauri::command]
pub fn get_settings(shared: tauri::State<'_, Arc<Shared>>) -> Settings {
    shared.settings().settings.clone()
}

#[tauri::command]
pub fn set_settings(shared: tauri::State<'_, Arc<Shared>>, settings: Settings) -> CmdResult<Settings> {
    let mut store = shared.settings();
    store
        .save(settings)
        .map_err(|e| CmdError::new("settings_not_saved", e))?;
    Ok(store.settings.clone())
}

/// Runs a firewall change off the async runtime: COM calls block.
#[cfg(windows)]
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> CmdResult<T> + Send + 'static) -> CmdResult<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| CmdError::new("internal", e.to_string()))?
}

#[cfg(windows)]
mod firewall_ops {
    use super::{CmdError, CmdResult};
    use netswitch_core::windows::firewall;
    use std::path::{Path, PathBuf};

    pub fn backups_dir(documents: Option<PathBuf>) -> CmdResult<PathBuf> {
        let dir = documents
            .ok_or_else(|| CmdError::new("no_documents_folder", "the Documents folder could not be found"))?
            .join("netSwitch")
            .join("Backups");
        std::fs::create_dir_all(&dir).map_err(|e| CmdError::new("backup_failed", e.to_string()))?;
        Ok(dir)
    }

    pub fn backup(dir: &Path) -> CmdResult<PathBuf> {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let file = dir.join(format!("FirewallBackup-{stamp}.wfw"));
        firewall::export_policy(&file).map_err(|e| CmdError::new("backup_failed", e.to_string()))?;
        Ok(file)
    }

    /// The first change of a day saves the firewall first, when the setting is on.
    pub fn daily_backup(dir: &Path) -> CmdResult<()> {
        let today = format!("FirewallBackup-{}", chrono::Local::now().format("%Y%m%d"));
        let exists = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .flatten()
                    .any(|e| e.file_name().to_string_lossy().starts_with(&today))
            })
            .unwrap_or(false);
        if !exists {
            backup(dir)?;
        }
        Ok(())
    }

    pub fn failed(e: impl ToString) -> CmdError {
        CmdError::new("firewall_failed", e.to_string())
    }
}

#[cfg(not(windows))]
fn unsupported<T>() -> CmdResult<T> {
    Err(CmdError::new(
        "unsupported",
        "netSwitch changes the firewall on Windows only",
    ))
}

/// Blocks or unblocks one program, inbound and outbound.
#[tauri::command]
pub async fn set_app_blocked(
    app: tauri::AppHandle,
    shared: tauri::State<'_, Arc<Shared>>,
    path: String,
    name: String,
    blocked: bool,
) -> CmdResult<()> {
    let shared = shared.inner().clone();
    #[cfg(windows)]
    {
        let documents = app.path().document_dir().ok();
        let daily = shared.settings().settings.daily_backup;
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        if blocked && netswitch_core::rules::is_protected(&path, &system_root) {
            return Err(CmdError::new("protected_program", name));
        }
        blocking(move || {
            use netswitch_core::windows::firewall;
            if daily {
                firewall_ops::daily_backup(&firewall_ops::backups_dir(documents)?)?;
            }
            if blocked {
                firewall::block(&path, &name).map_err(firewall_ops::failed)?;
            } else {
                firewall::unblock_program(&path).map_err(firewall_ops::failed)?;
            }
            shared.refresh_blocklist.store(true, Ordering::SeqCst);
            Ok(())
        })
        .await
    }
    #[cfg(not(windows))]
    {
        let _ = (app, shared, path, name, blocked);
        unsupported()
    }
}

/// Removes the rules of one application on the Blocked page, from netSwitch or AppIntBlocker.
#[tauri::command]
pub async fn unblock_application(
    app: tauri::AppHandle,
    shared: tauri::State<'_, Arc<Shared>>,
    name: String,
) -> CmdResult<u32> {
    let shared = shared.inner().clone();
    #[cfg(windows)]
    {
        let documents = app.path().document_dir().ok();
        let daily = shared.settings().settings.daily_backup;
        blocking(move || {
            if daily {
                firewall_ops::daily_backup(&firewall_ops::backups_dir(documents)?)?;
            }
            let removed =
                netswitch_core::windows::firewall::unblock_application(&name).map_err(firewall_ops::failed)?;
            shared.refresh_blocklist.store(true, Ordering::SeqCst);
            Ok(removed)
        })
        .await
    }
    #[cfg(not(windows))]
    {
        let _ = (app, shared, name);
        unsupported()
    }
}

/// Removes every AppIntBlocker-named rule.
#[tauri::command]
pub async fn unblock_all(app: tauri::AppHandle, shared: tauri::State<'_, Arc<Shared>>) -> CmdResult<u32> {
    let shared = shared.inner().clone();
    #[cfg(windows)]
    {
        let documents = app.path().document_dir().ok();
        let daily = shared.settings().settings.daily_backup;
        blocking(move || {
            if daily {
                firewall_ops::daily_backup(&firewall_ops::backups_dir(documents)?)?;
            }
            let removed = netswitch_core::windows::firewall::unblock_all().map_err(firewall_ops::failed)?;
            shared.refresh_blocklist.store(true, Ordering::SeqCst);
            Ok(removed)
        })
        .await
    }
    #[cfg(not(windows))]
    {
        let _ = (app, shared);
        unsupported()
    }
}

/// Saves the whole firewall policy and returns the file's path.
#[tauri::command]
pub async fn backup_firewall(app: tauri::AppHandle) -> CmdResult<String> {
    #[cfg(windows)]
    {
        let documents = app.path().document_dir().ok();
        blocking(move || {
            let file = firewall_ops::backup(&firewall_ops::backups_dir(documents)?)?;
            Ok(file.to_string_lossy().into_owned())
        })
        .await
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        unsupported()
    }
}

/// Shows a program in File Explorer.
#[tauri::command]
pub fn reveal_program(path: String) -> CmdResult<()> {
    if !std::path::Path::new(&path).is_absolute() {
        return Err(CmdError::new("not_found", path));
    }
    tauri_plugin_opener::reveal_item_in_dir(&path).map_err(|e| CmdError::new("not_found", e.to_string()))
}

/// Opens Windows Defender Firewall with Advanced Security.
#[tauri::command]
pub fn open_firewall_console() -> CmdResult<()> {
    #[cfg(windows)]
    {
        std::process::Command::new("mmc.exe")
            .arg("wf.msc")
            .spawn()
            .map(|_| ())
            .map_err(|e| CmdError::new("not_found", e.to_string()))
    }
    #[cfg(not(windows))]
    {
        unsupported()
    }
}
