use std::process::Command;
use tauri::Runtime;

pub async fn activate_window<R: Runtime>(_window: tauri::Window<R>, window_id: String) -> Result<(), String> {
    activate_vscode_window(&window_id)
}

fn activate_vscode_window(window_id: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        activate_macOS(window_id)
    }

    #[cfg(target_os = "windows")]
    {
        activate_windows(window_id)
    }

    #[cfg(target_os = "linux")]
    {
        activate_linux(window_id)
    }
}

#[cfg(target_os = "macos")]
fn activate_macOS(_window_id: &str) -> Result<(), String> {
    let script = r#"
        tell application "Visual Studio Code"
            activate
            delay 0.3
            tell application "System Events"
                keystroke "`" using command down
            end tell
        end tell
    "#;

    let output = Command::new("osascript")
        .args(&["-e", script])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn activate_windows(_window_id: &str) -> Result<(), String> {
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('^{}`');"#,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn activate_linux(_window_id: &str) -> Result<(), String> {
    let output = Command::new("wmctrl")
        .args(&["-a", "Visual Studio Code"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let fallback = Command::new("xdotool")
            .args(&["search", "--name", "Visual Studio Code", "windowactivate"])
            .output()
            .map_err(|e| e.to_string())?;

        if !fallback.status.success() {
            return Err(String::from_utf8_lossy(&fallback.stderr).to_string());
        }
    }

    Ok(())
}
