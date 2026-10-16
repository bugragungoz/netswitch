//! The user's settings, kept as JSON in the app's configuration folder.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// "system", "dark" or "light".
    pub theme: String,
    /// "system", "en" or "tr".
    pub language: String,
    /// Show a Windows notification when an app reaches the internet for the first time in a run.
    pub notify_new_apps: bool,
    /// Save the firewall policy before the first change of each day.
    pub daily_backup: bool,
    /// "bytes" (MB/s) or "bits" (Mbit/s).
    pub rate_unit: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            language: "system".into(),
            notify_new_apps: true,
            daily_backup: true,
            rate_unit: "bytes".into(),
        }
    }
}

impl Settings {
    /// Settings from the window, with unknown values set back to their defaults.
    pub fn sanitized(mut self) -> Self {
        let defaults = Self::default();
        if !["system", "dark", "light"].contains(&self.theme.as_str()) {
            self.theme = defaults.theme;
        }
        if !["system", "en", "tr"].contains(&self.language.as_str()) {
            self.language = defaults.language;
        }
        if !["bytes", "bits"].contains(&self.rate_unit.as_str()) {
            self.rate_unit = defaults.rate_unit;
        }
        self
    }
}

pub struct Store {
    file: Option<PathBuf>,
    pub settings: Settings,
}

impl Store {
    pub fn open(config_dir: Option<PathBuf>) -> Self {
        let file = config_dir.map(|d| d.join("settings.json"));
        let settings = file
            .as_ref()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
            .map(Settings::sanitized)
            .unwrap_or_default();
        Self { file, settings }
    }

    pub fn save(&mut self, settings: Settings) -> Result<(), String> {
        let settings = settings.sanitized();
        if let Some(file) = &self.file {
            if let Some(dir) = file.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            let text = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
            std::fs::write(file, text).map_err(|e| e.to_string())?;
        }
        self.settings = settings;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_values_fall_back_to_defaults() {
        let s = Settings {
            theme: "neon".into(),
            language: "fr".into(),
            rate_unit: "furlongs".into(),
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.theme, "system");
        assert_eq!(s.language, "system");
        assert_eq!(s.rate_unit, "bytes");
    }

    #[test]
    fn settings_survive_a_restart_and_missing_fields_take_defaults() {
        let dir = std::env::temp_dir().join(format!("netswitch-settings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut store = Store::open(Some(dir.clone()));
        assert_eq!(store.settings, Settings::default());
        store
            .save(Settings {
                theme: "light".into(),
                notify_new_apps: false,
                ..Settings::default()
            })
            .unwrap();
        let again = Store::open(Some(dir.clone()));
        assert_eq!(again.settings.theme, "light");
        assert!(!again.settings.notify_new_apps);

        std::fs::write(dir.join("settings.json"), r#"{"theme":"dark"}"#).unwrap();
        let partial = Store::open(Some(dir.clone()));
        assert_eq!(partial.settings.theme, "dark");
        assert!(partial.settings.daily_backup, "a missing field takes its default");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
