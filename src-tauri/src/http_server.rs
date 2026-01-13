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
use tracing::{debug, error, info};

lazy_static::lazy_static! {
    static ref SHARED_STATE: Arc<SharedState> = Arc::new(SharedState::new());
}

// ============================================================================
// Task 数据结构
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub is_focused: bool,
    pub ide: String,
    pub window_title: String,
    pub project_path: Option<String>,
    pub active_file: Option<String>,
    pub progress: u32,
    pub status: String,
    pub source: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    #[serde(default)]
    pub last_heartbeat: u64,
}

// ============================================================================
// Request 数据结构
// ============================================================================

#[derive(Serialize, Deserialize, Debug)]
pub struct ReportRequest {
    pub task_id: String,
    pub name: String,
    pub ide: String,
    pub window_title: String,
    #[serde(default)]
    pub is_focused: bool,
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub active_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateStateRequest {
    pub task_id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub progress: Option<u32>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetRequest {
    #[serde(default)]
    pub task_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteTaskRequest {
    pub task_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateStateByPathRequest {
    pub project_path: String,
    #[serde(default)]
    pub ide: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub progress: Option<u32>,
    #[serde(default)]
    pub source: Option<String>,
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
        Self { status: "ok".to_string(), reason: None, error: None }
    }

    fn ignored(reason: &str) -> Self {
        Self { status: "ignored".to_string(), reason: Some(reason.to_string()), error: None }
    }

    fn error(msg: &str) -> Self {
        Self { status: "error".to_string(), reason: None, error: Some(msg.to_string()) }
    }
}

#[derive(Serialize)]
struct StatusResponse {
    tasks: Vec<Task>,
    #[serde(rename = "taskCount")]
    task_count: usize,
}

// ============================================================================
// Shared State
// ============================================================================

pub struct SharedState {
    pub tasks: Mutex<Vec<Task>>,
    pub block_plugin_status: Mutex<bool>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            tasks: Mutex::new(Vec::new()),
            block_plugin_status: Mutex::new(true),
        }
    }
}

pub fn set_block_plugin_status(block: bool) {
    let state = SHARED_STATE.clone();
    *state.block_plugin_status.lock().unwrap() = block;
    info!("Block plugin status set to: {}", block);
}

#[allow(dead_code)]
pub fn get_block_plugin_status() -> bool {
    *SHARED_STATE.block_plugin_status.lock().unwrap()
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_source_priority(source: &str) -> u8 {
    match source {
        "hook" => 3,
        "mcp" => 2,
        "plugin" => 1,
        _ => 0,
    }
}

fn can_update_source(current_source: &str, new_source: &str) -> bool {
    get_source_priority(new_source) >= get_source_priority(current_source)
}

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

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}

// ============================================================================
// Task Merge Logic (Rust层合并)
// ============================================================================

/// 获取合并后的任务列表，清理过期任务
pub fn get_merged_tasks() -> Vec<Task> {
    const HEARTBEAT_TIMEOUT_MS: u64 = 5000;
    let now = now_millis();

    let state = SHARED_STATE.clone();
    
    // Clean up stale tasks
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
            info!("Cleaned up {} stale tasks", removed);
        }
    }

    let mut tasks_vec = state.tasks.lock().unwrap().clone();
    sort_tasks_by_priority(&mut tasks_vec);

    tasks_vec
}

// ============================================================================
// API Handlers
// ============================================================================

async fn get_status(State(_state): State<Arc<SharedState>>) -> Json<StatusResponse> {
    let tasks_vec = get_merged_tasks();

    Json(StatusResponse {
        task_count: tasks_vec.len(),
        tasks: tasks_vec,
    })
}

