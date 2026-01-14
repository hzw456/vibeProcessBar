use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::Runtime;
use tracing::info;

/// IDE bundle identifiers for window scanning
pub const IDE_BUNDLES: &[(&str, &str, &str)] = &[
    ("com.microsoft.VSCode", "vscode", "Visual Studio Code"),
    ("com.todesktop.230313mzl4w4u92", "cursor", "Cursor"),
    ("dev.kiro.desktop", "kiro", "Kiro"),
    ("com.google.antigravity", "antigravity", "Antigravity"),
    ("com.codeium.windsurf", "windsurf", "Windsurf"),
    ("com.trae.app", "trae", "Trae"),
    ("com.tencent.codebuddycn", "codebuddycn", "CodeBuddy CN"),
    ("com.tencent.codebuddy", "codebuddy", "CodeBuddy"),

];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdeWindow {
    pub bundle_id: String,
    pub ide: String,
    pub app_name: String,
    pub window_title: String,
    pub window_index: i32,
    pub pid: String,
}

/// Scan all IDE windows using AppleScript
#[cfg(target_os = "macos")]
pub fn scan_ide_windows() -> Vec<IdeWindow> {
    let mut all_windows = Vec::new();

    // AppleScript to get all Electron and Cursor processes with their windows
    let script = r#"
        set output to ""
        tell application "System Events"
            repeat with p in (every application process whose name is "Electron" or name is "Cursor")
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

    let output = Command::new("osascript").args(&["-e", script]).output();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);

            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.splitn(3, ":::").collect();
                if parts.len() != 3 {
                    continue;
                }

                let app_path = parts[0];
                let pid = parts[1];
                let win_names_str = parts[2];

                // Match IDE from app path using IDE_BUNDLES
                let ide_info = IDE_BUNDLES.iter().find(|(_, _, app_name)| {
                    app_path.contains(app_name)
                });

                if let Some((bundle_id, ide, app_name)) = ide_info {
                    let win_names: Vec<&str> = win_names_str.split("|||").collect();
                    for (idx, win_title) in win_names.iter().enumerate() {
                        let win_title = win_title.trim();
                        if !win_title.is_empty() {
                            all_windows.push(IdeWindow {
                                bundle_id: bundle_id.to_string(),
                                ide: ide.to_string(),
                                app_name: app_name.to_string(),
                                window_title: win_title.to_string(),
                                window_index: (idx + 1) as i32,
                                pid: pid.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    info!("Scanned {} IDE windows", all_windows.len());
    all_windows
}

#[cfg(target_os = "windows")]
pub fn scan_ide_windows() -> Vec<IdeWindow> {
    use std::process::Command;
    
    let mut all_windows = Vec::new();
    
    // Use PowerShell to enumerate windows
    let script = r#"
        Get-Process | Where-Object {$_.MainWindowTitle -ne ""} | 
        Where-Object {$_.ProcessName -match "Code|Cursor|Kiro|Antigravity|Windsurf|Trae|CodeBuddy"} |
        ForEach-Object {
            "$($_.ProcessName)|$($_.Id)|$($_.MainWindowTitle)"
        }
    "#;
    
    let output = Command::new("powershell")
        .args(&["-NoProfile", "-Command", script])
        .output();
    
    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() != 3 {
                    continue;
                }
                
                let process_name = parts[0].to_lowercase();
                let pid = parts[1];
                let window_title = parts[2];
                
                // Match IDE from process name
                let ide_info = IDE_BUNDLES.iter().find(|(_, ide, _)| {
                    process_name.contains(&ide.to_lowercase())
                });
                
                if let Some((bundle_id, ide, app_name)) = ide_info {
                    all_windows.push(IdeWindow {
                        bundle_id: bundle_id.to_string(),
                        ide: ide.to_string(),
                        app_name: app_name.to_string(),
                        window_title: window_title.to_string(),
                        window_index: 1,
                        pid: pid.to_string(),
                    });
                }
            }
        }
    }
    
    info!("Scanned {} IDE windows on Windows", all_windows.len());
    all_windows
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn scan_ide_windows() -> Vec<IdeWindow> {
    Vec::new()
}

/// Match criteria for finding the best window
#[derive(Debug)]
pub struct MatchCriteria {
    pub ide: Option<String>,
    pub workspace: Option<String>,
    pub file: Option<String>,
}

/// Find the best matching window based on criteria
/// Priority: ide > workspace > file
/// 当同一IDE有多个窗口时，优先匹配workspace，然后file
pub fn find_best_match<'a>(windows: &'a [IdeWindow], criteria: &MatchCriteria) -> Option<&'a IdeWindow> {
    let mut candidates: Vec<(&IdeWindow, u32)> = Vec::new();

    info!("=== Window Matching Debug ===");
    info!("Criteria: ide={:?}, workspace={:?}, file={:?}", criteria.ide, criteria.workspace, criteria.file);
    info!("Available windows: {}", windows.len());

    for window in windows {
        let mut score = 0u32;
        let mut match_reasons: Vec<String> = Vec::new();

        info!("  Checking window: ide={}, title=\"{}\"", window.ide, window.window_title);

        // IDE match (required if specified)
        if let Some(ref ide) = criteria.ide {
            if window.ide.to_lowercase() == ide.to_lowercase() {
                score += 100;
                match_reasons.push(format!("ide_match(+100)"));
            } else {
                info!("    -> SKIP: IDE mismatch ({} != {})", window.ide, ide);
                continue; // IDE must match if specified
            }
        }

        // Workspace/window_title match (high priority)
        if let Some(ref workspace) = criteria.workspace {
            info!("    Comparing workspace \"{}\" with window_title \"{}\"", workspace, window.window_title);
            
            // 精确匹配窗口标题
            if window.window_title == *workspace {
                score += 80;
                match_reasons.push(format!("exact_title_match(+80)"));
            }
            // 窗口标题包含workspace
            else if window.window_title.contains(workspace) {
                score += 50;
                match_reasons.push(format!("title_contains_workspace(+50)"));
            }
            // workspace包含窗口标题
            else if workspace.contains(&window.window_title) {
                score += 45;
                match_reasons.push(format!("workspace_contains_title(+45)"));
            }
            // workspace包含在窗口标题的某部分（处理 "file — Project" 格式）
            else {
                let parts: Vec<&str> = window.window_title.split(" — ").collect();
                let mut found = false;
                for part in &parts {
                    let part = part.trim();
                    if part.contains(workspace) {
                        score += 40;
                        match_reasons.push(format!("part_contains_workspace(+40): \"{}\"", part));
                        found = true;
                        break;
                    }
                    if workspace.contains(part) && part.len() > 3 {
                        score += 35;
                        match_reasons.push(format!("workspace_contains_part(+35): \"{}\"", part));
                        found = true;
                        break;
                    }
                }
                if !found {
                    info!("    -> No workspace match found in parts: {:?}", parts);
                }
            }
        }

        // File match (lower priority)
        if let Some(ref file) = criteria.file {
            if window.window_title.contains(file) {
                score += 20;
                match_reasons.push(format!("file_match(+20)"));
            }
        }

        info!("    -> Score: {}, Reasons: {:?}", score, match_reasons);

        // 只有IDE匹配时也加入候选（score >= 100）
        if score >= 100 {
            candidates.push((window, score));
        }
    }

    // Sort by score descending, then by window_index for stability
    candidates.sort_by(|a, b| {
        let score_cmp = b.1.cmp(&a.1);
        if score_cmp == std::cmp::Ordering::Equal {
            a.0.window_index.cmp(&b.0.window_index)
        } else {
            score_cmp
        }
    });
    
    info!("=== Candidates (sorted) ===");
    for (i, (window, score)) in candidates.iter().enumerate() {
        info!("  {}: \"{}\" (score: {})", i + 1, window.window_title, score);
    }
    
    if let Some((window, score)) = candidates.first() {
        info!("=== Best match: \"{}\" (score: {}) ===", window.window_title, score);
    } else {
        info!("=== No match found ===");
    }
    
    candidates.first().map(|(w, _)| *w)
}

