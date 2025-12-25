#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, WindowEvent};

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
fn set_window_transparency(window: tauri::Window, transparent: bool) {
    window.set_effects(tauri::window::EffectsConfig::default()).ok();
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            minimize_window,
            close_window,
            set_window_always_on_top,
            set_window_transparency,
            show_window,
            hide_window,
            get_window_position,
            set_window_position
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