async fn report_task(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<ReportRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let mut tasks = state.tasks.lock().unwrap();
    let existing = tasks.iter_mut().find(|t| t.id == req.task_id);

    if let Some(task) = existing {
        task.last_heartbeat = now_millis();
        let focus_gained = req.is_focused && !task.is_focused;
        task.is_focused = req.is_focused;

        if focus_gained && task.status == "completed" {
            info!(task_id = %req.task_id, "Auto-transitioning completed task to armed");
            task.status = "armed".to_string();
            task.progress = 0;
            task.start_time = 0;
            task.end_time = None;
        }

        if !can_update_source(&task.source, "plugin") {
            debug!(task_id = %req.task_id, "Report ignored - lower priority source");
            return (StatusCode::OK, Json(ApiResponse::ignored("lower_priority_source")));
        }

        task.name = req.name;
        task.ide = req.ide;
        task.window_title = req.window_title;
        if let Some(path) = req.project_path {
            task.project_path = Some(path);
        }
        if let Some(file) = req.active_file {
            task.active_file = Some(file);
        }

        debug!(task_id = %req.task_id, is_focused = %req.is_focused, "Task report processed");
    } else {
        info!(task_id = %req.task_id, name = %req.name, ide = %req.ide, "Task auto-registered");
        let task = Task {
            id: req.task_id.clone(),
            name: req.name,
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
    }

    (StatusCode::OK, Json(ApiResponse::ok()))
}

async fn update_state(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<UpdateStateRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let request_source = req.source.as_deref().unwrap_or("plugin");
    
    let valid_sources = ["hook", "mcp", "plugin"];
    if !valid_sources.contains(&request_source) {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::error(&format!(
            "Invalid source '{}'. Valid: {:?}", request_source, valid_sources
        ))));
    }

    if request_source == "plugin" && *state.block_plugin_status.lock().unwrap() {
        debug!(task_id = %req.task_id, "Ignoring plugin status update - blocked");
        return (StatusCode::OK, Json(ApiResponse::ignored("plugin_status_blocked")));
    }

    let mut tasks = state.tasks.lock().unwrap();
    let found = tasks.iter_mut().find(|t| t.id == req.task_id);

    if let Some(task) = found {
        if !can_update_source(&task.source, request_source) {
            info!(task_id = %req.task_id, "Ignoring update_state - lower priority");
            return (StatusCode::OK, Json(ApiResponse::ignored("lower_priority_source")));
        }

        task.source = request_source.to_string();

        if let Some(progress) = req.progress {
            task.progress = progress.clamp(0, 100);
        }

        if let Some(ref status) = req.status {
            let valid_statuses = ["armed", "running", "completed", "error", "cancelled"];
            if !valid_statuses.contains(&status.as_str()) {
                return (StatusCode::BAD_REQUEST, Json(ApiResponse::error(&format!(
                    "Invalid status '{}'. Valid: {:?}", status, valid_statuses
                ))));
            }

            task.status = status.clone();

            if status == "running" && task.start_time == 0 {
                task.start_time = now_millis();
                info!(task_id = %req.task_id, "Task started");
            }

            if status == "completed" || status == "error" || status == "cancelled" {
                task.end_time = Some(now_millis());
                if status == "completed" {
                    task.progress = 100;
                }
                info!(task_id = %req.task_id, new_status = %status, "Task ended");
            }
        }

        (StatusCode::OK, Json(ApiResponse::ok()))
    } else {
        (StatusCode::NOT_FOUND, Json(ApiResponse::error("Task not found")))
    }
}

async fn reset_tasks(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<ResetRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let mut tasks = state.tasks.lock().unwrap();

    if let Some(task_id) = req.task_id {
        tasks.retain(|t| t.id != task_id);
        info!(task_id = %task_id, "Task removed");
    } else {
        *tasks = Vec::new();
        info!("All tasks reset");
    }

    (StatusCode::OK, Json(ApiResponse::ok()))
}

async fn delete_task(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<DeleteTaskRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let mut tasks = state.tasks.lock().unwrap();
    let before_count = tasks.len();
    
    tasks.retain(|t| t.id != req.task_id);
    
    if before_count - tasks.len() > 0 {
        info!(task_id = %req.task_id, "Task deleted");
        (StatusCode::OK, Json(ApiResponse::ok()))
    } else {
        (StatusCode::NOT_FOUND, Json(ApiResponse::error("Task not found")))
    }
}

