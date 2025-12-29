use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomColors {
    pub primary_color: String,
    pub background_color: String,
    pub text_color: String,
}

impl Default for CustomColors {
    fn default() -> Self {
        Self {
            primary_color: "".to_string(),
            background_color: "".to_string(),
            text_color: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub opacity: f64,
    pub font_size: i32,
    pub always_on_top: bool,
    pub auto_start: bool,
    pub notifications: bool,
    pub sound: bool,
    pub sound_volume: f64,
    pub http_port: u16,
    pub custom_colors: CustomColors,
    pub reminder_threshold: i32,
    pub do_not_disturb: bool,
    pub do_not_disturb_start: String,
    pub do_not_disturb_end: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            opacity: 0.85,
            font_size: 14,
            always_on_top: true,
            auto_start: false,
            notifications: true,
            sound: true,
            sound_volume: 0.7,
            http_port: 31415,
            custom_colors: CustomColors::default(),
            reminder_threshold: 100,
            do_not_disturb: false,
            do_not_disturb_start: "22:00".to_string(),
            do_not_disturb_end: "08:00".to_string(),
        }
    }
}

impl AppSettings {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }
}

pub struct SettingsState {
    pub settings: Mutex<AppSettings>,
    pub path: PathBuf,
}

impl SettingsState {
    pub fn new(app: &AppHandle) -> Self {
        let path = app
            .path()
            .app_config_dir()
            .expect("Failed to get app config dir")
            .join("settings.json");
        
        let settings = AppSettings::load(&path);
        
        Self {
            settings: Mutex::new(settings),
            path,
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let settings = self.settings.lock().map_err(|_| "Failed to lock settings mutex")?;
        settings.save(&self.path)
    }
}
