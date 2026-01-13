#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::json;
use tauri::menu::MenuItem;
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, Runtime, WindowEvent};
use tracing::info;

mod http_server;
mod settings;
mod window_manager;

use settings::{AppSettings, SettingsState};
use window_manager::IdeWindow;

// ============================================================================
// Window Commands
// ============================================================================

#[tauri::command]
async fn activate_window<R: Runtime>(
    window: tauri::Window<R>,
    window_id: String,
) -> Result<(), String> {
    window_manager::activate_window(window, window_id).await
}

#[tauri::command]
async fn activate_ide_window<R: Runtime>(
    _window: tauri::Window<R>,
    ide: String,
    window_title: Option<String>,
    project_path: Option<String>,
    active_file: Option<String>,
) -> Result<(), String> {
    window_manager::activate_ide(
        &ide,
        window_title.as_deref(),
        project_path.as_deref(),
        active_file.as_deref(),
    )
}

#[tauri::command]
async fn get_ide_windows() -> Result<Vec<IdeWindow>, String> {
    Ok(window_manager::scan_ide_windows())
}

#[tauri::command]
fn open_settings_window<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        // 窗口已存在，确保显示并聚焦
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        // 创建新窗口，设置 visible: true 确保立即显示
        let window = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("index.html?type=settings".into()),
        )
        .title("Settings")
        .inner_size(800.0, 600.0)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .decorations(true)
        .transparent(false)
        .visible(true)
        .focused(true)
        .build()
        .map_err(|e| e.to_string())?;
        
        // 确保窗口显示并聚焦
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

// ============================================================================
// Task Commands (Rust层合并逻辑)
// ============================================================================

#[tauri::command]
async fn get_tasks() -> Result<Vec<http_server::Task>, String> {
    Ok(http_server::get_merged_tasks())
}

// ============================================================================
// Settings Commands
// ============================================================================

#[tauri::command]
async fn get_app_settings(
    state: tauri::State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    Ok(state.get_settings())
}

