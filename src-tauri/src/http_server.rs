use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info, warn};

lazy_static::lazy_static! {
    /// Global shared state for HTTP server, accessible by tray menu
    static ref SHARED_STATE: Arc<SharedState> = Arc::new(SharedState::new());
}

// ============================================================================
// Task 数据结构
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,

    // === Window Info (register/active/heartbeat 接口上报) ===
    /// 任务名称 (format: "IDE - WindowTitle")
    pub name: String,
    /// 显示名称 (去掉 IDE 前缀，用于 UI 展示)
    #[serde(default, skip_deserializing)]
    pub display_name: String,
    /// 窗口是否有焦点
    pub is_focused: bool,
    /// IDE 类型 (cursor, vscode, antigravity, etc.)
    pub ide: String,
    /// 窗口标题
    pub window_title: String,
    /// 项目路径
    pub project_path: Option<String>,
    /// 当前活动文件
    pub active_file: Option<String>,

    // === Process State (update_state 接口上报) ===
    /// 进度 (0-100)
    pub progress: u32,
    /// 状态: armed, running, completed, error, cancelled
    pub status: String,
    /// 上报来源: hook, mcp, plugin (priority: hook > mcp > plugin)
    pub source: String,

    // === Process Info (由 Rust 后端自动记录) ===
    /// 任务开始时间 (status 变为 running 时自动设置)
    pub start_time: u64,
    /// 任务结束时间 (status 变为 completed/error/cancelled 时自动设置)
    pub end_time: Option<u64>,

    // === Internal ===
    /// 最后心跳时间戳 (毫秒)
    #[serde(default)]
    pub last_heartbeat: u64,
}

// ============================================================================
// Request 数据结构
// ============================================================================

/// Window Info 请求 (report)
/// 统一的窗口信息上报接口，用于启动、焦点变化、心跳保活
#[derive(Serialize, Deserialize, Debug)]
pub struct ReportRequest {
    pub task_id: String,
    pub name: String,
    pub ide: String,
    pub window_title: String,
    /// 窗口是否有焦点 (true=前台, false=后台)
    #[serde(default)]
    pub is_focused: bool,
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub active_file: Option<String>,
}

/// Process State 请求 (update_state)
/// 用于上报进程状态变化
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateStateRequest {
    pub task_id: String,
    /// 状态: running, completed, error, cancelled, armed
    #[serde(default)]
    pub status: Option<String>,
    /// 进度 (0-100)
    #[serde(default)]
    pub progress: Option<u32>,
}

/// Reset 请求
#[derive(Serialize, Deserialize, Debug)]
pub struct ResetRequest {
    #[serde(default)]
    pub task_id: Option<String>,
}

/// Delete Task 请求
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteTaskRequest {
    pub task_id: String,
}

// ============================================================================
// Response 数据结构
// ============================================================================

#[derive(Serialize)]
struct ApiResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ApiResponse {
    fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            reason: None,
            error: None,
        }
    }

    fn ignored(reason: &str) -> Self {
        Self {
            status: "ignored".to_string(),
            reason: Some(reason.to_string()),
            error: None,
        }
    }

    fn error(msg: &str) -> Self {
        Self {
            status: "error".to_string(),
            reason: None,
            error: Some(msg.to_string()),
        }
    }
}

#[derive(Serialize)]
struct StatusResponse {
    #[serde(rename = "currentTask")]
    current_task: Option<Task>,
    tasks: Vec<Task>,
    #[serde(rename = "taskCount")]
    task_count: usize,
}

// ============================================================================
// Shared State
// ============================================================================

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

// ============================================================================
// Helper Functions
// ============================================================================

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

/// Get current timestamp in milliseconds
fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}

// ============================================================================
// API Handlers
// ============================================================================

