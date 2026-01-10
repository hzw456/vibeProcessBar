#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Runtime, WindowEvent, Emitter};
use tauri::menu::MenuItem;
use tauri::tray::TrayIconBuilder;
use serde_json::json;
use std::process::Command;
use tracing::info;

mod window_manager;
mod http_server;
mod settings;
mod db;

use settings::{AppSettings, SettingsState};
use db::DatabaseState;

#[tauri::command]
async fn activate_window<R: Runtime>(window: tauri::Window<R>, window_id: String) -> Result<(), String> {
    window_manager::activate_window(window, window_id).await
}

#[tauri::command]
async fn get_translated_string<R: Runtime>(
    window: tauri::Window<R>,
    key: String
) -> Result<String, String> {
    window.emit("get-translated-string", key.clone())
        .map_err(|e| e.to_string())?;
    Ok(key)
}

#[tauri::command]
fn open_settings_window<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _ = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("Settings")
        .inner_size(800.0, 600.0)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .decorations(true)
        .transparent(false)
        .build()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// IDE bundle identifiers for window scanning
const IDE_BUNDLES: &[(&str, &str)] = &[
    ("com.microsoft.VSCode", "vscode"),
    ("com.todesktop.230313mzl4w4u92", "cursor"),
    ("dev.kiro.desktop", "kiro"),
    ("com.google.antigravity", "antigravity"),
    ("com.codeium.windsurf", "windsurf"),
    ("com.trae.app", "trae"),
];

#[derive(serde::Serialize, Clone, Debug)]
pub struct IdeWindow {
    pub bundle_id: String,
    pub ide: String,
    pub window_title: String,
    pub window_index: i32,
}

// Global state for storing scanned IDE windows for tray menu
lazy_static::lazy_static! {
    static ref SCANNED_WINDOWS: std::sync::Mutex<Vec<IdeWindow>> = std::sync::Mutex::new(Vec::new());
}

/// Scan all IDE windows using a single AppleScript call
/// This avoids the bug where querying by PID returns incomplete results
#[tauri::command]
async fn get_ide_windows() -> Result<Vec<IdeWindow>, String> {
    #[cfg(target_os = "macos")]
    {
        let mut all_windows = Vec::new();
        
        // Use a single AppleScript to iterate all Electron processes and get their windows
        // Format: appPath|||pid|||winName1|||winName2\n
        let script = r#"
            set output to ""
            tell application "System Events"
                repeat with p in (every application process whose name is "Electron")
                    try
                        set pId to unix id of p
                        set appFile to application file of p
                        set appPath to POSIX path of appFile
                        set winNames to name of every window of p
                        set AppleScript's text item delimiters to "|||"
                        set winNamesStr to winNames as text
                        if winNamesStr is not "" then
                            set output to output & appPath & ":::" & pId & ":::" & winNamesStr & "\n"
                        end if
                    end try
                end repeat
            end tell
            return output
        "#;
        
        let output = Command::new("osascript")
            .args(&["-e", script])
            .output();
        
        if let Ok(result) = output {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                
                for line in stdout.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    
                    // Format: appPath:::pid:::winName1|||winName2|||...
                    let parts: Vec<&str> = line.splitn(3, ":::").collect();
                    if parts.len() != 3 {
                        continue;
                    }
                    
                    let app_path = parts[0];
                    let pid = parts[1];
                    let win_names_str = parts[2];
                    
                    // Determine IDE type from app path
                    let ide = if app_path.contains("Antigravity") {
                        "antigravity"
                    } else if app_path.contains("Kiro") {
                        "kiro"
                    } else if app_path.contains("Cursor") {
                        "cursor"
                    } else if app_path.contains("Windsurf") {
                        "windsurf"
                    } else if app_path.contains("Trae") {
                        "trae"
                    } else {
                        "vscode"
                    };
                    
                    // Split window names by |||
                    let win_names: Vec<&str> = win_names_str.split("|||").collect();
                    for (idx, win_title) in win_names.iter().enumerate() {
                        let win_title = win_title.trim();
                        if !win_title.is_empty() {
                            all_windows.push(IdeWindow {
                                bundle_id: format!("pid:{}", pid),
                                ide: ide.to_string(),
                                window_title: win_title.to_string(),
                                window_index: (idx + 1) as i32,
                            });
                        }
                    }
                }
            }
        }
        
        info!("Scanned {} IDE windows", all_windows.len());
        
        // Update global state for tray menu
        if let Ok(mut windows) = SCANNED_WINDOWS.lock() {
            *windows = all_windows.clone();
        }
        
        Ok(all_windows)
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Ok(Vec::new())
    }
}

