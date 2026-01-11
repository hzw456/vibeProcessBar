use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tracing::info;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone)]
pub struct DatabaseState {
    pub connection: Arc<Mutex<Connection>>,
    pub path: PathBuf,
}

impl DatabaseState {
    pub fn new(app: &AppHandle) -> Self {
        let path = app
            .path()
            .app_config_dir()
            .expect("Failed to get app config dir")
            .join("settings.db");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create database directory");
        }

        let conn = match Connection::open(&path) {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Failed to open database: {}", e);
                panic!("Database initialization failed: {}", e);
            }
        };

        let schema = include_str!("../schema/v1.sql");
        if let Err(e) = conn.execute_batch(schema) {
            tracing::error!("Failed to apply schema: {}", e);
        }

        // Migration: Add http_host column if it doesn't exist
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN http_host TEXT DEFAULT '127.0.0.1'",
            [],
        );

        info!("Database initialized at: {:?}", path);

        Self {
            connection: Arc::new(Mutex::new(conn)),
            path,
        }
    }

    pub fn get_connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.connection.lock().unwrap()
    }

    pub fn migrate_from_json(&self, json_path: &Path) {
        if !json_path.exists() {
            return;
        }

        info!("Migrating settings from JSON: {:?}", json_path);

        let content = match std::fs::read_to_string(json_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to read JSON settings: {}", e);
                return;
            }
        };

        let settings: serde_json::Value = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to parse JSON settings: {}", e);
                return;
            }
        };

        let conn = self.get_connection();

        let theme = settings
            .get("theme")
            .and_then(|v| v.as_str())
            .unwrap_or("dark");
        let opacity = settings
            .get("opacity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.85);
        let font_size = settings
            .get("fontSize")
            .and_then(|v| v.as_i64())
            .unwrap_or(14);
        let always_on_top = settings
            .get("alwaysOnTop")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let auto_start = settings
            .get("autoStart")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let notifications = settings
            .get("notifications")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let sound = settings
            .get("sound")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let sound_volume = settings
            .get("soundVolume")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);
        let http_port = settings
            .get("httpPort")
            .and_then(|v| v.as_u64())
            .unwrap_or(31415) as u16;
        let custom_colors = serde_json::to_string(
            &settings
                .get("customColors")
                .unwrap_or(&serde_json::json!({})),
        )
        .unwrap_or_else(|_| "{}".to_string());
        let reminder_threshold = settings
            .get("reminderThreshold")
            .and_then(|v| v.as_i64())
            .unwrap_or(100);
        let do_not_disturb = settings
            .get("doNotDisturb")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let do_not_disturb_start = settings
            .get("doNotDisturbStart")
            .and_then(|v| v.as_str())
            .unwrap_or("22:00")
            .to_string();
        let do_not_disturb_end = settings
            .get("doNotDisturbEnd")
            .and_then(|v| v.as_str())
            .unwrap_or("08:00")
            .to_string();
        let window_visible = settings
            .get("windowVisible")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let language = settings
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("en");

        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO settings (
                id, theme, opacity, font_size, always_on_top, auto_start,
                notifications, sound, sound_volume, http_port, custom_colors,
                reminder_threshold, do_not_disturb, do_not_disturb_start,
                do_not_disturb_end, window_visible, language, updated_at
            ) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            (
                theme,
                opacity,
                font_size as f64,
                always_on_top as i32,
                auto_start as i32,
                notifications as i32,
                sound as i32,
                sound_volume,
                http_port as i32,
                custom_colors,
                reminder_threshold as i32,
                do_not_disturb as i32,
                do_not_disturb_start,
                do_not_disturb_end,
                window_visible as i32,
                language,
            ),
        ) {
            tracing::error!("Failed to insert settings: {}", e);
            return;
        }

        drop(conn);

        if let Err(e) = std::fs::rename(json_path, json_path.with_extension("json.bak")) {
            tracing::error!("Failed to backup JSON settings: {}", e);
        } else {
            info!("Settings migrated from JSON successfully");
        }
    }
}
