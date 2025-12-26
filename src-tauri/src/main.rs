#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Runtime, WindowEvent};
use serde_json::json;

mod window_manager;

#[tauri::command]
async fn activate_window<R: Runtime>(window: tauri::Window<R>, window_id: String) -> Result<(), String> {
    window_manager::activate_window(window, window_id).await
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
            open_url
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            
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