/// Activate a specific IDE window
#[cfg(target_os = "macos")]
pub fn activate_ide_window(window: &IdeWindow) -> Result<(), String> {
    let script = format!(
        r#"
        tell application "System Events"
            set foundWindow to false
            repeat with p in (every application process whose unix id is {})
                try
                    repeat with w in (every window of p)
                        if title of w contains "{}" then
                            set frontmost of p to true
                            perform action "AXRaise" of w
                            set foundWindow to true
                            exit repeat
                        end if
                    end repeat
                    if foundWindow then exit repeat
                end try
            end repeat
        end tell
        if not foundWindow then
            tell application "{}" to activate
        end if
        "#,
        window.pid,
        window.window_title.replace("\"", "\\\""),
        window.app_name
    );

    let output = Command::new("osascript")
        .args(&["-e", &script])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        info!("AppleScript failed: {}", stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

/// Activate IDE by name (fallback when no specific window)
#[cfg(target_os = "macos")]
pub fn activate_ide_by_name(ide: &str) -> Result<(), String> {
    let app_name = IDE_BUNDLES
        .iter()
        .find(|(_, id, _)| id.to_lowercase() == ide.to_lowercase())
        .map(|(_, _, name)| *name)
        .unwrap_or(ide);

    let script = format!(r#"tell application "{}" to activate"#, app_name);

    let output = Command::new("osascript")
        .args(&["-e", &script])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn activate_ide_window(window: &IdeWindow) -> Result<(), String> {
    use std::process::Command;
    
    // Use PowerShell to activate window by PID and title
    let script = format!(
        r#"
        $proc = Get-Process -Id {} -ErrorAction SilentlyContinue
        if ($proc) {{
            Add-Type @"
                using System;
                using System.Runtime.InteropServices;
                public class Win32 {{
                    [DllImport("user32.dll")]
                    public static extern bool SetForegroundWindow(IntPtr hWnd);
                    [DllImport("user32.dll")]
                    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
                }}
"@
            $hwnd = $proc.MainWindowHandle
            [Win32]::ShowWindow($hwnd, 9) # SW_RESTORE
            [Win32]::SetForegroundWindow($hwnd)
        }}
        "#,
        window.pid
    );
    
    let output = Command::new("powershell")
        .args(&["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| e.to_string())?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn activate_ide_by_name(ide: &str) -> Result<(), String> {
    use std::process::Command;
    
    let app_name = IDE_BUNDLES
        .iter()
        .find(|(_, id, _)| id.to_lowercase() == ide.to_lowercase())
        .map(|(_, _, name)| *name)
        .unwrap_or(ide);
    
    // Try to start the application if not running
    let _ = Command::new("cmd")
        .args(&["/C", "start", "", app_name])
        .spawn();
    
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn activate_ide_window(_window: &IdeWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn activate_ide_by_name(_ide: &str) -> Result<(), String> {
    Ok(())
}

/// Main function to activate IDE window with matching
pub fn activate_ide(
    ide: &str,
    workspace: Option<&str>,
    _project_path: Option<&str>,
    active_file: Option<&str>,
) -> Result<(), String> {
    info!(
        "activate_ide: ide={}, workspace={:?}, active_file={:?}",
        ide, workspace, active_file
    );

    #[cfg(target_os = "macos")]
    {
        // Scan current windows
        let windows = scan_ide_windows();

        // Build match criteria
        let criteria = MatchCriteria {
            ide: Some(ide.to_string()),
            workspace: workspace.map(|s| s.to_string()),
            file: active_file.map(|s| s.to_string()),
        };

        // Find best match
        if let Some(window) = find_best_match(&windows, &criteria) {
            info!("Found matching window: {:?}", window);
            activate_ide_window(window)
        } else {
            info!("No matching window found, activating IDE by name");
            activate_ide_by_name(ide)
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Scan current windows
        let windows = scan_ide_windows();

        // Build match criteria
        let criteria = MatchCriteria {
            ide: Some(ide.to_string()),
            workspace: workspace.map(|s| s.to_string()),
            file: active_file.map(|s| s.to_string()),
        };

        // Find best match
        if let Some(window) = find_best_match(&windows, &criteria) {
            info!("Found matching window: {:?}", window);
            activate_ide_window(window)
        } else {
            info!("No matching window found, activating IDE by name");
            activate_ide_by_name(ide)
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Ok(())
    }
}

// Legacy function for backward compatibility
pub async fn activate_window<R: Runtime>(
    _window: tauri::Window<R>,
    window_id: String,
) -> Result<(), String> {
    activate_ide(&window_id, None, None, None)
}