/// GET /api/status - 获取所有任务状态
async fn get_status(State(state): State<Arc<SharedState>>) -> Json<StatusResponse> {
    const HEARTBEAT_TIMEOUT_MS: u64 = 5000; // 5 seconds - faster cleanup when window closes
    let now = now_millis();

    // Clean up stale tasks (no heartbeat for 15 seconds)
    {
        let mut tasks = state.tasks.lock().unwrap();
        let before_count = tasks.len();
        tasks.retain(|t| {
            let age = if t.last_heartbeat > 0 {
                now.saturating_sub(t.last_heartbeat)
            } else {
                0
            };
            age < HEARTBEAT_TIMEOUT_MS
        });
        let removed = before_count - tasks.len();
        if removed > 0 {
            info!(
                "Cleaned up {} stale tasks (no heartbeat for {}ms)",
                removed, HEARTBEAT_TIMEOUT_MS
            );
        }
    }

    let mut tasks_vec = state.tasks.lock().unwrap().clone();
    sort_tasks_by_priority(&mut tasks_vec);

    // Calculate display_name for each task
    for task in &mut tasks_vec {
        task.display_name = get_display_name(&task.name, &task.ide);
    }

    let current_task_id = state.current_task_id.lock().unwrap();
    let current_task = current_task_id
        .as_ref()
        .and_then(|id| tasks_vec.iter().find(|t| t.id == *id))
        .cloned();

    Json(StatusResponse {
        current_task,
        task_count: tasks_vec.len(),
        tasks: tasks_vec,
    })
}

/// POST /api/task/report - 统一的窗口信息上报接口
/// 合并了 register/active/heartbeat 三个接口
/// - is_focused=true: 窗口获得焦点
/// - is_focused=false: 窗口在后台
async fn report_task(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<ReportRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let mut tasks = state.tasks.lock().unwrap();
    let existing = tasks.iter_mut().find(|t| t.id == req.task_id);

    if let Some(task) = existing {
        // 更新心跳时间
        task.last_heartbeat = now_millis();

        // 更新焦点状态
        task.is_focused = req.is_focused;

        // 检查来源优先级
        if !can_update_source(&task.source, "plugin") {
            debug!(task_id = %req.task_id, "Report ignored - lower priority source");
            return (
                StatusCode::OK,
                Json(ApiResponse::ignored("lower_priority_source")),
            );
        }

        // 窗口获得焦点时，completed -> armed 自动转换
        if req.is_focused && task.status == "completed" {
            info!(task_id = %req.task_id, "Auto-transitioning completed task to armed (window focused)");
            task.status = "armed".to_string();
            // 注意：不修改 source，source 只由 update_state 控制
            task.progress = 0;
            task.start_time = 0;
            task.end_time = None;
        }

        // 更新窗口信息
        task.name = req.name;
        task.ide = req.ide;
        task.window_title = req.window_title;
        if let Some(path) = req.project_path {
            task.project_path = Some(path);
        }
        if let Some(file) = req.active_file {
            task.active_file = Some(file);
        }

        debug!(task_id = %req.task_id, is_focused = %req.is_focused, status = %task.status, "Task report processed");
        drop(tasks);
        *state.current_task_id.lock().unwrap() = Some(req.task_id);
    } else {
        // 任务不存在，自动注册
        info!(task_id = %req.task_id, name = %req.name, ide = %req.ide, "Task auto-registered");
        let task = Task {
            id: req.task_id.clone(),
            name: req.name,
            display_name: String::new(),
            progress: 0,
            status: "armed".to_string(),
            is_focused: req.is_focused,
            ide: req.ide,
            window_title: req.window_title,
            start_time: 0,
            end_time: None,
            project_path: req.project_path,
            active_file: req.active_file,
            source: "plugin".to_string(),
            last_heartbeat: now_millis(),
        };
        tasks.push(task);
        drop(tasks);
        *state.current_task_id.lock().unwrap() = Some(req.task_id);
    }

    (StatusCode::OK, Json(ApiResponse::ok()))
}

