use rusqlite::Result as SqliteResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

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
    pub block_plugin_status: bool, // 屏蔽插件状态上报
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
            block_plugin_status: true, // 默认开启屏蔽
        }
    }
}

impl AppSettings {
    pub fn load(conn: &rusqlite::Connection) -> Self {
        let mut stmt = match conn.prepare_cached(
            "SELECT theme, opacity, font_size, always_on_top, auto_start,
                    sound, sound_volume, http_host, http_port,
                    window_visible, language, block_plugin_status
             FROM settings WHERE id = 1",
        ) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };

        match stmt.query_row([], |row| {
            Ok(AppSettings {
                theme: row.get(0)?,
                opacity: row.get(1)?,
                font_size: row.get(2)?,
                always_on_top: row.get::<_, i32>(3)? != 0,
                auto_start: row.get::<_, i32>(4)? != 0,
                sound: row.get::<_, i32>(5)? != 0,
                sound_volume: row.get(6)?,
                http_host: row
                    .get::<_, String>(7)
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                http_port: row.get::<_, i32>(8)? as u16,
                window_visible: row.get::<_, i32>(9)? != 0,
                language: row.get(10)?,
                block_plugin_status: row.get::<_, i32>(11).unwrap_or(1) != 0,
            })
        }) {
            Ok(settings) => settings,
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, conn: &rusqlite::Connection) -> SqliteResult<usize> {
        conn.execute(
            "INSERT OR REPLACE INTO settings (
                id, theme, opacity, font_size, always_on_top, auto_start,
                sound, sound_volume, http_host, http_port,
                window_visible, language, block_plugin_status, updated_at
            ) VALUES (
                1, :theme, :opacity, :font_size, :always_on_top, :auto_start,
                :sound, :sound_volume, :http_host, :http_port,
                :window_visible, :language, :block_plugin_status, CURRENT_TIMESTAMP
            )",
            rusqlite::named_params! {
                ":theme": &self.theme,
                ":opacity": self.opacity,
                ":font_size": self.font_size as f64,
                ":always_on_top": self.always_on_top as i32,
                ":auto_start": self.auto_start as i32,
                ":sound": self.sound as i32,
                ":sound_volume": self.sound_volume,
                ":http_host": &self.http_host,
                ":http_port": self.http_port as i32,
                ":window_visible": self.window_visible as i32,
                ":language": &self.language,
                ":block_plugin_status": self.block_plugin_status as i32,
            },
        )
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

        let settings = AppSettings::default();

        Self {
            settings: Mutex::new(settings),
            path,
        }
    }

    pub fn save(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        let settings = self
            .settings
            .lock()
            .map_err(|_| "Failed to lock settings mutex")?;
        settings.save(conn).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(&self, conn: &rusqlite::Connection) {
        let settings = AppSettings::load(conn);
        *self.settings.lock().unwrap() = settings;
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }
}