async fn update_state_by_path(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<UpdateStateByPathRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let request_source = req.source.as_deref().unwrap_or("hook");
    
    let valid_sources = ["hook", "mcp", "plugin"];
    if !valid_sources.contains(&request_source) {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::error(&format!(
            "Invalid source '{}'. Valid: {:?}", request_source, valid_sources
        ))));
    }

    let mut tasks = state.tasks.lock().unwrap();
    
    let found = tasks.iter_mut().find(|t| {
        let path_match = t.project_path.as_ref().map_or(false, |p| p == &req.project_path);
        let ide_match = req.ide.as_ref().map_or(true, |ide| &t.ide == ide);
        path_match && ide_match
    });

    if let Some(task) = found {
        if !can_update_source(&task.source, request_source) {
            info!(project_path = %req.project_path, "Ignoring update_state_by_path - lower priority");
            return (StatusCode::OK, Json(ApiResponse::ignored("lower_priority_source")));
        }

        task.source = request_source.to_string();

        if let Some(progress) = req.progress {
            task.progress = progress.clamp(0, 100);
        }

        if let Some(ref status) = req.status {
            let valid_statuses = ["armed", "running", "completed", "error", "cancelled"];
            if !valid_statuses.contains(&status.as_str()) {
                return (StatusCode::BAD_REQUEST, Json(ApiResponse::error(&format!(
                    "Invalid status '{}'. Valid: {:?}", status, valid_statuses
                ))));
            }

            task.status = status.clone();

            if status == "running" && task.start_time == 0 {
                task.start_time = now_millis();
            }

            if status == "completed" || status == "error" || status == "cancelled" {
                task.end_time = Some(now_millis());
                if status == "completed" {
                    task.progress = 100;
                }
            }
        }

        (StatusCode::OK, Json(ApiResponse::ok()))
    } else {
        (StatusCode::NOT_FOUND, Json(ApiResponse::error("Task not found for project_path")))
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
                "instructions": "Vibe Process Bar - AI Task Status Tracker."
            })
        }
        "notifications/initialized" => {
            return (StatusCode::OK, Json(serde_json::json!({
                "jsonrpc": "2.0",
                "result": {},
                "id": req.id
            })));
        }
        "tools/list" => {
            serde_json::json!({
                "tools": [
                    {
                        "name": "list_tasks",
                        "description": "Get all IDE windows/tasks",
                        "inputSchema": { "type": "object", "properties": {}, "required": [] }
                    },
                    {
                        "name": "update_task_status",
                        "description": "Update a task's status",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": { "type": "string" },
                                "status": { "type": "string" }
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
            let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

            match tool_name {
                "list_tasks" => {
                    let tasks_vec = get_merged_tasks();
                    let task_list: Vec<serde_json::Value> = tasks_vec.iter().map(|t| {
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
                    }).collect();

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

                    let valid_statuses = ["running", "completed", "error", "cancelled", "armed"];
                    if !valid_statuses.contains(&status) {
                        return (StatusCode::OK, Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {"code": -32602, "message": format!("Invalid status '{}'", status)},
                            "id": req.id
                        })));
                    }

                    let mut tasks = state.tasks.lock().unwrap();
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                        if !can_update_source(&task.source, "mcp") {
                            return (StatusCode::OK, Json(serde_json::json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "content": [{ "type": "text", "text": format!("Ignored: higher priority source") }]
                                },
                                "id": req.id
                            })));
                        }

                        let old_status = task.status.clone();
                        task.status = status.to_string();
                        task.source = "mcp".to_string();

                        if status == "running" && task.start_time == 0 {
                            task.start_time = now_millis();
                        } else if ["completed", "error", "cancelled"].contains(&status) {
                            task.end_time = Some(now_millis());
                            if status == "completed" {
                                task.progress = 100;
                            }
                        }

                        serde_json::json!({
                            "content": [{ "type": "text", "text": format!("Task {} status: {} -> {}", task_id, old_status, status) }]
                        })
                    } else {
                        return (StatusCode::OK, Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {"code": -32602, "message": format!("Task not found: {}", task_id)},
                            "id": req.id
                        })));
                    }
                }
                _ => {
                    return (StatusCode::OK, Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32601, "message": format!("Unknown tool: {}", tool_name)},
                        "id": req.id
                    })));
                }
            }
        }
        _ => {
            return (StatusCode::OK, Json(serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32601, "message": format!("Method not found: {}", req.method)},
                "id": req.id
            })));
        }
    };

    (StatusCode::OK, Json(serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": req.id
    })))
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
        .route("/api/task/update_state_by_path", post(update_state_by_path))
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
    info!(host = %host, port = %port, "HTTP server listening on {}", addr);

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

#[allow(dead_code)]
pub fn get_state() -> Arc<SharedState> {
    SHARED_STATE.clone()
}