/// POST /api/task/update_state - 更新进程状态
async fn update_state(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<UpdateStateRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let mut tasks = state.tasks.lock().unwrap();
    let found = tasks.iter_mut().find(|t| t.id == req.task_id);

    if let Some(task) = found {
        // Check source priority for 'plugin' source
        if !can_update_source(&task.source, "plugin") {
            info!(task_id = %req.task_id, current_source = %task.source, "Ignoring update_state - lower priority");
            return (
                StatusCode::OK,
                Json(ApiResponse::ignored("lower_priority_source")),
            );
        }

        // Update progress if provided
        if let Some(progress) = req.progress {
            task.progress = progress.clamp(0, 100);
            debug!(task_id = %req.task_id, progress = %task.progress, "Progress updated");
        }

        // Update status if provided, and auto-record timestamps
        if let Some(ref status) = req.status {
            let old_status = task.status.clone();

            // Validate status
            let valid_statuses = ["armed", "running", "completed", "error", "cancelled"];
            if !valid_statuses.contains(&status.as_str()) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(&format!(
                        "Invalid status '{}'. Valid: {:?}",
                        status, valid_statuses
                    ))),
                );
            }

            task.status = status.clone();

            // Auto-record start_time when transitioning to running
            if status == "running" && task.start_time == 0 {
                task.start_time = now_millis();
                info!(task_id = %req.task_id, "Task started (auto-recorded start_time)");
            }

            // Auto-record end_time when transitioning to terminal states
            if status == "completed" || status == "error" || status == "cancelled" {
                task.end_time = Some(now_millis());
                if status == "completed" {
                    task.progress = 100;
                }
                info!(task_id = %req.task_id, old_status = %old_status, new_status = %status, "Task ended (auto-recorded end_time)");
            }
        }

        (StatusCode::OK, Json(ApiResponse::ok()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Task not found")),
        )
    }
}

/// POST /api/reset - 重置所有任务
async fn reset_tasks(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<ResetRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let mut tasks = state.tasks.lock().unwrap();

    if let Some(task_id) = req.task_id {
        // Reset specific task
        tasks.retain(|t| t.id != task_id);
        let mut current_id = state.current_task_id.lock().unwrap();
        if *current_id == Some(task_id.clone()) {
            *current_id = None;
        }
        info!(task_id = %task_id, "Task removed");
    } else {
        // Reset all tasks
        *tasks = Vec::new();
        *state.current_task_id.lock().unwrap() = None;
        info!("All tasks reset");
    }

    (StatusCode::OK, Json(ApiResponse::ok()))
}

