use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub opacity: f64,
    pub font_size: i32,
    pub always_on_top: bool,
    pub auto_start: bool,
    pub sound: bool,
    pub sound_volume: f64,
    pub http_host: String,
    pub http_port: u16,
    pub window_visible: bool,
    pub language: String,
    pub block_plugin_status: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            opacity: 0.85,
            font_size: 14,
            always_on_top: true,
            auto_start: false,
            sound: true,
            sound_volume: 0.7,
            http_host: "127.0.0.1".to_string(),
            http_port: 31415,
            window_visible: true,
            language: "en".to_string(),
            block_plugin_status: true,
        }
    }
}

impl AppSettings {
    pub fn load_from_file(path: &PathBuf) -> Self {
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(settings) => settings,
                Err(e) => {
                    error!("Failed to parse settings JSON: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                error!("Failed to read settings file: {}", e);
                Self::default()
            }
        }
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())?;
        info!("Settings saved to {:?}", path);
        Ok(())
    }
}

pub struct SettingsState {
    pub settings: Mutex<AppSettings>,
    pub path: PathBuf,
}

impl SettingsState {
    pub fn new(app: &tauri::AppHandle) -> Self {
        let path = app
            .path()
            .app_config_dir()
            .expect("Failed to get app config dir")
            .join("settings.json");

        let settings = AppSettings::load_from_file(&path);
        info!("Settings loaded from {:?}", path);

        Self {
            settings: Mutex::new(settings),
            path,
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let settings = self
            .settings
            .lock()
            .map_err(|_| "Failed to lock settings mutex")?;
        settings.save_to_file(&self.path)
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update_settings(&self, new_settings: AppSettings) -> Result<(), String> {
        {
            let mut settings = self
                .settings
                .lock()
                .map_err(|_| "Failed to lock settings mutex")?;
            *settings = new_settings;
        }
        self.save()
    }
}