#[tauri::command]
async fn activate_ide_window<R: Runtime>(
    _window: tauri::Window<R>,
    ide: String,
    window_title: Option<String>,
    project_path: Option<String>,
    active_file: Option<String>
) -> Result<(), String> {
    activate_ide(&ide, window_title.as_deref(), project_path.as_deref(), active_file.as_deref())
}

fn activate_ide(ide: &str, window_title: Option<&str>, project_path: Option<&str>, active_file: Option<&str>) -> Result<(), String> {
    info!("activate_ide called with ide={}, window_title={:?}, project_path={:?}, active_file={:?}", ide, window_title, project_path, active_file);
    #[cfg(target_os = "macos")]
    {
        let script = match ide.to_lowercase().as_str() {
            // VSCode-based AI IDEs (forks with extension support)
            "cursor" => {
                if let Some(title) = window_title {
                    format!(
                        r#"
                        tell application "Cursor"
                            activate
                            delay 0.5
                            tell application "System Events" to keystroke "{}"
                        end tell
                    "#,
                        title
                    )
                } else {
                    r#"
                    tell application "Cursor"
                        activate
                        delay 0.3
                    end tell
                    "#.to_string()
                }
            }
            "windsurf" | "codeium" | "codeium editor" => {
                if let Some(title) = window_title {
                    format!(
                        r#"
                        tell application "Windsurf"
                            activate
                            delay 0.5
                            tell application "System Events" to keystroke "{}"
                        end tell
                    "#,
                        title
                    )
                } else {
                    r#"
                    tell application "Windsurf"
                        activate
                        delay 0.3
                    end tell
                    "#.to_string()
                }
            }
            "trae" => {
                if let Some(title) = window_title {
                    format!(
                        r#"
                        tell application "Trae"
                            activate
                            delay 0.5
                            tell application "System Events" to keystroke "{}"
                        end tell
                    "#,
                        title
                    )
                } else {
                    r#"
                    tell application "Trae"
                        activate
                        delay 0.3
                    end tell
                    "#.to_string()
                }
            }
            "void" | "void editor" | "void-editor" => {
                if let Some(title) = window_title {
                    format!(
                        r#"
                        tell application "Void"
                            activate
                            delay 0.5
                            tell application "System Events" to keystroke "{}"
                        end tell
                    "#,
                        title
                    )
                } else {
                    r#"
                    tell application "Void"
                        activate
                        delay 0.3
                    end tell
                    "#.to_string()
                }
            }
            "pearai" | "pear-ai" | "pear ai" => {
                r#"
                tell application "PearAI"
                    activate
                    delay 0.3
                end tell
                "#.to_string()
            }
            "blueberryai" | "blueberry ai" | "blueberry" => {
                r#"
                tell application "BlueberryAI"
                    activate
                    delay 0.3
                end tell
                "#.to_string()
            }
            "aide" | "codestoryai" | "codestory ai" => {
                r#"
                tell application "Aide"
                    activate
                    delay 0.3
                end tell
                "#.to_string()
            }
            "codebuddy" | "code buddy" | "tencent codebuddy" => {
                r#"
                tell application "CodeBuddy"
                    activate
                    delay 0.3
                end tell
                "#.to_string()
            }
            "kilocode" | "kilo-code" | "kilo" => {
                r#"
                tell application "Kilo Code"
                    activate
                    delay 0.3
                end tell
                "#.to_string()
            }
            "kiro" => {
                // Matching priority: IDE -> workspace (window_title) -> active_file
                let workspace_search = window_title.unwrap_or("");
                let file_search = active_file.unwrap_or("");
                
                if !workspace_search.is_empty() || !file_search.is_empty() {
                    format!(
                        r#"
                        tell application "System Events"
                            set workspaceTerm to "{}"
                            set fileTerm to "{}"
                            set foundWindow to false
                            repeat with p in (every application process whose name is "Electron")
                                try
                                    set appPath to POSIX path of (application file of p)
                                    if appPath contains "Kiro" then
                                        -- First try workspace (window_title)
                                        if workspaceTerm is not "" then
                                            repeat with w in (every window of p)
                                                set winTitle to title of w
                                                if winTitle contains workspaceTerm then
                                                    set frontmost of p to true
                                                    perform action "AXRaise" of w
                                                    set foundWindow to true
                                                    exit repeat
                                                end if
                                            end repeat
                                        end if
                                        -- If workspace not found, try active_file
                                        if not foundWindow and fileTerm is not "" then
                                            repeat with w in (every window of p)
                                                set winTitle to title of w
                                                if winTitle contains fileTerm then
                                                    set frontmost of p to true
                                                    perform action "AXRaise" of w
                                                    set foundWindow to true
                                                    exit repeat
                                                end if
                                            end repeat
                                        end if
                                        if foundWindow then exit repeat
                                    end if
                                end try
                            end repeat
                        end tell
                        tell application "Kiro" to activate
                    "#,
                        workspace_search, file_search
                    )
                } else {
                    r#"
                    tell application "Kiro" to activate
                    "#.to_string()
                }
            }
            "antigravity" => {
                // Matching priority: IDE -> workspace (window_title) -> active_file
                let workspace_search = window_title.unwrap_or("");
                let file_search = active_file.unwrap_or("");
                
                if !workspace_search.is_empty() || !file_search.is_empty() {
                    format!(
                        r#"
                        tell application "System Events"
                            set workspaceTerm to "{}"
                            set fileTerm to "{}"
                            set foundWindow to false
                            repeat with p in (every application process whose name is "Electron")
                                try
                                    set appPath to POSIX path of (application file of p)
                                    if appPath contains "Antigravity" then
                                        -- First try workspace (window_title)
                                        if workspaceTerm is not "" then
                                            repeat with w in (every window of p)
                                                set winTitle to title of w
                                                if winTitle contains workspaceTerm then
                                                    set frontmost of p to true
                                                    perform action "AXRaise" of w
                                                    set foundWindow to true
                                                    exit repeat
                                                end if
                                            end repeat
                                        end if
                                        -- If workspace not found, try active_file
                                        if not foundWindow and fileTerm is not "" then
                                            repeat with w in (every window of p)
                                                set winTitle to title of w
                                                if winTitle contains fileTerm then
                                                    set frontmost of p to true
                                                    perform action "AXRaise" of w
                                                    set foundWindow to true
                                                    exit repeat
                                                end if
                                            end repeat
                                        end if
                                        if foundWindow then exit repeat
                                    end if
                                end try
                            end repeat
                        end tell
                        tell application "Antigravity" to activate
                    "#,
                        workspace_search, file_search
                    )
                } else {
                    r#"
                    tell application "Antigravity" to activate
                    "#.to_string()
                }
            }
            "claude" | "claude-code" => {
                if let Some(title) = window_title {
                    format!(
                        r#"
                        tell application "Claude"
                            activate
                            delay 0.5
                            tell application "System Events" to keystroke "{}"
                        end tell
                    "#,
                        title
                    )
                } else {
                    r#"
                    tell application "Claude"
                        activate
                        delay 0.3
                    end tell
                    "#.to_string()
                }
            }
            "vscode" | "visual studio code" => {
                // Matching priority: IDE -> workspace (window_title) -> active_file
                let workspace_search = window_title.unwrap_or("");
                let file_search = active_file.unwrap_or("");
                
                if !workspace_search.is_empty() || !file_search.is_empty() {
                    format!(
                        r#"
                        tell application "System Events"
                            set workspaceTerm to "{}"
                            set fileTerm to "{}"
                            set foundWindow to false
                            repeat with p in (every application process whose name is "Electron")
                                try
                                    set appPath to POSIX path of (application file of p)
                                    if appPath contains "Visual Studio Code" then
                                        -- First try workspace (window_title)
                                        if workspaceTerm is not "" then
                                            repeat with w in (every window of p)
                                                set winTitle to title of w
                                                if winTitle contains workspaceTerm then
                                                    set frontmost of p to true
                                                    perform action "AXRaise" of w
                                                    set foundWindow to true
                                                    exit repeat
                                                end if
                                            end repeat
                                        end if
                                        -- If workspace not found, try active_file
                                        if not foundWindow and fileTerm is not "" then
                                            repeat with w in (every window of p)
                                                set winTitle to title of w
                                                if winTitle contains fileTerm then
                                                    set frontmost of p to true
                                                    perform action "AXRaise" of w
                                                    set foundWindow to true
                                                    exit repeat
                                                end if
                                            end repeat
                                        end if
                                        if foundWindow then exit repeat
                                    end if
                                end try
                            end repeat
                        end tell
                        tell application "Visual Studio Code" to activate
                    "#,
                        workspace_search, file_search
                    )
                } else {
                    r#"
                    tell application "Visual Studio Code" to activate
                    "#.to_string()
                }
            }
            _ => {
                return Err(format!("Unknown IDE: {}", ide));
            }
        };

        info!("Running AppleScript for IDE: {}", ide);
        let output = Command::new("osascript")
            .args(&["-e", &script])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            info!("AppleScript failed for {}: {}", ide, stderr);
            return Err(stderr.to_string());
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let ide_exe = match ide {
            "cursor" => "Cursor.exe",
            "windsurf" | "codeium" | "codeium editor" => "Windsurf.exe",
            "trae" => "Trae.exe",
            "void" | "void editor" | "void-editor" => "Void.exe",
            "pearai" | "pear-ai" | "pear ai" => "PearAI.exe",
            "blueberryai" | "blueberry ai" | "blueberry" => "BlueberryAI.exe",
            "aide" | "codestoryai" | "codestory ai" => "Aide.exe",
            "codebuddy" | "code buddy" | "tencent codebuddy" => "CodeBuddy.exe",
            "kilocode" | "kilo-code" | "kilo" => "Kilo Code.exe",
            "kiro" => "Kiro.exe",
            "antigravity" => "Antigravity.exe",
            "claude" | "claude-code" => "Claude.exe",
            "vscode" | "visual studio code" => "Code.exe",
            _ => {
                let output = Command::new("powershell")
                    .args(&["-Command", &format!(r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('^{}`');"#)])
                    .output()
                    .map_err(|e| e.to_string())?;
                if !output.status.success() {
                    return Err(String::from_utf8_lossy(&output.stderr).to_string());
                }
                return Ok(());
            }
        };

        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(r#"Start-Process {}; Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('^{}`');"#, ide_exe),
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let ide_name = match ide {
            "cursor" => "Cursor",
            "windsurf" | "codeium" | "codeium editor" => "Windsurf",
            "trae" => "Trae",
            "void" | "void editor" | "void-editor" => "Void",
            "pearai" | "pear-ai" | "pear ai" => "PearAI",
            "blueberryai" | "blueberry ai" | "blueberry" => "BlueberryAI",
            "aide" | "codestoryai" | "codestory ai" => "Aide",
            "codebuddy" | "code buddy" | "tencent codebuddy" => "CodeBuddy",
            "kilocode" | "kilo-code" | "kilo" => "Kilo Code",
            "kiro" => "Kiro",
            "antigravity" => "Antigravity",
            "claude" | "claude-code" => "Claude",
            "vscode" | "visual studio code" => "code",
            _ => return Err(format!("Unknown IDE: {}", ide)),
        };

        let output = Command::new("wmctrl")
            .args(&["-a", ide_name])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            let fallback = Command::new("xdotool")
                .args(&["search", "--name", ide_name, "windowactivate"])
                .output()
                .map_err(|e| e.to_string())?;

            if !fallback.status.success() {
                return Err(String::from_utf8_lossy(&fallback.stderr).to_string());
            }
        }
        Ok(())
    }
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
#[tauri::command]
fn minimize_window(window: tauri::Window) {
    window.minimize().unwrap();
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    window.close().unwrap();
}

#[tauri::command]
fn set_window_always_on_top(window: tauri::Window, on_top: bool) {
    window.set_always_on_top(on_top).unwrap();
}

#[tauri::command]
fn set_window_opacity(_window: tauri::Window, _opacity: f64) {
}

#[tauri::command]
fn set_window_transparency(_window: tauri::Window, _transparent: bool) {
}

#[tauri::command]
fn show_window(window: tauri::Window) {
    window.show().unwrap();
    window.set_focus().unwrap();
}

#[tauri::command]
fn hide_window(window: tauri::Window) {
    window.hide().unwrap();
}

#[tauri::command]
fn get_window_position(window: tauri::Window) -> (f64, f64) {
    window.inner_position().map(|p| (p.x as f64, p.y as f64)).unwrap_or((0.0, 0.0))
}

#[tauri::command]
fn set_window_position(window: tauri::Window, x: f64, y: f64) {
    window.set_position(tauri::LogicalPosition::new(x, y)).unwrap();
}

#[tauri::command]
fn resize_window(window: tauri::Window, width: f64, height: f64) {
    window.set_size(tauri::LogicalSize::new(width, height)).unwrap();
}

#[tauri::command]
async fn toggle_window_always_on_top<R: Runtime>(window: tauri::Window<R>) -> Result<bool, String> {
    let current = window.is_always_on_top().map_err(|e| e.to_string())?;
    let new_value = !current;
    window.set_always_on_top(new_value).map_err(|e| e.to_string())?;
    Ok(new_value)
}

#[tauri::command]
async fn get_all_windows<R: Runtime>(app: tauri::AppHandle<R>) -> Result<serde_json::Value, String> {
    let windows = app.webview_windows();
    let mut result = Vec::new();
    for (label, win) in windows {
        let position = win.inner_position().ok().map(|p| json!({"x": p.x, "y": p.y}));
        let size = win.inner_size().map_err(|e| e.to_string())?;
        result.push(json!({
            "label": label,
            "position": position,
            "width": size.width,
            "height": size.height
        }));
    }
    Ok(json!(result))
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    webbrowser::open(&url).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_http_server<R: Runtime>(
    _app: tauri::AppHandle<R>,
    port: u16
) -> Result<(), String> {
    http_server::start_server_background(port);
    Ok(())
}

#[tauri::command]
async fn trigger_notification<R: Runtime>(
    _window: tauri::Window<R>,
    _title: String,
    _body: String
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
async fn get_app_settings(
    state: tauri::State<'_, settings::SettingsState>,
    db_state: tauri::State<'_, db::DatabaseState>,
) -> Result<AppSettings, String> {
    let settings = state.get_settings();
    Ok(settings)
}

#[tauri::command]
async fn update_app_settings<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, settings::SettingsState>,
    db_state: tauri::State<'_, db::DatabaseState>,
    new_settings: AppSettings
) -> Result<(), String> {
    {
        let mut settings = state.settings.lock().map_err(|_| "Failed to lock settings mutex")?;
        *settings = new_settings.clone();
    }

    {
        let conn = db_state.get_connection();
        state.save(&conn)?;
    }

    app.emit("settings-changed", new_settings.clone()).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_window_visibility(
    state: tauri::State<'_, settings::SettingsState>,
    _db_state: tauri::State<'_, db::DatabaseState>,
) -> Result<bool, String> {
    let settings = state.get_settings();
    Ok(settings.window_visible)
}

#[tauri::command]
async fn set_window_visibility<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, settings::SettingsState>,
    db_state: tauri::State<'_, db::DatabaseState>,
    visible: bool,
) -> Result<(), String> {
    {
        let mut settings = state.settings.lock().map_err(|_| "Failed to lock settings mutex")?;
        settings.window_visible = visible;
    }

    {
        let conn = db_state.get_connection();
        let settings = state.settings.lock().map_err(|_| "Failed to lock settings mutex")?;
        settings.save(&conn).map_err(|e| e.to_string())?;
    }

    app.emit("window-visibility-changed", visible).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_tray_translations(language: String) -> serde_json::Value {
    let translations: std::collections::HashMap<&str, &str> = match language.as_str() {
        "zh-CN" => vec![
            ("showWindow", "☀ 显示窗口"),
            ("hideWindow", "☾ 隐藏窗口"),
            ("settings", "设置"),
            ("quit", "退出"),
            ("noTasks", "无任务"),
            ("tasks", "任务"),
        ].into_iter().collect(),
        "zh-TW" => vec![
            ("showWindow", "☀ 顯示窗口"),
            ("hideWindow", "☾ 隱藏窗口"),
            ("settings", "設置"),
            ("quit", "退出"),
            ("noTasks", "無任務"),
            ("tasks", "任務"),
        ].into_iter().collect(),
        "de" => vec![
            ("showWindow", "☀ Fenster anzeigen"),
            ("hideWindow", "☾ Fenster ausblenden"),
            ("settings", "Einstellungen"),
            ("quit", "Beenden"),
            ("noTasks", "Keine Aufgaben"),
            ("tasks", "Aufgaben"),
        ].into_iter().collect(),
        "es" => vec![
            ("showWindow", "☀ Mostrar ventana"),
            ("hideWindow", "☾ Ocultar ventana"),
            ("settings", "Configuración"),
            ("quit", "Salir"),
            ("noTasks", "Sin tareas"),
            ("tasks", "Tareas"),
        ].into_iter().collect(),
        "fr" => vec![
            ("showWindow", "☀ Afficher la fenêtre"),
            ("hideWindow", "☾ Masquer la fenêtre"),
            ("settings", "Paramètres"),
            ("quit", "Quitter"),
            ("noTasks", "Aucune tâche"),
            ("tasks", "Tâches"),
        ].into_iter().collect(),
        "ja" => vec![
            ("showWindow", "☀ ウィンドウを表示"),
            ("hideWindow", "☾ ウィンドウを非表示"),
            ("settings", "設定"),
            ("quit", "終了"),
            ("noTasks", "タスクなし"),
            ("tasks", "タスク"),
        ].into_iter().collect(),
        "ko" => vec![
            ("showWindow", "☀ 창 표시"),
            ("hideWindow", "☾ 창 숨기기"),
            ("settings", "설정"),
            ("quit", "종료"),
            ("noTasks", "작업 없음"),
            ("tasks", "작업"),
        ].into_iter().collect(),
        "pt" => vec![
            ("showWindow", "☀ Mostrar janela"),
            ("hideWindow", "☾ Ocultar janela"),
            ("settings", "Configurações"),
            ("quit", "Sair"),
            ("noTasks", "Sem tarefas"),
            ("tasks", "Tarefas"),
        ].into_iter().collect(),
        "ru" => vec![
            ("showWindow", "☀ Показать окно"),
            ("hideWindow", "☾ Скрыть окно"),
            ("settings", "Настройки"),
            ("quit", "Выйти"),
            ("noTasks", "Нет задач"),
            ("tasks", "Задачи"),
        ].into_iter().collect(),
        "ar" => vec![
            ("showWindow", "☀ إظهار النافذة"),
            ("hideWindow", "☾ إخفاء النافذة"),
            ("settings", "الإعدادات"),
            ("quit", "خروج"),
            ("noTasks", "لا توجد مهام"),
            ("tasks", "المهام"),
        ].into_iter().collect(),
        _ => vec![
            ("showWindow", "☀ Show Window"),
            ("hideWindow", "☾ Hide Window"),
            ("settings", "Settings"),
            ("quit", "Quit"),
            ("noTasks", "No tasks"),
            ("tasks", "Tasks"),
        ].into_iter().collect(),
    };

    serde_json::to_value(translations).unwrap_or_default()
}

#[tauri::command]
async fn get_current_language(
    state: tauri::State<'_, settings::SettingsState>,
) -> Result<String, String> {
    let settings = state.get_settings();
    Ok(settings.language)
}

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            minimize_window,
            close_window,
            set_window_always_on_top,
            set_window_opacity,
            set_window_transparency,
            show_window,
            hide_window,
            get_window_position,
            set_window_position,
            resize_window,
            toggle_window_always_on_top,
            get_all_windows,
            open_url,
            start_http_server,
            trigger_notification,
            activate_ide_window,
            get_ide_windows,
            open_settings_window,
            get_translated_string,
            get_app_settings,
            update_app_settings,
            get_window_visibility,
            set_window_visibility,
            get_tray_translations,
            get_current_language,
        ])
        .setup(|app| {
            let app_handle = app.app_handle().clone();
            
            let db_state = DatabaseState::new(&app_handle);
            app.manage(db_state.clone());

            let settings_state = SettingsState::new(&app_handle);
            settings_state.load(&db_state.get_connection());
            app.manage(settings_state);

            let window = app_handle.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            {
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
                let _ = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, Some(NSVisualEffectState::Active), Some(12.0));
            }

            #[cfg(target_os = "windows")]
            {
                use window_vibrancy::apply_blur;
                let _ = apply_blur(&window, Some((18, 18, 18, 200)));
            }

            let db_state = app.state::<db::DatabaseState>();
            let json_path = {
                let app_handle = app.app_handle();
                app_handle.path()
                    .app_config_dir()
                    .expect("Failed to get app config dir")
                    .join("settings.json")
            };

            if json_path.exists() {
                let _ = db_state.migrate_from_json(&json_path);
            }

            http_server::start_server_background(31415);
            info!(port = 31415, "HTTP server started on port 31415");

            let settings_state = app.state::<settings::SettingsState>();
            let current_settings = settings_state.get_settings();
            let language = current_settings.language;
            let translations = get_tray_translations(language);

            let show_window_text = translations.get("showWindow")
                .and_then(|v| v.as_str())
                .unwrap_or("☀ Show Window");
            let hide_window_text = translations.get("hideWindow")
                .and_then(|v| v.as_str())
                .unwrap_or("☾ Hide Window");
            let settings_text = translations.get("settings")
                .and_then(|v| v.as_str())
                .unwrap_or("Settings");
            let quit_text = translations.get("quit")
                .and_then(|v| v.as_str())
                .unwrap_or("Quit");

            let window_toggle_item = MenuItem::with_id(&app_handle, "toggle-window", hide_window_text, true, None::<&str>)?;
            let settings_item = MenuItem::with_id(&app_handle, "settings", settings_text, true, None::<&str>)?;
            let quit_item = MenuItem::with_id(&app_handle, "quit", quit_text, true, None::<&str>)?;

            let icon_bytes = include_bytes!("../icons/32x32.png");
            let img = image::load_from_memory(icon_bytes).expect("Failed to load icon");
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);

            info!("Creating system tray with icon {}x{}", width, height);

            let app_handle_clone = app.app_handle().clone();
            let translations_clone = translations.clone();

            let rebuild_menu = move |app: &tauri::AppHandle, is_visible: bool| {
                let lang = {
                    let state = app.state::<settings::SettingsState>();
                    state.get_settings().language
                };
                let trans = get_tray_translations(lang);
                let window_text = if is_visible {
                    trans.get("hideWindow").and_then(|v| v.as_str()).unwrap_or("☾ Hide Window")
                } else {
                    trans.get("showWindow").and_then(|v| v.as_str()).unwrap_or("☀ Show Window")
                };
                let settings_text = trans.get("settings").and_then(|v| v.as_str()).unwrap_or("Settings");
                let quit_text = trans.get("quit").and_then(|v| v.as_str()).unwrap_or("Quit");

                let window_toggle = MenuItem::with_id(app, "toggle-window", window_text, true, None::<&str>).ok();
                let settings = MenuItem::with_id(app, "settings", settings_text, true, None::<&str>).ok();
                let quit = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>).ok();

                if let (Some(w), Some(s), Some(q)) = (window_toggle, settings, quit) {
                    // Get tasks from http_server state
                    let tasks: Vec<http_server::Task> = http_server::get_state().tasks.lock()
                        .map(|t| t.clone())
                        .unwrap_or_default();
                    
                    // Build menu items
                    let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&w, &s, &q];
                    
                    // Add separator and tasks if any
                    let sep1 = tauri::menu::PredefinedMenuItem::separator(app).ok();
                    let mut task_items: Vec<MenuItem<tauri::Wry>> = Vec::new();
                    
                    for task in &tasks {
                        // Status icon based on task status
                        let status_icon = match task.status.as_str() {
                            "running" => "🔄",
                            "armed" => "⏳",
                            "completed" => "✅",
                            "error" => "❌",
                            "cancelled" => "⏹️",
                            "registered" => "📋",
                            "active" => "👁️",
                            _ => "•",
                        };
                        let title = format!("{} {} - {}", status_icon, task.name, task.status);
                        let id = format!("task_{}", task.id);
                        if let Ok(item) = MenuItem::with_id(app, &id, &title, true, None::<&str>) {
                            task_items.push(item);
                        }
                    }
                    
                    // Add tasks section
                    if !task_items.is_empty() {
                        if let Some(ref sep) = sep1 {
                            items.push(sep);
                        }
                        for item in &task_items {
                            items.push(item);
                        }
                    }
                    
                    if let Ok(menu) = tauri::menu::Menu::with_items(app, &items) {
                        let _ = app.tray_by_id("main-tray").unwrap().set_menu(Some(menu));
                    }
                }
            };

            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .icon_as_template(false)
                .tooltip("Vibe Process Bar")
                .menu(&tauri::menu::Menu::with_items(&app_handle, &[&window_toggle_item, &settings_item, &quit_item])?)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "toggle-window" => {
                            let db_state = app.state::<db::DatabaseState>();
                            let settings_state = app.state::<settings::SettingsState>();
                            
                            let conn = db_state.get_connection();
                            let settings = AppSettings::load(&conn);
                            drop(conn);

                            let window = app.get_webview_window("main").unwrap();
                            let is_visible = window.is_visible().unwrap_or(true);

                            if is_visible {
                                let _ = window.hide();
                                let mut settings = settings_state.settings.lock().unwrap();
                                settings.window_visible = false;
                                drop(settings);
                                let conn = db_state.get_connection();
                                let _ = settings_state.save(&conn);
                                drop(conn);
                                let _ = app.tray_by_id("main-tray").unwrap().set_tooltip(Some("Vibe Process Bar (Hidden)"));
                                rebuild_menu(app, false);
                            } else {
                                let _ = window.show();
                                let mut settings = settings_state.settings.lock().unwrap();
                                settings.window_visible = true;
                                drop(settings);
                                let conn = db_state.get_connection();
                                let _ = settings_state.save(&conn);
                                drop(conn);
                                let _ = app.tray_by_id("main-tray").unwrap().set_tooltip(Some("Vibe Process Bar"));
                                rebuild_menu(app, true);
                            }
                        }
                        "settings" => {
                            let _ = open_settings_window(app.app_handle().clone());
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        other => {
                            // Handle task activation events
                            if other.starts_with("task_") {
                                if let Some(task_id) = other.strip_prefix("task_") {
                                    let tasks: Vec<http_server::Task> = http_server::get_state().tasks.lock()
                                        .map(|t| t.clone())
                                        .unwrap_or_default();
                                    if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
                                        info!("Activating task from tray: {} ({})", task.name, task.ide);
                                        let _ = activate_ide(
                                            &task.ide,
                                            Some(&task.window_title),
                                            task.project_path.as_deref(),
                                            task.active_file.as_deref()
                                        );
                                    }
                                }
                            }
                            // Handle window activation events
                            else if other.starts_with("activate_win_") {
                                if let Some(index_str) = other.strip_prefix("activate_win_") {
                                    if let Ok(index) = index_str.parse::<usize>() {
                                        let windows: Vec<IdeWindow> = SCANNED_WINDOWS.lock()
                                            .map(|w| w.clone())
                                            .unwrap_or_default();
                                        if let Some(win) = windows.get(index) {
                                            info!("Activating window from tray: {} ({})", win.window_title, win.ide);
                                            let _ = activate_ide(&win.ide, Some(&win.window_title), None, None);
                                        }
                                    }
                                }
                            }
                        }
                    }
                })
                .build(app)?;

            info!("System tray created successfully with id: {:?}", tray.id());

            // Periodically check and refresh tray menu only when tasks change
            let app_handle_for_timer = app.app_handle().clone();
            std::thread::spawn(move || {
                let mut last_task_snapshot: String = String::new();
                
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    
                    // Get current tasks and create a snapshot for comparison
                    let tasks: Vec<http_server::Task> = http_server::get_state().tasks.lock()
                        .map(|t| t.clone())
                        .unwrap_or_default();
                    
                    // Create a simple snapshot of task IDs and statuses
                    let current_snapshot: String = tasks.iter()
                        .map(|t| format!("{}:{}", t.id, t.status))
                        .collect::<Vec<_>>()
                        .join(",");
                    
                    // Only rebuild menu if tasks have changed
                    if current_snapshot != last_task_snapshot {
                        last_task_snapshot = current_snapshot;
                        
                        let app = &app_handle_for_timer;
                        let window = app.get_webview_window("main");
                        let is_visible = window.map(|w| w.is_visible().unwrap_or(true)).unwrap_or(true);
                        
                        let lang = {
                            let state = app.state::<settings::SettingsState>();
                            state.get_settings().language
                        };
                        let trans = get_tray_translations(lang);
                        let window_text = if is_visible {
                            trans.get("hideWindow").and_then(|v| v.as_str()).unwrap_or("☾ Hide Window")
                        } else {
                            trans.get("showWindow").and_then(|v| v.as_str()).unwrap_or("☀ Show Window")
                        };
                        let settings_text = trans.get("settings").and_then(|v| v.as_str()).unwrap_or("Settings");
                        let quit_text = trans.get("quit").and_then(|v| v.as_str()).unwrap_or("Quit");

                        let window_toggle = tauri::menu::MenuItem::with_id(app, "toggle-window", window_text, true, None::<&str>).ok();
                        let settings = tauri::menu::MenuItem::with_id(app, "settings", settings_text, true, None::<&str>).ok();
                        let quit = tauri::menu::MenuItem::with_id(app, "quit", quit_text, true, None::<&str>).ok();

                        if let (Some(w), Some(s), Some(q)) = (window_toggle, settings, quit) {
                            let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&w, &s, &q];
                            let sep1 = tauri::menu::PredefinedMenuItem::separator(app).ok();
                            let mut task_items: Vec<tauri::menu::MenuItem<tauri::Wry>> = Vec::new();
                            
                            for task in &tasks {
                                let status_icon = match task.status.as_str() {
                                    "running" => "🔄",
                                    "armed" => "⏳",
                                    "completed" => "✅",
                                    "error" => "❌",
                                    "cancelled" => "⏹️",
                                    "registered" => "📋",
                                    "active" => "👁️",
                                    _ => "•",
                                };
                                let title = format!("{} {} - {}", status_icon, task.name, task.status);
                                let id = format!("task_{}", task.id);
                                if let Ok(item) = tauri::menu::MenuItem::with_id(app, &id, &title, true, None::<&str>) {
                                    task_items.push(item);
                                }
                            }
                            
                            if !task_items.is_empty() {
                                if let Some(ref sep) = sep1 {
                                    items.push(sep);
                                }
                                for item in &task_items {
                                    items.push(item);
                                }
                            }
                            
                            if let Ok(menu) = tauri::menu::Menu::with_items(app, &items) {
                                if let Some(tray) = app.tray_by_id("main-tray") {
                                    let _ = tray.set_menu(Some(menu));
                                }
                            }
                        }
                    }
                }
            });

            Box::leak(Box::new(tray));

            let window_clone = window.clone();
            let app_handle_clone = app.app_handle().clone();

            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let db_state = app_handle_clone.state::<db::DatabaseState>();
                    let settings_state = app_handle_clone.state::<settings::SettingsState>();
                    
                    let conn = db_state.get_connection();
                    let mut settings = settings_state.settings.lock().unwrap();
                    settings.window_visible = false;
                    let _ = settings_state.save(&conn);
                    
                    window_clone.hide().unwrap();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
