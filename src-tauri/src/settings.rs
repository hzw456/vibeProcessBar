use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;
use rusqlite::Result as SqliteResult;

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
    pub window_visible: bool,
    pub language: String,
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
            window_visible: true,
            language: "en".to_string(),
        }
    }
}

impl AppSettings {
    pub fn load(conn: &rusqlite::Connection) -> Self {
        let mut stmt = match conn.prepare_cached(
            "SELECT theme, opacity, font_size, always_on_top, auto_start,
                    notifications, sound, sound_volume, http_port, custom_colors,
                    reminder_threshold, do_not_disturb, do_not_disturb_start,
                    do_not_disturb_end, window_visible, language
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
                notifications: row.get::<_, i32>(5)? != 0,
                sound: row.get::<_, i32>(6)? != 0,
                sound_volume: row.get(7)?,
                http_port: row.get::<_, i32>(8)? as u16,
                custom_colors: {
                    let json_str: String = row.get(9)?;
                    serde_json::from_str(&json_str).unwrap_or_default()
                },
                reminder_threshold: row.get::<_, i32>(10)?,
                do_not_disturb: row.get::<_, i32>(11)? != 0,
                do_not_disturb_start: row.get(12)?,
                do_not_disturb_end: row.get(13)?,
                window_visible: row.get::<_, i32>(14)? != 0,
                language: row.get(15)?,
            })
        }) {
            Ok(settings) => settings,
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, conn: &rusqlite::Connection) -> SqliteResult<usize> {
        let custom_colors_json = serde_json::to_string(&self.custom_colors)
            .unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "INSERT OR REPLACE INTO settings (
                id, theme, opacity, font_size, always_on_top, auto_start,
                notifications, sound, sound_volume, http_port, custom_colors,
                reminder_threshold, do_not_disturb, do_not_disturb_start,
                do_not_disturb_end, window_visible, language, updated_at
            ) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            (
                &self.theme,
                self.opacity,
                self.font_size as f64,
                self.always_on_top as i32,
                self.auto_start as i32,
                self.notifications as i32,
                self.sound as i32,
                self.sound_volume,
                self.http_port as i32,
                custom_colors_json,
                self.reminder_threshold as i32,
                self.do_not_disturb as i32,
                &self.do_not_disturb_start,
                &self.do_not_disturb_end,
                self.window_visible as i32,
                &self.language,
            ),
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
        let settings = self.settings.lock().map_err(|_| "Failed to lock settings mutex")?;
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