#[tauri::command]
async fn update_app_settings<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, SettingsState>,
    new_settings: AppSettings,
) -> Result<(), String> {
    // 更新 HTTP server 的屏蔽设置
    http_server::set_block_plugin_status(new_settings.block_plugin_status);

    // 保存设置
    state.update_settings(new_settings.clone())?;

    // 通过 emit 发送到所有窗口
    app.emit("settings-changed", &new_settings)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_window_visibility(
    state: tauri::State<'_, SettingsState>,
) -> Result<bool, String> {
    Ok(state.get_settings().window_visible)
}

#[tauri::command]
async fn set_window_visibility<R: Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, SettingsState>,
    visible: bool,
) -> Result<(), String> {
    let mut settings = state.get_settings();
    settings.window_visible = visible;
    state.update_settings(settings)?;

    app.emit("window-visibility-changed", visible)
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Window Management Commands
// ============================================================================

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
fn set_window_always_on_top(window: tauri::Window, on_top: bool) {
    let _ = window.set_always_on_top(on_top);
}

#[tauri::command]
async fn set_always_on_top<R: Runtime>(
    app: tauri::AppHandle<R>,
    always_on_top: bool,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(always_on_top).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn set_auto_start(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let app_path = "/Applications/vibe-process-bar.app";
        
        if enabled {
            let script = format!(
                r#"tell application "System Events" to make login item at end with properties {{path:"{}", hidden:false}}"#,
                app_path
            );
            Command::new("osascript")
                .args(&["-e", &script])
                .output()
                .map_err(|e| e.to_string())?;
        } else {
            let script = r#"tell application "System Events" to delete login item "vibe-process-bar""#;
            let _ = Command::new("osascript").args(&["-e", script]).output();
        }
    }
    Ok(())
}

#[tauri::command]
fn set_window_opacity(_window: tauri::Window, _opacity: f64) {}

#[tauri::command]
fn set_window_transparency(_window: tauri::Window, _transparent: bool) {}

#[tauri::command]
fn show_window(window: tauri::Window) {
    let _ = window.show();
    let _ = window.set_focus();
}

#[tauri::command]
fn hide_window(window: tauri::Window) {
    let _ = window.hide();
}

#[tauri::command]
fn get_window_position(window: tauri::Window) -> (f64, f64) {
    window
        .inner_position()
        .map(|p| (p.x as f64, p.y as f64))
        .unwrap_or((0.0, 0.0))
}

#[tauri::command]
fn set_window_position(window: tauri::Window, x: f64, y: f64) {
    let _ = window.set_position(tauri::LogicalPosition::new(x, y));
}

#[tauri::command]
async fn set_main_window_position<R: Runtime>(app: tauri::AppHandle<R>, x: f64, y: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_position(tauri::LogicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn resize_window(window: tauri::Window, width: f64, height: f64) {
    let _ = window.set_size(tauri::LogicalSize::new(width, height));
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
async fn start_http_server<R: Runtime>(_app: tauri::AppHandle<R>, port: u16) -> Result<(), String> {
    http_server::start_server_background("127.0.0.1".to_string(), port);
    Ok(())
}

#[tauri::command]
async fn trigger_notification<R: Runtime>(
    _window: tauri::Window<R>,
    _title: String,
    _body: String,
) -> Result<(), String> {
    Ok(())
}

// ============================================================================
// Tray Menu
// ============================================================================

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayTranslations {
    pub show_window: String,
    pub hide_window: String,
    pub settings: String,
    pub quit: String,
    pub no_tasks: String,
    pub tasks: String,
}

lazy_static::lazy_static! {
    static ref TRAY_TRANSLATIONS: std::sync::Mutex<TrayTranslations> = std::sync::Mutex::new(TrayTranslations {
        show_window: "Show/Hide".to_string(),
        hide_window: "Show/Hide".to_string(),
        settings: "Settings".to_string(),
        quit: "Quit".to_string(),
        no_tasks: "No tasks".to_string(),
        tasks: "Tasks".to_string(),
    });
}

fn get_tray_translations_internal() -> TrayTranslations {
    TRAY_TRANSLATIONS.lock().map(|t| t.clone()).unwrap_or_else(|_| TrayTranslations {
        show_window: "Show/Hide".to_string(),
        hide_window: "Show/Hide".to_string(),
        settings: "Settings".to_string(),
        quit: "Quit".to_string(),
        no_tasks: "No tasks".to_string(),
        tasks: "Tasks".to_string(),
    })
}

#[tauri::command]
async fn update_tray_translations<R: Runtime>(
    app: tauri::AppHandle<R>,
    translations: TrayTranslations,
) -> Result<(), String> {
    if let Ok(mut trans) = TRAY_TRANSLATIONS.lock() {
        if *trans == translations {
            return Ok(());
        }
        *trans = translations;
    }
    update_tray_menu(&app);
    Ok(())
}

#[tauri::command]
async fn get_current_language(
    state: tauri::State<'_, SettingsState>,
) -> Result<String, String> {
    Ok(state.get_settings().language)
}

fn update_tray_menu<R: Runtime>(app: &tauri::AppHandle<R>) {
    let trans = get_tray_translations_internal();

    let window_toggle = MenuItem::with_id(app, "toggle-window", &trans.show_window, true, None::<&str>).ok();
    let settings_item = MenuItem::with_id(app, "settings", &trans.settings, true, None::<&str>).ok();
    let quit_item = MenuItem::with_id(app, "quit", &trans.quit, true, None::<&str>).ok();

    if let (Some(w), Some(s), Some(q)) = (window_toggle, settings_item, quit_item) {
        let items: Vec<&dyn tauri::menu::IsMenuItem<R>> = vec![&w, &s, &q];

        if let Ok(menu) = tauri::menu::Menu::with_items(app, &items) {
            if let Some(tray) = app.tray_by_id("main-tray") {
                let _ = tray.set_menu(Some(menu));
            }
        }
    }
}

// ============================================================================
// Emit Events (任务更新推送)
// ============================================================================

#[tauri::command]
async fn emit_tasks_updated<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    let tasks = http_server::get_merged_tasks();
    app.emit("tasks-updated", &tasks).map_err(|e| e.to_string())
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(main_window) = app.get_webview_window("main") {
                let _ = main_window.show();
                let _ = main_window.set_focus();
            }
            if let Some(settings_window) = app.get_webview_window("settings") {
                let _ = settings_window.show();
                let _ = settings_window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            minimize_window,
            close_window,
            set_window_always_on_top,
            set_always_on_top,
            set_auto_start,
            set_window_opacity,
            set_window_transparency,
            show_window,
            hide_window,
            get_window_position,
            set_window_position,
            set_main_window_position,
            resize_window,
            toggle_window_always_on_top,
            get_all_windows,
            open_url,
            start_http_server,
            trigger_notification,
            activate_window,
            activate_ide_window,
            get_ide_windows,
            open_settings_window,
            get_app_settings,
            update_app_settings,
            get_window_visibility,
            set_window_visibility,
            update_tray_translations,
            get_current_language,
            get_tasks,
            emit_tasks_updated,
        ])
        .setup(|app| {
            let app_handle = app.app_handle().clone();

            // 初始化设置 (JSON文件存储)
            let settings_state = SettingsState::new(&app_handle);
            let current_settings = settings_state.get_settings();
            app.manage(settings_state);

            let window = app_handle.get_webview_window("main").unwrap();

            // 恢复保存的窗口位置
            if let (Some(x), Some(y)) = (current_settings.window_x, current_settings.window_y) {
                let _ = window.set_position(tauri::LogicalPosition::new(x, y));
                info!("Restored window position: ({}, {})", x, y);
            }

            #[cfg(target_os = "macos")]
            {
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
                let _ = apply_vibrancy(
                    &window,
                    NSVisualEffectMaterial::HudWindow,
                    Some(NSVisualEffectState::Active),
                    Some(12.0),
                );
            }

            #[cfg(target_os = "windows")]
            {
                use window_vibrancy::apply_blur;
                let _ = apply_blur(&window, Some((18, 18, 18, 200)));
            }

            // 初始化 HTTP server 的屏蔽设置
            http_server::set_block_plugin_status(current_settings.block_plugin_status);

            // 启动 HTTP server
            http_server::start_server_background(
                current_settings.http_host.clone(),
                current_settings.http_port,
            );
            info!(host = %current_settings.http_host, port = %current_settings.http_port, "HTTP server started");

            // 创建托盘
            let trans = get_tray_translations_internal();
            let window_toggle_item = MenuItem::with_id(
                &app_handle,
                "toggle-window",
                &trans.hide_window,
                true,
                None::<&str>,
            )?;
            let settings_item = MenuItem::with_id(&app_handle, "settings", &trans.settings, true, None::<&str>)?;
            let quit_item = MenuItem::with_id(&app_handle, "quit", &trans.quit, true, None::<&str>)?;

            let icon_bytes = include_bytes!("../icons/tray.png");
            let img = image::load_from_memory(icon_bytes).expect("Failed to load tray icon");
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let raw_data: Vec<u8> = rgba.into_raw();
            let icon = tauri::image::Image::new_owned(raw_data, width, height);

            info!("Creating system tray with icon {}x{}", width, height);

            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .icon_as_template(false)
                .tooltip("Vibe Process Bar")
                .menu(&tauri::menu::Menu::with_items(
                    &app_handle,
                    &[&window_toggle_item, &settings_item, &quit_item],
                )?)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "toggle-window" => {
                            let settings_state = app.state::<SettingsState>();
                            let window = app.get_webview_window("main").unwrap();
                            let is_visible = window.is_visible().unwrap_or(true);

                            if is_visible {
                                let _ = window.hide();
                                let mut settings = settings_state.get_settings();
                                settings.window_visible = false;
                                let _ = settings_state.update_settings(settings);
                                let _ = app.tray_by_id("main-tray").unwrap()
                                    .set_tooltip(Some("Vibe Process Bar (Hidden)"));
                            } else {
                                let _ = window.show();
                                let mut settings = settings_state.get_settings();
                                settings.window_visible = true;
                                let _ = settings_state.update_settings(settings);
                                let _ = app.tray_by_id("main-tray").unwrap()
                                    .set_tooltip(Some("Vibe Process Bar"));
                            }
                        }
                        "settings" => {
                            let _ = open_settings_window(app.app_handle().clone());
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            info!("System tray created successfully");
            Box::leak(Box::new(tray));

            let window_clone = window.clone();
            let app_handle_clone = app.app_handle().clone();

            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let settings_state = app_handle_clone.state::<SettingsState>();
                    let mut settings = settings_state.get_settings();
                    settings.window_visible = false;
                    let _ = settings_state.update_settings(settings);
                    let _ = window_clone.hide();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
