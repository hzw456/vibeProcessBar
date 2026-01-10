use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::spawn;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn, error};

lazy_static::lazy_static! {
    /// Global shared state for HTTP server, accessible by tray menu
    static ref SHARED_STATE: Arc<SharedState> = Arc::new(SharedState::new());
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,
    
    pub name: String,
    #[serde(default, skip_deserializing)]
    pub display_name: String,  // 显示名称 (去掉 IDE 前缀)
    pub progress: u32,
    pub tokens: u64,
    pub status: String,  // registered, armed, running, completed
    pub is_focused: bool,  // 窗口是否有焦点
    pub ide: String,
    pub window_title: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub project_path: Option<String>,
    pub active_file: Option<String>,
    pub source: String,  // "hook", "mcp", or "plugin" - 上报来源 (priority: hook > mcp > plugin)
    #[serde(default)]
    pub last_heartbeat: u64,  // 最后心跳时间戳 (毫秒)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTaskRequest {
    pub task_id: String,
    
    pub name: String,
    pub ide: String,
    pub window_title: String,
    pub project_path: Option<String>,
    pub active_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTaskRequest {
    pub task_id: String,
    
    pub name: Option<String>,
    pub ide: Option<String>,
    pub window_title: Option<String>,
    pub project_path: Option<String>,
    pub active_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTaskRequest {
    pub task_id: String,
    
    pub name: String,
    pub ide: String,
    pub window_title: String,
    pub project_path: Option<String>,
    pub active_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgressRequest {
    pub task_id: String,
    pub progress: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenRequest {
    pub task_id: String,
    pub tokens: u64,
    pub increment: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompleteRequest {
    pub task_id: String,
    
    pub total_tokens: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorRequest {
    pub task_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelRequest {
    pub task_id: String,
    
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetRequest {
    pub task_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActiveNotificationRequest {
    pub task_id: String,
    
    pub name: String,
    pub ide: String,
    pub window_title: String,
    pub project_path: Option<String>,
    pub active_file: Option<String>,
    pub status: Option<String>,  // Extension sends this extra field
}

pub struct SharedState {
    pub tasks: Mutex<Vec<Task>>,
    pub current_task_id: Mutex<Option<String>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            tasks: Mutex::new(Vec::new()),
            current_task_id: Mutex::new(None),
        }
    }
}

/// Source priority: hook (highest) > mcp > plugin (lowest)
fn get_source_priority(source: &str) -> u8 {
    match source {
        "hook" => 3,
        "mcp" => 2,
        "plugin" => 1,
        _ => 0,
    }
}

/// Check if the new source can update the task (only if higher or equal priority)
fn can_update_source(current_source: &str, new_source: &str) -> bool {
    get_source_priority(new_source) >= get_source_priority(current_source)
}

/// Sort tasks by source priority (highest first), then by id for stability
fn sort_tasks_by_priority(tasks: &mut Vec<Task>) {
    tasks.sort_by(|a, b| {
        let priority_cmp = get_source_priority(&b.source).cmp(&get_source_priority(&a.source));
        if priority_cmp == std::cmp::Ordering::Equal {
            a.id.cmp(&b.id)
        } else {
            priority_cmp
        }
    });
}

/// Calculate display name by removing IDE prefix (e.g., "Cursor - project" -> "project")
fn get_display_name(name: &str, ide: &str) -> String {
    let prefix = format!("{} - ", ide);
    if name.starts_with(&prefix) {
        name[prefix.len()..].to_string()
    } else {
        name.to_string()
    }
}

async fn handle_connection(
    stream: &mut tokio::net::TcpStream,
    state: &Arc<SharedState>,
) -> Result<(), std::io::Error> {
    let mut buffer = [0; 8192];
    let bytes_read = stream.read(&mut buffer).await?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let body = extract_body(&request);

    debug!(method = %request.lines().next().unwrap_or(""), path = %request.lines().next().map(|l| l.split_whitespace().nth(1).unwrap_or("")).unwrap_or(""), "HTTP request received");
    let body_summary = body.chars().take(100).collect::<String>();
    debug!(body = %body_summary, "Request body");

    let response = if request.starts_with("GET /api/status") {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        const HEARTBEAT_TIMEOUT_MS: u64 = 15000;  // 15 seconds
        
        // Clean up stale tasks (no heartbeat for 15 seconds)
        {
            let mut tasks = state.tasks.lock().unwrap();
            let before_count = tasks.len();
            tasks.retain(|t| {
                // Keep tasks with recent heartbeat or that haven't been registered yet (last_heartbeat == 0 means just created)
                let age = if t.last_heartbeat > 0 { now.saturating_sub(t.last_heartbeat) } else { 0 };
                age < HEARTBEAT_TIMEOUT_MS
            });
            let removed = before_count - tasks.len();
            if removed > 0 {
                info!("Cleaned up {} stale tasks (no heartbeat for {}ms)", removed, HEARTBEAT_TIMEOUT_MS);
            }
        }
        
        let mut tasks_vec = state.tasks.lock().unwrap().clone();
        // Sort tasks by source priority (hook > mcp > plugin)
        sort_tasks_by_priority(&mut tasks_vec);
        // Calculate display_name for each task
        for task in &mut tasks_vec {
            task.display_name = get_display_name(&task.name, &task.ide);
        }
        let current_task_id = state.current_task_id.lock().unwrap();
        let current_task = current_task_id.as_ref()
            .and_then(|id| tasks_vec.iter().find(|t| t.id == *id))
            .cloned();
        let resp = serde_json::json!({
            "currentTask": current_task,
            "tasks": tasks_vec,
            "taskCount": tasks_vec.len()
        });
        format_response(200, &resp.to_string())
    } else if request.starts_with("POST /api/task/register") {
        // REGISTER state - 扩展启动时注册任务
        match serde_json::from_str::<RegisterTaskRequest>(&body) {
            Ok(req) => {
                info!(task_id = %req.task_id, name = %req.name, ide = %req.ide, active_file = ?req.active_file, "Task registered");
                let mut tasks = state.tasks.lock().unwrap();
                let existing = tasks.iter_mut().find(|t| t.id == req.task_id);
                
                if let Some(task) = existing {
                   // Always update heartbeat when receiving register notification
                   task.last_heartbeat = chrono::Utc::now().timestamp_millis() as u64;
                   
                   // Check source priority - only update if plugin has >= priority
                   if !can_update_source(&task.source, "plugin") {
                       info!(task_id = %req.task_id, current_source = %task.source, "Ignoring plugin register - lower priority than current source");
                       format_response(200, r#"{"status":"ignored","reason":"lower_priority_source"}"#)
                   } else {
                       task.name = req.name;
                       task.ide = req.ide;
                       task.window_title = req.window_title;
                       task.status = "armed".to_string();  // registered -> armed immediately
                       task.is_focused = false;
                       if let Some(path) = req.project_path {
                           task.project_path = Some(path);
                       }
                       if let Some(file) = req.active_file {
                           task.active_file = Some(file);
                       }
                       task.progress = 0;
                       task.tokens = 0;
                       task.start_time = 0;
                       task.end_time = None;
                       drop(tasks);  // Release lock before setting current_task_id
                       *state.current_task_id.lock().unwrap() = Some(req.task_id);
                       format_response(200, r#"{"status":"ok"}"#)
                   }
                } else {
                   let task = Task {
                        id: req.task_id.clone(),
                        
                        name: req.name,
                        display_name: String::new(),  // Computed on API response
                        progress: 0,
                        tokens: 0,
                        status: "armed".to_string(),  // registered -> armed immediately
                        is_focused: false,
                        ide: req.ide,
                        window_title: req.window_title,
                        start_time: 0,
                        end_time: None,
                        project_path: req.project_path,
                        active_file: req.active_file,
                        source: "plugin".to_string(),
                        last_heartbeat: chrono::Utc::now().timestamp_millis() as u64,
                    };
                    tasks.push(task);
                    drop(tasks);  // Release lock before setting current_task_id
                    *state.current_task_id.lock().unwrap() = Some(req.task_id);
                    format_response(200, r#"{"status":"ok"}"#)
                }
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/task/update") {
        // UPDATE - 更新任务信息（如活动文件变化）
        match serde_json::from_str::<UpdateTaskRequest>(&body) {
            Ok(req) => {
                debug!(task_id = %req.task_id, active_file = ?req.active_file, "Task updated");
                let mut tasks = state.tasks.lock().unwrap();
                let found = tasks.iter_mut().find(|t| t.id == req.task_id);
                if let Some(task) = found {
                    if let Some(name) = req.name {
                        task.name = name;
                    }
                    if let Some(ide) = req.ide {
                        task.ide = ide;
                    }
                    if let Some(title) = req.window_title {
                        task.window_title = title;
                    }
                    if let Some(path) = req.project_path {
                        task.project_path = Some(path);
                    }
                    if let Some(file) = req.active_file {
                        task.active_file = Some(file);
                    }
                    format_response(200, r#"{"status":"ok"}"#)
                } else {
                    format_response(404, r#"{"error":"Task not found"}"#)
                }
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/task/armed") {
        // ARMED state - 保留兼容性
        match serde_json::from_str::<StartTaskRequest>(&body) {
            Ok(req) => {
                info!(task_id = %req.task_id, name = %req.name, ide = %req.ide, "Task armed");
                let mut tasks = state.tasks.lock().unwrap();
                let existing = tasks.iter_mut().find(|t| t.id == req.task_id);
                
                if let Some(task) = existing {
                   // Always update heartbeat when receiving armed notification
                   task.last_heartbeat = chrono::Utc::now().timestamp_millis() as u64;
                   
                   // Check source priority - only update if plugin has >= priority
                   if !can_update_source(&task.source, "plugin") {
                       info!(task_id = %req.task_id, current_source = %task.source, "Ignoring plugin armed - lower priority than current source");
                       format_response(200, r#"{"status":"ignored","reason":"lower_priority_source"}"#)
                   } else {
                       // Don't change running tasks to armed - they should only become completed
                       if task.status == "running" {
                           info!(task_id = %req.task_id, "Ignoring armed request for running task");
                           // Just update metadata, don't change status
                           if let Some(file) = req.active_file {
                               task.active_file = Some(file);
                           }
                           // Don't change status, keep running
                       } else {
                           task.name = req.name;
                           task.ide = req.ide;
                           task.window_title = req.window_title;
                           task.status = "armed".to_string();
                           task.is_focused = false;  // Lost focus
                           if let Some(path) = req.project_path {
                               task.project_path = Some(path);
                           }
                           if let Some(file) = req.active_file {
                               task.active_file = Some(file);
                           }
                           // Don't reset progress/tokens/start_time - preserve them for continuing tasks
                           task.end_time = None;
                       }
                       drop(tasks);
                       *state.current_task_id.lock().unwrap() = Some(req.task_id);
                       format_response(200, r#"{"status":"ok"}"#)
                   }
                } else {
                   let task = Task {
                        id: req.task_id.clone(),
                        
                        name: req.name,
                        display_name: String::new(),  // Computed on API response
                        progress: 0,
                        tokens: 0,
                        status: "armed".to_string(),
                        is_focused: false,
                        ide: req.ide,
                        window_title: req.window_title,
                        start_time: 0,
                        end_time: None,
                        project_path: req.project_path,
                        active_file: req.active_file,
                        source: "plugin".to_string(),
                        last_heartbeat: chrono::Utc::now().timestamp_millis() as u64,
                    };
                    tasks.push(task);
                    drop(tasks);
                    *state.current_task_id.lock().unwrap() = Some(req.task_id);
                    format_response(200, r#"{"status":"ok"}"#)
                }
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/task/start") {
        match serde_json::from_str::<StartTaskRequest>(&body) {
            Ok(req) => {
                info!(task_id = %req.task_id, name = %req.name, "Task started");
                let mut tasks = state.tasks.lock().unwrap();
                // Check if task exists and is in registered/armed state
                let existing = tasks.iter_mut().find(|t| t.id == req.task_id);
                if let Some(task) = existing {
                    // Check source priority - only update if plugin has >= priority
                    if !can_update_source(&task.source, "plugin") {
                        info!(task_id = %req.task_id, current_source = %task.source, "Ignoring plugin start - lower priority than current source");
                        format_response(200, r#"{"status":"ignored","reason":"lower_priority_source"}"#)
                    } else {
                        // Update existing task to running
                        task.status = "running".to_string();
                        task.start_time = chrono::Utc::now().timestamp_millis() as u64;
                        task.name = req.name;
                        if let Some(path) = req.project_path {
                            task.project_path = Some(path);
                        }
                        if let Some(file) = req.active_file {
                            task.active_file = Some(file);
                        }
                        drop(tasks);
                        *state.current_task_id.lock().unwrap() = Some(req.task_id);
                        format_response(200, r#"{"status":"ok"}"#)
                    }
                } else {
                    // Create new running task
                    let task = Task {
                        id: req.task_id.clone(),
                        
                        name: req.name,
                        display_name: String::new(),  // Computed on API response
                        progress: 0,
                        tokens: 0,
                        status: "running".to_string(),
                        is_focused: false,
                        ide: req.ide,
                        window_title: req.window_title,
                        start_time: chrono::Utc::now().timestamp_millis() as u64,
                        end_time: None,
                        project_path: req.project_path,
                        active_file: req.active_file,
                        source: "plugin".to_string(),
                        last_heartbeat: chrono::Utc::now().timestamp_millis() as u64,
                    };
                    tasks.push(task);
                    drop(tasks);
                    *state.current_task_id.lock().unwrap() = Some(req.task_id);
                    format_response(200, r#"{"status":"ok"}"#)
                }
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/task/progress") {
        match serde_json::from_str::<ProgressRequest>(&body) {
            Ok(req) => {
                debug!(task_id = %req.task_id, progress = %req.progress, "Task progress updated");
                let mut tasks = state.tasks.lock().unwrap();
                let found = tasks.iter_mut().find(|t| t.id == req.task_id);
                if let Some(task) = found {
                    task.progress = req.progress.clamp(0, 100);
                    format_response(200, r#"{"status":"ok"}"#)
                } else {
                    format_response(404, r#"{"error":"Task not found"}"#)
                }
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/task/token") {
        match serde_json::from_str::<TokenRequest>(&body) {
            Ok(req) => {
                debug!(task_id = %req.task_id, tokens = %req.tokens, increment = %req.increment.unwrap_or(false), "Token count updated");
                let mut tasks = state.tasks.lock().unwrap();
                let found = tasks.iter_mut().find(|t| t.id == req.task_id);
                if let Some(task) = found {
                    let increment = req.increment.unwrap_or(false);
                    if increment {
                        task.tokens += req.tokens;
                    } else {
                        task.tokens = req.tokens;
                    }
                    format_response(200, r#"{"status":"ok"}"#)
                } else {
                    format_response(404, r#"{"error":"Task not found"}"#)
                }
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/task/complete") {
        match serde_json::from_str::<CompleteRequest>(&body) {
            Ok(req) => {
                info!(task_id = %req.task_id, "Task completed");
                let mut tasks = state.tasks.lock().unwrap();
                let found = tasks.iter_mut().find(|t| t.id == req.task_id);
                if let Some(task) = found {
                    task.status = "completed".to_string();
                    task.progress = 100;
                    task.end_time = Some(chrono::Utc::now().timestamp_millis() as u64);
                    if let Some(tokens) = req.total_tokens {
                        task.tokens = tokens;
                    }
                    format_response(200, r#"{"status":"ok"}"#)
                } else {
                    format_response(404, r#"{"error":"Task not found"}"#)
                }
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/task/error") {
        match serde_json::from_str::<ErrorRequest>(&body) {
            Ok(req) => {
                warn!(task_id = %req.task_id, message = %req.message, "Task error");
                let mut tasks = state.tasks.lock().unwrap();
                let found = tasks.iter_mut().find(|t| t.id == req.task_id);
                if let Some(task) = found {
                    task.status = "error".to_string();
                    format_response(200, r#"{"status":"ok"}"#)
                } else {
                    format_response(404, r#"{"error":"Task not found"}"#)
                }
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/task/cancel") {
        match serde_json::from_str::<CancelRequest>(&body) {
            Ok(req) => {
                info!(task_id = %req.task_id, "Task cancelled");
                let mut tasks = state.tasks.lock().unwrap();
                tasks.retain(|t| t.id != req.task_id);
                let mut current_id = state.current_task_id.lock().unwrap();
                if *current_id == Some(req.task_id.clone()) {
                    *current_id = None;
                }
                format_response(200, r#"{"status":"ok"}"#)
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/task/active") {
        // ACTIVE/FOCUS state - window has focus
        // Use ActiveNotificationRequest which includes the optional status field the extension sends
        match serde_json::from_str::<ActiveNotificationRequest>(&body) {
            Ok(req) => {
                let mut tasks = state.tasks.lock().unwrap();
                let existing = tasks.iter_mut().find(|t| t.id == req.task_id);
                
                if let Some(task) = existing {
                    // Always update heartbeat and is_focused when receiving active heartbeat
                    task.last_heartbeat = chrono::Utc::now().timestamp_millis() as u64;
                    task.is_focused = true;
                    
                    // Auto-transition: completed -> armed when window gains focus
                    // This should happen regardless of source priority since it's user action
                    if task.status == "completed" {
                        info!(task_id = %req.task_id, "Auto-transitioning completed task to armed (window focused)");
                        task.status = "armed".to_string();
                        task.source = "plugin".to_string(); // Reset source after user interaction
                        task.progress = 0;
                        task.start_time = 0;
                        task.end_time = None;
                    }
                    
                    // Check source priority - only update metadata if plugin has >= priority
                    // But is_focused and completed->armed transition always happen
                    if can_update_source(&task.source, "plugin") {
                        // Update metadata as well since we have it
                        task.name = req.name;
                        task.ide = req.ide;
                        task.window_title = req.window_title;
                        if let Some(path) = req.project_path {
                            task.project_path = Some(path);
                        }
                        if let Some(file) = req.active_file {
                            task.active_file = Some(file);
                        }
                    }
                    
                    // Log active heartbeat for debugging visibility
                    info!(task_id = %req.task_id, status = %task.status, "Task active heartbeat processed");
                    drop(tasks);
                    *state.current_task_id.lock().unwrap() = Some(req.task_id);
                    format_response(200, r#"{"status":"ok"}"#)
                } else {
                    // Task not found - register it automatically
                    info!(task_id = %req.task_id, name = %req.name, "Task auto-registered via active heartbeat");
                    let task = Task {
                        id: req.task_id.clone(),
                        
                        name: req.name,
                        display_name: String::new(),  // Computed on API response
                        progress: 0,
                        tokens: 0,
                        status: "armed".to_string(), // Default to armed (monitoring)
                        is_focused: true,            // It is currently focused
                        ide: req.ide,
                        window_title: req.window_title,
                        start_time: 0,
                        end_time: None,
                        project_path: req.project_path,
                        active_file: req.active_file,
                        source: "plugin".to_string(),
                        last_heartbeat: chrono::Utc::now().timestamp_millis() as u64,
                    };
                    tasks.push(task);
                    drop(tasks);
                    *state.current_task_id.lock().unwrap() = Some(req.task_id);
                    format_response(200, r#"{"status":"ok"}"#)
                }
            }
            Err(e) => format_response(400, &format!(r#"{{"error":"Invalid request: {}"}}"#, e))
        }
    } else if request.starts_with("POST /api/reset") {
        let mut tasks = state.tasks.lock().unwrap();
        *tasks = Vec::new();
        *state.current_task_id.lock().unwrap() = None;
        format_response(200, r#"{"status":"ok"}"#)
    } else if request.starts_with("OPTIONS") {
        // Handle CORS preflight
        format_cors_response()
    } else if request.starts_with("POST /mcp") {
        // MCP JSON-RPC handler
        handle_mcp_request(&body, state)
    } else {
        format_response(404, r#"{"error":"Not found"}"#)
    };

    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

fn format_response(status: u16, body: &str) -> String {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Unknown",
    };
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\n\r\n{}",
        status, status_text, body.len(), body
    )
}

fn format_cors_response() -> String {
    "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: 0\r\n\r\n".to_string()
}

fn extract_body(request: &str) -> String {
    if let Some(pos) = request.find("\r\n\r\n") {
        request[pos + 4..].trim().to_string()
    } else {
        String::new()
    }
}

/// MCP JSON-RPC request structure
#[derive(Deserialize, Debug)]
struct McpRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

/// Handle MCP JSON-RPC requests
fn handle_mcp_request(body: &str, state: &Arc<SharedState>) -> String {
    let req: McpRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            return format_response(400, &serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32700, "message": format!("Parse error: {}", e)},
                "id": null
            }).to_string());
        }
    };

    let result = match req.method.as_str() {
        "initialize" => {
            // MCP initialize response
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "vibe-process-bar",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": r#"Vibe Process Bar - AI Task Status Tracker.

Usage:
1. Call `list_tasks` first to get the current list of tasks.
2. Find the target task by matching `ide`, `project_path` (or `window_title`), and `active_file`.
   - `task_id` format is usually "{ide}_{project_name}".
3. Updates are prioritized by source: hook > mcp > plugin.
   - You are the "mcp" source. You can override "plugin" updates, but "hook" updates will override yours.

To update status:
- Call `update_task_status(task_id, status)`

Status values:
- `armed`: Task is registered and monitoring (e.g. window focused).
- `running`: Task is actively processing (AI generating).
- `completed`: Task finished successfully.
- `error`: Task failed.
- `cancelled`: Task was cancelled.
- `active`: Window has focus (usually sent by plugin).
- `registered`: Initial state."#
            })
        }
        "notifications/initialized" => {
            // Client initialized notification - no response needed for notifications
            return format_response(200, &serde_json::json!({
                "jsonrpc": "2.0",
                "result": {},
                "id": req.id
            }).to_string());
        }
        "tools/list" => {
            // Return available tools
            serde_json::json!({
                "tools": [
                    {
                        "name": "list_tasks",
                        "description": "Get all IDE windows/tasks with their UUID, IDE name, project path, active file, and status",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    },
                    {
                        "name": "update_task_status",
                        "description": "Update a task's status. Priority: hook > mcp > plugin. Valid statuses: running, completed, error, cancelled, armed, active, registered",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": {
                                    "type": "string",
                                    "description": "The task ID to update"
                                },
                                "status": {
                                    "type": "string",
                                    "description": "New status: running, completed, error, cancelled, armed, or active"
                                }
                            },
                            "required": ["task_id", "status"]
                        }
                    }
                ]
            })
        }
        "tools/call" => {
            // Call a tool
            let params = req.params.unwrap_or(serde_json::json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
            
            match tool_name {
                "list_tasks" => {
                    let mut tasks_vec = state.tasks.lock().unwrap().clone();
                    // Sort by source priority (hook > mcp > plugin)
                    sort_tasks_by_priority(&mut tasks_vec);
                    let task_list: Vec<serde_json::Value> = tasks_vec.iter().map(|t| {
                        serde_json::json!({
                            "id": t.id,
                            "ide": t.ide,
                            "window_title": t.window_title,
                            "project_path": t.project_path,
                            "active_file": t.active_file,
                            "status": t.status,
                            "progress": t.progress,
                            "tokens": t.tokens,
                            "source": t.source
                        })
                    }).collect();
                    
                    info!("MCP list_tasks: returning {} tasks", task_list.len());
                    serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&task_list).unwrap_or("[]".to_string())
                        }]
                    })
                }
                "update_task_status" => {
                    let task_id = arguments.get("task_id").and_then(|v| v.as_str()).unwrap_or("");
                    let status = arguments.get("status").and_then(|v| v.as_str()).unwrap_or("");
                    
                    let valid_statuses = ["running", "completed", "error", "cancelled", "armed", "active", "registered"];
                    if !valid_statuses.contains(&status) {
                        return format_response(200, &serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {"code": -32602, "message": format!("Invalid status '{}'. Valid: {:?}", status, valid_statuses)},
                            "id": req.id
                        }).to_string());
                    }
                    
                    let mut tasks = state.tasks.lock().unwrap();
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                        // Check source priority - MCP can only update if >= current priority
                        if !can_update_source(&task.source, "mcp") {
                            info!("MCP update_task_status: {} - ignoring, current source {} has higher priority", task_id, task.source);
                            return format_response(200, &serde_json::json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "content": [{
                                        "type": "text",
                                        "text": format!("Ignored: task {} has higher priority source '{}'", task_id, task.source)
                                    }]
                                },
                                "id": req.id
                            }).to_string());
                        }
                        let old_status = task.status.clone();
                        task.status = status.to_string();
                        task.source = "mcp".to_string();  // 标记为MCP上报
                        
                        if status == "running" && task.start_time == 0 {
                            task.start_time = chrono::Utc::now().timestamp_millis() as u64;
                        } else if status == "completed" || status == "error" || status == "cancelled" {
                            task.end_time = Some(chrono::Utc::now().timestamp_millis() as u64);
                            if status == "completed" {
                                task.progress = 100;
                            }
                        }
                        
                        info!("MCP update_task_status: {} {} -> {}", task_id, old_status, status);
                        serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Task {} status updated: {} -> {}", task_id, old_status, status)
                            }]
                        })
                    } else {
                        return format_response(200, &serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {"code": -32602, "message": format!("Task not found: {}", task_id)},
                            "id": req.id
                        }).to_string());
                    }
                }
                _ => {
                    return format_response(200, &serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32601, "message": format!("Unknown tool: {}", tool_name)},
                        "id": req.id
                    }).to_string());
                }
            }
        }
        _ => {
            return format_response(200, &serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32601, "message": format!("Method not found: {}", req.method)},
                "id": req.id
            }).to_string());
        }
    };

    format_response(200, &serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": req.id
    }).to_string())
}

pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = SHARED_STATE.clone();
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    info!(port = %port, "HTTP server listening on 127.0.0.1:{}", port);

    loop {
        let (mut stream, _) = listener.accept().await?;
        let state = state.clone();
        spawn(async move {
            if let Err(e) = handle_connection(&mut stream, &state).await {
                error!(error = %e, "Connection error");
            }
        });
    }
}

pub fn start_server_background(port: u16) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            if let Err(e) = start_server(port).await {
                error!(port = %port, error = %e, "HTTP server error");
            }
        });
    });
}

/// Get the global shared state for use by tray menu
pub fn get_state() -> Arc<SharedState> {
    SHARED_STATE.clone()
}
