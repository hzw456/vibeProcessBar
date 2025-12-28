#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Runtime, WindowEvent};
use serde_json::json;
use std::process::Command;

mod window_manager;
mod http_server;

#[tauri::command]
async fn activate_window<R: Runtime>(window: tauri::Window<R>, window_id: String) -> Result<(), String> {
    window_manager::activate_window(window, window_id).await
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
        let script = match ide {
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
    window: tauri::Window<R>,
    title: String,
    body: String
) -> Result<(), String> {
    window.notify(&title, &body).map_err(|e| e.to_string())
}

fn main() {
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
            activate_ide_window
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            http_server::start_server_background(31415);

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