/// POST /api/task/delete - 删除指定任务
async fn delete_task(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<DeleteTaskRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let mut tasks = state.tasks.lock().unwrap();
    let before_count = tasks.len();
    
    tasks.retain(|t| t.id != req.task_id);
    
    let removed = before_count - tasks.len();
    if removed > 0 {
        let mut current_id = state.current_task_id.lock().unwrap();
        if *current_id == Some(req.task_id.clone()) {
            *current_id = None;
        }
        info!(task_id = %req.task_id, "Task deleted");
        (StatusCode::OK, Json(ApiResponse::ok()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Task not found")),
        )
    }
}

// ============================================================================
// MCP Handler
// ============================================================================

#[derive(Deserialize, Debug)]
struct McpRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

async fn mcp_handler(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<McpRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let result = match req.method.as_str() {
        "initialize" => {
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "vibe-process-bar",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": r#"Vibe Process Bar - AI Task Status Tracker.

Usage:
1. Call `list_tasks` first to get the current list of tasks.
2. Find the target task by matching `ide`, `project_path` (or `window_title`), and `active_file`.
3. Updates are prioritized by source: hook > mcp > plugin.

To update status:
- Call `update_task_status(task_id, status)`

Status values:
- `armed`: Task is registered and monitoring.
- `running`: Task is actively processing (AI generating).
- `completed`: Task finished successfully.
- `error`: Task failed.
- `cancelled`: Task was cancelled."#
            })
        }
        "notifications/initialized" => {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": {},
                    "id": req.id
                })),
            );
        }
        "tools/list" => {
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
                        "description": "Update a task's status. Priority: hook > mcp > plugin. Valid statuses: running, completed, error, cancelled, armed",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": { "type": "string", "description": "The task ID to update" },
                                "status": { "type": "string", "description": "New status: running, completed, error, cancelled, armed" }
                            },
                            "required": ["task_id", "status"]
                        }
                    }
                ]
            })
        }
        "tools/call" => {
            let params = req.params.unwrap_or(serde_json::json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            match tool_name {
                "list_tasks" => {
                    let mut tasks_vec = state.tasks.lock().unwrap().clone();
                    sort_tasks_by_priority(&mut tasks_vec);
                    let task_list: Vec<serde_json::Value> = tasks_vec
                        .iter()
                        .map(|t| {
                            serde_json::json!({
                                "id": t.id,
                                "ide": t.ide,
                                "window_title": t.window_title,
                                "project_path": t.project_path,
                                "active_file": t.active_file,
                                "status": t.status,
                                "progress": t.progress,
                                "source": t.source
                            })
                        })
                        .collect();

                    info!("MCP list_tasks: returning {} tasks", task_list.len());
                    serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&task_list).unwrap_or("[]".to_string())
                        }]
                    })
                }
                "update_task_status" => {
                    let task_id = arguments
                        .get("task_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let status = arguments
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let valid_statuses = ["running", "completed", "error", "cancelled", "armed"];
                    if !valid_statuses.contains(&status) {
                        return (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "jsonrpc": "2.0",
                                "error": {"code": -32602, "message": format!("Invalid status '{}'. Valid: {:?}", status, valid_statuses)},
                                "id": req.id
                            })),
                        );
                    }

                    let mut tasks = state.tasks.lock().unwrap();
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                        if !can_update_source(&task.source, "mcp") {
                            info!("MCP update_task_status: {} - ignoring, current source {} has higher priority", task_id, task.source);
                            return (
                                StatusCode::OK,
                                Json(serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "result": {
                                        "content": [{ "type": "text", "text": format!("Ignored: task {} has higher priority source '{}'", task_id, task.source) }]
                                    },
                                    "id": req.id
                                })),
                            );
                        }

                        let old_status = task.status.clone();
                        task.status = status.to_string();
                        task.source = "mcp".to_string();

                        if status == "running" && task.start_time == 0 {
                            task.start_time = now_millis();
                        } else if status == "completed"
                            || status == "error"
                            || status == "cancelled"
                        {
                            task.end_time = Some(now_millis());
                            if status == "completed" {
                                task.progress = 100;
                            }
                        }

                        info!(
                            "MCP update_task_status: {} {} -> {}",
                            task_id, old_status, status
                        );
                        serde_json::json!({
                            "content": [{ "type": "text", "text": format!("Task {} status updated: {} -> {}", task_id, old_status, status) }]
                        })
                    } else {
                        return (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "jsonrpc": "2.0",
                                "error": {"code": -32602, "message": format!("Task not found: {}", task_id)},
                                "id": req.id
                            })),
                        );
                    }
                }
                _ => {
                    return (
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {"code": -32601, "message": format!("Unknown tool: {}", tool_name)},
                            "id": req.id
                        })),
                    );
                }
            }
        }
        _ => {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32601, "message": format!("Method not found: {}", req.method)},
                    "id": req.id
                })),
            );
        }
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": req.id
        })),
    )
}

// ============================================================================
// Server Startup
// ============================================================================

fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

fn create_app(state: Arc<SharedState>) -> Router {
    Router::new()
        .route("/api/status", get(get_status))
        .route("/api/task/report", post(report_task))
        .route("/api/task/update_state", post(update_state))
        .route("/api/task/delete", post(delete_task))
        .route("/api/reset", post(reset_tasks))
        .route("/mcp", post(mcp_handler))
        .layer(create_cors_layer())
        .with_state(state)
}

pub async fn start_server(host: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = SHARED_STATE.clone();
    let app = create_app(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(host = %host, port = %port, "HTTP server (Axum) listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

pub fn start_server_background(host: String, port: u16) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            if let Err(e) = start_server(host.clone(), port).await {
                error!(host = %host, port = %port, error = %e, "HTTP server error");
            }
        });
    });
}

/// Get the global shared state for use by tray menu
pub fn get_state() -> Arc<SharedState> {
    SHARED_STATE.clone()
}
