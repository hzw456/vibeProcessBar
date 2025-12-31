#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Runtime, WindowEvent, Emitter};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use serde_json::json;
use std::process::Command;
use tracing::info;

mod window_manager;
mod http_server;
mod settings;

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
        // Calculate position: center of screen effectively
        // We'll let the OS decide or center it
        let _ = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("设置")
        .inner_size(800.0, 600.0)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .decorations(true) // Use system title bar
        .transparent(false) // Standard opaque window
        .build()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn activate_ide_window<R: Runtime>(
    window: tauri::Window<R>,
    ide: String,
    window_title: Option<String>
) -> Result<(), String> {
    activate_ide(&ide, window_title.as_deref())
}

fn activate_ide(ide: &str, window_title: Option<&str>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let script = match ide.to_lowercase().as_str() {
            "cursor" => {
                if let Some(title) = window_title {
                    format!(
                        r#"
                        tell application "Cursor"
                            activate
                            delay 0.5
                            tell application "System Events"
                                keystroke "p" using command down
                            end tell
                        end tell
                        tell application "System Events" to keystroke "{}"
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
            "kiro" => {
                if let Some(title) = window_title {
                    // For Kiro, search for window containing the title and raise it
                    format!(
                        r#"
                        tell application "System Events"
                            set kiroProcess to application process "Kiro"
                            set frontmost of kiroProcess to true
                            
                            set winCount to count of windows of kiroProcess
                            set searchTitle to "{}"
                            
                            repeat with i from 1 to winCount
                                try
                                    set w to window i of kiroProcess
                                    set winTitle to title of w
                                    if winTitle contains searchTitle then
                                        perform action "AXRaise" of w
                                        exit repeat
                                    end if
                                end try
                            end repeat
                        end tell
                        tell application "Kiro" to activate
                    "#,
                        title
                    )
                } else {
                    r#"
                    tell application "Kiro" to activate
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
                r#"
                tell application "Visual Studio Code"
                    activate
                    delay 0.3
                end tell
                "#.to_string()
            }
            "antigravity" => {
                if let Some(title) = window_title {
                    // For Antigravity, search for window containing the title and raise it
                    format!(
                        r#"
                        tell application "System Events"
                            set agProcess to application process "Antigravity"
                            set frontmost of agProcess to true
                            
                            set winCount to count of windows of agProcess
                            set searchTitle to "{}"
                            
                            repeat with i from 1 to winCount
                                try
                                    set w to window i of agProcess
                                    set winTitle to title of w
                                    if winTitle contains searchTitle then
                                        perform action "AXRaise" of w
                                        exit repeat
                                    end if
                                end try
                            end repeat
                        end tell
                        tell application "Antigravity" to activate
                    "#,
                        title
                    )
                } else {
                    r#"
                    tell application "Antigravity" to activate
                    "#.to_string()
                }
            }
            _ => {
                return Err(format!("Unknown IDE: {}", ide));
            }
        };

        let output = Command::new("osascript")
            .args(&["-e", &script])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let ide_exe = match ide {
            "cursor" => "Cursor.exe",
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
            "claude" | "claude-code" => "Claude",
            "vscode" | "visual studio code" => "code" ,
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
    // Notification functionality - can be implemented with tauri-plugin-notification if needed
    Ok(())
}

#[tauri::command]
async fn get_app_settings(state: tauri::State<'_, settings::SettingsState>) -> Result<settings::AppSettings, String> {
    let settings = state.settings.lock().map_err(|_| "Failed to lock settings mutex")?;
    Ok(settings.clone())
}

#[tauri::command]
async fn update_app_settings<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, settings::SettingsState>,
    new_settings: settings::AppSettings
) -> Result<(), String> {
    {
        let mut settings = state.settings.lock().map_err(|_| "Failed to lock settings mutex")?;
        *settings = new_settings.clone();
    } // Drop lock before saving/emitting

    state.save()?; // Save to disk

    // Emit event to all windows
    app.emit("settings-changed", new_settings).map_err(|e| e.to_string())?;
    
    Ok(())
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
            open_settings_window,
            get_translated_string,
            get_app_settings,
            update_app_settings
        ])
        .setup(|app| {
            // Initialize settings state
            let settings_state = settings::SettingsState::new(app.app_handle());
            app.manage(settings_state);

            let window = app.get_webview_window("main").unwrap();

            // Set webview background to transparent
            #[cfg(target_os = "macos")]
            {
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
                // Apply vibrancy effect for macOS - this makes the window background translucent
                let _ = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, Some(NSVisualEffectState::Active), Some(12.0));
            }

            // Apply blur effect for Windows
            #[cfg(target_os = "windows")]
            {
                use window_vibrancy::apply_blur;
                let _ = apply_blur(&window, Some((18, 18, 18, 200)));
            }

            http_server::start_server_background(31415);
            info!(port = 31415, "HTTP server started on port 31415");

            // Setup system tray
            let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            // Load tray icon from PNG file
            let icon_bytes = include_bytes!("../icons/32x32.png");
            let img = image::load_from_memory(icon_bytes).expect("Failed to load icon");
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);

            info!("Creating system tray with icon {}x{}", width, height);

            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .icon_as_template(false)
                .tooltip("Vibe Process Bar")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "settings" => {
                            // Show settings window
                            let _ = open_settings_window(app.app_handle().clone());
                        }
                        "quit" => {
                            // Exit the application
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            info!("System tray created successfully with id: {:?}", tray.id());
            
            // Leak the tray to prevent it from being dropped
            // This is safe because we want the tray to live for the entire application lifetime
            Box::leak(Box::new(tray));

            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    window_clone.hide().unwrap();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
