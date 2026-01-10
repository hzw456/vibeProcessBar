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
    pub window_id: Option<String>,  // UUID 用于精确匹配窗口
    pub name: String,
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
    pub source: String,  // "plugin" or "mcp" - 上报来源
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTaskRequest {
    pub task_id: String,
    pub window_id: Option<String>,
    pub name: String,
    pub ide: String,
    pub window_title: String,
    pub project_path: Option<String>,
    pub active_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTaskRequest {
    pub task_id: String,
    pub window_id: Option<String>,
    pub name: Option<String>,
    pub ide: Option<String>,
    pub window_title: Option<String>,
    pub project_path: Option<String>,
    pub active_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTaskRequest {
    pub task_id: String,
    pub window_id: Option<String>,
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
    pub window_id: Option<String>,
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
    pub window_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetRequest {
    pub task_id: Option<String>,
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
        let tasks = state.tasks.lock().unwrap();
        let current_task_id = state.current_task_id.lock().unwrap();
        let current_task = current_task_id.as_ref()
            .and_then(|id| tasks.iter().find(|t| t.id == *id))
            .cloned();
        let resp = serde_json::json!({
            "currentTask": current_task,
            "tasks": *tasks,
            "taskCount": tasks.len()
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
                } else {
                   let task = Task {
                        id: req.task_id.clone(),
                        window_id: req.window_id.clone(),
                        name: req.name,
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
                    };
                    tasks.push(task);
                }
                *state.current_task_id.lock().unwrap() = Some(req.task_id);
                format_response(200, r#"{"status":"ok"}"#)
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
                } else {
                   let task = Task {
                        id: req.task_id.clone(),
                        window_id: req.window_id.clone(),
                        name: req.name,
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
                    };
                    tasks.push(task);
                }
                *state.current_task_id.lock().unwrap() = Some(req.task_id);
                format_response(200, r#"{"status":"ok"}"#)
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
                } else {
                    // Create new running task
                    let task = Task {
                        id: req.task_id.clone(),
                        window_id: req.window_id.clone(),
                        name: req.name,
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
                    };
                    tasks.push(task);
                }
                *state.current_task_id.lock().unwrap() = Some(req.task_id);
                format_response(200, r#"{"status":"ok"}"#)
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
        // ACTIVE/FOCUS state - window has focus, only update is_focused flag
        match serde_json::from_str::<CancelRequest>(&body) {
            Ok(req) => {
                let mut tasks = state.tasks.lock().unwrap();
                let found = tasks.iter_mut().find(|t| t.id == req.task_id);
                if let Some(task) = found {
                    task.is_focused = true;  // Only set focus, don't change status
                    format_response(200, r#"{"status":"ok"}"#)
                } else {
                    format_response(404, r#"{"error":"Task not found"}"#)
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
                "instructions": r#"Vibe Process Bar - AI任务状态追踪器。

使用方法：
1. 任务开始时：调用 update_task_status(task_id, "running")
2. 任务完成时：调用 update_task_status(task_id, "completed")  
3. 任务出错时：调用 update_task_status(task_id, "error")

task_id 格式为 "{ide}_{workspace名}"，例如 "antigravity_myproject"。
可以先调用 list_tasks 获取当前任务列表及其 task_id。

状态值：running(进行中), completed(已完成), error(出错), cancelled(已取消)"#
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
                        "description": "Get all IDE windows/tasks with their UUID, IDE name, workspace name, active file, and status",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    },
                    {
                        "name": "update_task_status",
                        "description": "Update a task's status. Valid statuses: running, completed, error, cancelled, armed, active",
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
                    let tasks = state.tasks.lock().unwrap();
                    let task_list: Vec<serde_json::Value> = tasks.iter().map(|t| {
                        serde_json::json!({
                            "id": t.id,
                            "ide": t.ide,
                            "window_title": t.window_title,
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
