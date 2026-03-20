use axum::{
    extract::{Path, State},
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
    static ref BACKEND_EMAIL: Mutex<String> = Mutex::new("fulltest@vibe.app".to_string());
    static ref BACKEND_SERVER_URL: Mutex<String> = Mutex::new("http://localhost:3010".to_string());
    static ref BACKEND_API_KEY: Mutex<String> = Mutex::new(String::new());
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
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
    pub status: String,
    pub source: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    #[serde(default)]
    pub last_heartbeat: u64,
    /// 预估总时长（毫秒）
    #[serde(default)]
    pub estimated_duration: Option<u64>,
    /// 当前阶段描述
    #[serde(default)]
    pub current_stage: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BackendStateSnapshot {
    status: String,
    project_path: Option<String>,
    active_file: Option<String>,
    current_stage: Option<String>,
    window_title: String,
    is_focused: bool,
}

impl From<&Task> for BackendStateSnapshot {
    fn from(task: &Task) -> Self {
        Self {
            status: task.status.clone(),
            project_path: task.project_path.clone(),
            active_file: task.active_file.clone(),
            current_stage: task.current_stage.clone(),
            window_title: task.window_title.clone(),
            is_focused: task.is_focused,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BackendProgressSnapshot {
    estimated_duration: Option<u64>,
    current_stage: Option<String>,
}

impl From<&Task> for BackendProgressSnapshot {
    fn from(task: &Task) -> Self {
        Self {
            estimated_duration: task.estimated_duration,
            current_stage: task.current_stage.clone(),
        }
    }
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
    pub source: Option<String>,
    #[serde(default)]
    pub window_title: Option<String>,
    #[serde(default)]
    pub is_focused: Option<bool>,
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub active_file: Option<String>,
    /// 预估总时长（毫秒）
    #[serde(default)]
    pub estimated_duration: Option<u64>,
    /// 当前阶段描述
    #[serde(default)]
    pub current_stage: Option<String>,
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
    tasks: Vec<Task>,
    #[serde(rename = "taskCount")]
    task_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TaskStage {
    stage: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    started_at: Option<u64>,
    #[serde(default)]
    ended_at: Option<u64>,
    #[serde(default)]
    duration: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TaskStagesResponse {
    stages: Vec<TaskStage>,
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

pub fn set_backend_email(email: String) {
    let mut e = BACKEND_EMAIL.lock().unwrap();
    *e = email;
}

pub fn set_backend_server_url(url: String) {
    let normalized = url.trim_end_matches('/').to_string();
    *BACKEND_SERVER_URL.lock().unwrap() = normalized.clone();
    info!("Backend server URL set to: {}", normalized);
}

pub fn set_backend_api_key(api_key: String) {
    *BACKEND_API_KEY.lock().unwrap() = api_key;
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

fn should_sync_backend_state_for_update(req: &UpdateStateRequest) -> bool {
    req.status.is_some()
        || req.project_path.is_some()
        || req.active_file.is_some()
        || req.window_title.is_some()
        || req.is_focused.is_some()
}

fn has_backend_state_changed(before: &BackendStateSnapshot, after: &Task) -> bool {
    before != &BackendStateSnapshot::from(after)
}

fn has_backend_progress_changed(before: &BackendProgressSnapshot, after: &Task) -> bool {
    before != &BackendProgressSnapshot::from(after)
}

fn is_terminal_status(status: &str) -> bool {
    matches!(status, "completed" | "error" | "cancelled")
}

fn normalize_requested_stage(status: &str, stage: Option<&str>) -> Option<String> {
    let trimmed = stage.map(str::trim).filter(|value| !value.is_empty())?;

    match status {
        "completed" if trimmed == "__completed__" => Some(trimmed.to_string()),
        "running" | "armed" => Some(trimmed.to_string()),
        _ => None,
    }
}

fn apply_task_status_transition(
    task: &mut Task,
    new_status: &str,
    has_explicit_estimated_duration: bool,
    has_explicit_current_stage: bool,
) {
    let old_status = task.status.clone();
    task.status = new_status.to_string();

    apply_running_transition_defaults(
        task,
        &old_status,
        new_status,
        has_explicit_estimated_duration,
        has_explicit_current_stage,
    );

    if is_terminal_status(new_status) {
        if old_status != new_status {
            task.end_time = Some(now_millis());
        }

        if new_status == "completed" {
            task.current_stage = Some("__completed__".to_string());
        }

        if old_status != new_status {
            info!(task_id = %task.id, new_status = %new_status, "Task ended");
        }
    }

    if new_status == "armed" {
        task.estimated_duration = None;
        task.current_stage = None;
        task.start_time = 0;
        task.end_time = None;
    }
}

fn apply_task_progress_update(
    task: &mut Task,
    estimated_duration: Option<u64>,
    requested_stage: Option<&str>,
) {
    if let Some(estimated_duration) = estimated_duration {
        task.estimated_duration = Some(estimated_duration);
    }

    if let Some(stage) = normalize_requested_stage(&task.status, requested_stage) {
        task.current_stage = Some(stage);
    }
}

fn sort_stage_records_desc(stages: &mut [TaskStage]) {
    stages.sort_by(|a, b| {
        b.started_at
            .unwrap_or(0)
            .cmp(&a.started_at.unwrap_or(0))
            .then_with(|| b.ended_at.unwrap_or(0).cmp(&a.ended_at.unwrap_or(0)))
    });
}

fn normalize_task_stages(stages: Vec<TaskStage>) -> Vec<TaskStage> {
    let mut merged: Vec<TaskStage> = Vec::new();

    for mut stage in stages {
        stage.stage = stage.stage.trim().to_string();
        if stage.stage.is_empty() {
            continue;
        }

        if let (Some(started_at), Some(ended_at)) = (stage.started_at, stage.ended_at) {
            if ended_at < started_at {
                stage.ended_at = Some(started_at);
            }
        }

        if stage.duration.is_none() {
            stage.duration = match (stage.started_at, stage.ended_at) {
                (Some(started_at), Some(ended_at)) => Some(ended_at.saturating_sub(started_at)),
                _ => None,
            };
        }

        if let Some(existing) = merged.iter_mut().find(|item| item.stage == stage.stage) {
            existing.started_at = match (existing.started_at, stage.started_at) {
                (Some(left), Some(right)) => Some(left.min(right)),
                (Some(left), None) => Some(left),
                (None, Some(right)) => Some(right),
                (None, None) => None,
            };

            existing.ended_at = match (existing.ended_at, stage.ended_at) {
                (Some(left), Some(right)) => Some(left.max(right)),
                (Some(left), None) => Some(left),
                (None, Some(right)) => Some(right),
                (None, None) => None,
            };

            if let (Some(started_at), Some(ended_at)) = (existing.started_at, existing.ended_at) {
                if ended_at < started_at {
                    existing.ended_at = Some(started_at);
                }
            }

            existing.duration = match (existing.started_at, existing.ended_at) {
                (Some(started_at), Some(ended_at)) => Some(ended_at.saturating_sub(started_at)),
                _ => existing.duration.or(stage.duration),
            };

            if existing.description.is_none() {
                existing.description = stage.description.clone();
            }
        } else {
            merged.push(stage);
        }
    }

    sort_stage_records_desc(&mut merged);
    merged
}

fn apply_running_transition_defaults(
    task: &mut Task,
    old_status: &str,
    new_status: &str,
    has_explicit_estimated_duration: bool,
    has_explicit_current_stage: bool,
) {
    if new_status != "running" {
        return;
    }

    if old_status == "completed" || old_status == "error" || old_status == "cancelled" {
        task.start_time = now_millis();
        task.end_time = None;

        if !has_explicit_estimated_duration {
            task.estimated_duration = None;
        }

        if !has_explicit_current_stage {
            task.current_stage = task.active_file.clone();
        }

        info!(task_id = %task.id, "Task restarted from {}", old_status);
    } else if task.start_time == 0 {
        task.start_time = now_millis();

        if !has_explicit_current_stage && task.current_stage.is_none() {
            task.current_stage = task.active_file.clone();
        }

        info!(task_id = %task.id, "Task started");
    }
}

#[derive(Serialize)]
struct BackendUpdateStateRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    user_email: Option<String>,
    task_id: &'a str,
    status: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_file: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_stage: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_focused: Option<bool>,
}

#[derive(Serialize)]
struct BackendUpdateProgressRequest<'a> {
    task_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_stage: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_file: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_focused: Option<bool>,
}

async fn sync_backend_state(
    task_id: &str,
    status: &str,
    project_path: Option<&str>,
    active_file: Option<&str>,
    current_stage: Option<&str>,
    window_title: Option<&str>,
    is_focused: Option<bool>,
) {
    let base_url = BACKEND_SERVER_URL.lock().unwrap().clone();
    let api_key = BACKEND_API_KEY.lock().unwrap().clone();
    let url = format!("{}/api/task/update_state", base_url);
    let user_email = BACKEND_EMAIL.lock().unwrap().clone();
    let payload = BackendUpdateStateRequest {
        user_email: Some(user_email),
        task_id,
        status,
        project_path,
        active_file,
        current_stage,
        window_title,
        is_focused,
    };

    let request = HTTP_CLIENT.post(&url).json(&payload);
    let request = if api_key.is_empty() {
        request
    } else {
        request.header("x-api-key", api_key)
    };

    match request.send().await {
        Ok(response) if response.status().is_success() => {
            debug!(task_id = %task_id, status = %status, "Synced task state to backend");
        }
        Ok(response) => {
            let http_status = response.status();
            let response_body = response
                .text()
                .await
                .unwrap_or_else(|err| format!("<failed to read response body: {}>", err));
            error!(
                task_id = %task_id,
                status = %status,
                backend_url = %url,
                http_status = %http_status,
                response_body = %response_body,
                "Backend task state sync failed"
            );
        }
        Err(err) => {
            error!(
                task_id = %task_id,
                status = %status,
                backend_url = %url,
                error = %err,
                "Backend task state sync request failed"
            );
        }
    }
}

async fn sync_backend_progress(
    task_id: &str,
    estimated_duration_ms: Option<u64>,
    current_stage: Option<&str>,
    active_file: Option<&str>,
    window_title: Option<&str>,
    is_focused: Option<bool>,
) {
    let base_url = BACKEND_SERVER_URL.lock().unwrap().clone();
    let api_key = BACKEND_API_KEY.lock().unwrap().clone();
    let url = format!("{}/api/task/update_progress", base_url);
    let payload = BackendUpdateProgressRequest {
        task_id,
        estimated_duration_ms,
        current_stage,
        active_file,
        window_title,
        is_focused,
    };

    let request = HTTP_CLIENT.post(&url).json(&payload);
    let request = if api_key.is_empty() {
        request
    } else {
        request.header("x-api-key", api_key)
    };

    match request.send().await {
        Ok(response) if response.status().is_success() => {
            debug!(task_id = %task_id, "Synced task progress to backend");
        }
        Ok(response) => {
            let http_status = response.status();
            let response_body = response
                .text()
                .await
                .unwrap_or_else(|err| format!("<failed to read response body: {}>", err));
            error!(
                task_id = %task_id,
                backend_url = %url,
                http_status = %http_status,
                response_body = %response_body,
                "Backend task progress sync failed"
            );
        }
        Err(err) => {
            error!(
                task_id = %task_id,
                backend_url = %url,
                error = %err,
                "Backend task progress sync request failed"
            );
        }
    }
}

async fn fetch_backend_task_stages(task_id: &str) -> Result<Vec<TaskStage>, String> {
    let base_url = BACKEND_SERVER_URL.lock().unwrap().clone();
    let api_key = BACKEND_API_KEY.lock().unwrap().clone();
    let url = format!("{}/api/task/{}/stages", base_url, task_id);

    let request = HTTP_CLIENT.get(&url);
    let request = if api_key.is_empty() {
        request
    } else {
        request.header("x-api-key", api_key)
    };

    let response = request
        .send()
        .await
        .map_err(|err| format!("Failed to fetch task stages: {}", err))?;

    if !response.status().is_success() {
        let http_status = response.status();
        let response_body = response
            .text()
            .await
            .unwrap_or_else(|err| format!("<failed to read response body: {}>", err));
        return Err(format!(
            "Failed to fetch task stages: HTTP {} {}",
            http_status, response_body
        ));
    }

    let mut payload = response
        .json::<TaskStagesResponse>()
        .await
        .map_err(|err| format!("Invalid task stages response: {}", err))?;

    payload.stages.sort_by(|a, b| {
        b.started_at
            .unwrap_or(0)
            .cmp(&a.started_at.unwrap_or(0))
            .then_with(|| b.ended_at.unwrap_or(0).cmp(&a.ended_at.unwrap_or(0)))
    });

    Ok(payload.stages)
}

/// 重置任务为 armed 状态，并同步到后端
pub async fn reset_task_to_armed(task_id: &str) -> Result<(), String> {
    let state = SHARED_STATE.clone();
    let state_sync = {
        let mut tasks = state.tasks.lock().unwrap();

        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            apply_task_status_transition(task, "armed", false, false);
            info!(task_id = %task_id, "Task reset to armed");

            Some((
                task.id.clone(),
                task.status.clone(),
                task.project_path.clone(),
                task.active_file.clone(),
                task.current_stage.clone(),
                Some(task.window_title.clone()),
                Some(task.is_focused),
            ))
        } else {
            None
        }
    };

    if let Some((
        task_id,
        status,
        project_path,
        active_file,
        current_stage,
        window_title,
        is_focused,
    )) = state_sync
    {
        sync_backend_state(
            &task_id,
            &status,
            project_path.as_deref(),
            active_file.as_deref(),
            current_stage.as_deref(),
            window_title.as_deref(),
            is_focused,
        )
        .await;

        Ok(())
    } else {
        Err(format!("Task not found: {}", task_id))
    }
}

/// 取消任务并保留 cancelled 状态，避免立即回到 armed
pub async fn cancel_task(task_id: &str) -> Result<(), String> {
    let state = SHARED_STATE.clone();
    let state_sync = {
        let mut tasks = state.tasks.lock().unwrap();

        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            apply_task_status_transition(task, "cancelled", false, false);
            info!(task_id = %task_id, "Task marked as cancelled");

            Some((
                task.id.clone(),
                task.status.clone(),
                task.project_path.clone(),
                task.active_file.clone(),
                task.current_stage.clone(),
                Some(task.window_title.clone()),
                Some(task.is_focused),
            ))
        } else {
            None
        }
    };

    if let Some((
        task_id,
        status,
        project_path,
        active_file,
        current_stage,
        window_title,
        is_focused,
    )) = state_sync
    {
        sync_backend_state(
            &task_id,
            &status,
            project_path.as_deref(),
            active_file.as_deref(),
            current_stage.as_deref(),
            window_title.as_deref(),
            is_focused,
        )
        .await;
        Ok(())
    } else {
        Err(format!("Task not found: {}", task_id))
    }
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

async fn get_task_stages(
    Path(task_id): Path<String>,
) -> (StatusCode, Json<TaskStagesResponse>) {
    match fetch_backend_task_stages(&task_id).await {
        Ok(stages) => (
            StatusCode::OK,
            Json(TaskStagesResponse {
                stages: normalize_task_stages(stages),
            }),
        ),
        Err(err) => {
            error!(task_id = %task_id, error = %err, "Failed to get task stages");
            (StatusCode::BAD_GATEWAY, Json(TaskStagesResponse { stages: vec![] }))
        }
    }
}

async fn report_task(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<ReportRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let mut initial_state_sync: Option<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<bool>,
    )> = None;
    {
        let mut tasks = state.tasks.lock().unwrap();
        let existing = tasks.iter_mut().find(|t| t.id == req.task_id);

        if let Some(task) = existing {
            task.last_heartbeat = now_millis();
            task.is_focused = req.is_focused;

            if !can_update_source(&task.source, "plugin") {
                debug!(task_id = %req.task_id, "Report ignored - lower priority source");
                return (
                    StatusCode::OK,
                    Json(ApiResponse::ignored("lower_priority_source")),
                );
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
            let project_path = req.project_path.clone();
            let active_file = req.active_file.clone();
            let window_title = req.window_title.clone();
            let is_focused = req.is_focused;
            let task = Task {
                id: req.task_id.clone(),
                name: req.name,
                status: "armed".to_string(),
                is_focused,
                ide: req.ide,
                window_title: window_title.clone(),
                start_time: 0,
                end_time: None,
                project_path: req.project_path,
                active_file: req.active_file,
                source: "plugin".to_string(),
                last_heartbeat: now_millis(),
                estimated_duration: None,
                current_stage: None,
            };
            tasks.push(task);
            initial_state_sync = Some((
                req.task_id,
                "armed".to_string(),
                project_path,
                active_file,
                None,
                Some(window_title),
                Some(is_focused),
            ));
        }
    }

    if let Some((
        task_id,
        status,
        project_path,
        active_file,
        current_stage,
        window_title,
        is_focused,
    )) = initial_state_sync
    {
        sync_backend_state(
            &task_id,
            &status,
            project_path.as_deref(),
            active_file.as_deref(),
            current_stage.as_deref(),
            window_title.as_deref(),
            is_focused,
        )
        .await;
    }

    (StatusCode::OK, Json(ApiResponse::ok()))
}

async fn update_state(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<UpdateStateRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let request_source = req.source.as_deref().unwrap_or("plugin");
    let has_explicit_estimated_duration = req.estimated_duration.is_some();
    let has_explicit_current_stage = req.current_stage.is_some();

    let valid_sources = ["hook", "mcp", "plugin"];
    if !valid_sources.contains(&request_source) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!(
                "Invalid source '{}'. Valid: {:?}",
                request_source, valid_sources
            ))),
        );
    }

    if request_source == "plugin" && *state.block_plugin_status.lock().unwrap() {
        debug!(task_id = %req.task_id, "Ignoring plugin status update - blocked");
        return (
            StatusCode::OK,
            Json(ApiResponse::ignored("plugin_status_blocked")),
        );
    }

    let mut state_sync: Option<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<bool>,
    )> = None;
    let mut progress_sync: Option<(
        String,
        Option<u64>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<bool>,
    )> = None;

    {
        let mut tasks = state.tasks.lock().unwrap();
        let found = tasks.iter_mut().find(|t| t.id == req.task_id);

        if let Some(task) = found {
            let backend_state_before = BackendStateSnapshot::from(&*task);
            let backend_progress_before = BackendProgressSnapshot::from(&*task);

            if !can_update_source(&task.source, request_source) {
                info!(task_id = %req.task_id, "Ignoring update_state - lower priority");
                return (
                    StatusCode::OK,
                    Json(ApiResponse::ignored("lower_priority_source")),
                );
            }

            task.source = request_source.to_string();

            if let Some(ref project_path) = req.project_path {
                task.project_path = Some(project_path.clone());
            }

            if let Some(ref active_file) = req.active_file {
                task.active_file = Some(active_file.clone());
            }
            if let Some(ref window_title) = req.window_title {
                task.window_title = window_title.clone();
            }
            if let Some(is_focused) = req.is_focused {
                task.is_focused = is_focused;
            }

            if let Some(ref status) = req.status {
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

                apply_task_status_transition(
                    task,
                    status,
                    has_explicit_estimated_duration,
                    has_explicit_current_stage,
                );
            }

            apply_task_progress_update(task, req.estimated_duration, req.current_stage.as_deref());

            if should_sync_backend_state_for_update(&req)
                && has_backend_state_changed(&backend_state_before, task)
            {
                state_sync = Some((
                    task.id.clone(),
                    task.status.clone(),
                    task.project_path.clone(),
                    task.active_file.clone(),
                    task.current_stage.clone(),
                    Some(task.window_title.clone()),
                    Some(task.is_focused),
                ));
            } else if should_sync_backend_state_for_update(&req) {
                debug!(task_id = %req.task_id, "Skipping backend state sync for no-op update");
            }

            if (req.estimated_duration.is_some() || req.current_stage.is_some())
                && has_backend_progress_changed(&backend_progress_before, task)
            {
                progress_sync = Some((
                    task.id.clone(),
                    task.estimated_duration,
                    task.current_stage.clone(),
                    task.active_file.clone(),
                    Some(task.window_title.clone()),
                    Some(task.is_focused),
                ));
            } else if req.estimated_duration.is_some() || req.current_stage.is_some() {
                debug!(task_id = %req.task_id, "Skipping backend progress sync for no-op update");
            }
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Task not found")),
            );
        }
    }

    if let Some((
        task_id,
        status,
        project_path,
        active_file,
        current_stage,
        window_title,
        is_focused,
    )) = state_sync
    {
        sync_backend_state(
            &task_id,
            &status,
            project_path.as_deref(),
            active_file.as_deref(),
            current_stage.as_deref(),
            window_title.as_deref(),
            is_focused,
        )
        .await;
    }

    if let Some((
        task_id,
        estimated_duration,
        current_stage,
        active_file,
        window_title,
        is_focused,
    )) = progress_sync
    {
        sync_backend_progress(
            &task_id,
            estimated_duration,
            current_stage.as_deref(),
            active_file.as_deref(),
            window_title.as_deref(),
            is_focused,
        )
        .await;
    }

    (StatusCode::OK, Json(ApiResponse::ok()))
}

#[cfg(test)]
mod tests {
    use super::{
        apply_running_transition_defaults, should_sync_backend_state_for_update, Task,
        UpdateStateRequest,
    };

    fn sample_task(status: &str) -> Task {
        Task {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            is_focused: false,
            ide: "cursor".to_string(),
            window_title: "main.rs".to_string(),
            project_path: Some("/tmp/project".to_string()),
            active_file: Some("src/main.rs".to_string()),
            status: status.to_string(),
            source: "hook".to_string(),
            start_time: 1,
            end_time: Some(2),
            last_heartbeat: 0,
            estimated_duration: Some(10_000),
            current_stage: Some("__completed__".to_string()),
        }
    }

    #[test]
    fn running_restart_keeps_explicit_progress_fields() {
        let mut task = sample_task("completed");

        apply_running_transition_defaults(&mut task, "completed", "running", true, true);

        assert_eq!(task.end_time, None);
        assert_eq!(task.estimated_duration, Some(10_000));
        assert_eq!(task.current_stage.as_deref(), Some("__completed__"));
    }

    #[test]
    fn running_restart_falls_back_to_active_file_without_explicit_stage() {
        let mut task = sample_task("completed");

        apply_running_transition_defaults(&mut task, "completed", "running", false, false);

        assert_eq!(task.end_time, None);
        assert_eq!(task.estimated_duration, None);
        assert_eq!(task.current_stage.as_deref(), Some("src/main.rs"));
    }

    #[test]
    fn metadata_only_update_triggers_backend_state_sync() {
        let req = UpdateStateRequest {
            task_id: "task-1".to_string(),
            status: None,
            source: Some("hook".to_string()),
            window_title: Some("main.rs".to_string()),
            is_focused: Some(true),
            project_path: None,
            active_file: Some("src/main.rs".to_string()),
            estimated_duration: None,
            current_stage: None,
        };

        assert!(should_sync_backend_state_for_update(&req));
    }

    #[test]
    fn progress_only_update_does_not_trigger_state_sync() {
        let req = UpdateStateRequest {
            task_id: "task-1".to_string(),
            status: None,
            source: Some("mcp".to_string()),
            window_title: None,
            is_focused: None,
            project_path: None,
            active_file: None,
            estimated_duration: Some(5_000),
            current_stage: Some("Analyzing code".to_string()),
        };

        assert!(!should_sync_backend_state_for_update(&req));
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
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Task not found")),
        )
    }
}

async fn update_state_by_path(
    State(state): State<Arc<SharedState>>,
    Json(req): Json<UpdateStateByPathRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    let request_source = req.source.as_deref().unwrap_or("hook");

    let valid_sources = ["hook", "mcp", "plugin"];
    if !valid_sources.contains(&request_source) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!(
                "Invalid source '{}'. Valid: {:?}",
                request_source, valid_sources
            ))),
        );
    }

    let state_sync = {
        let mut tasks = state.tasks.lock().unwrap();

        let found = tasks.iter_mut().find(|t| {
            let path_match = t
                .project_path
                .as_ref()
                .map_or(false, |p| p == &req.project_path);
            let ide_match = req.ide.as_ref().map_or(true, |ide| &t.ide == ide);
            path_match && ide_match
        });

        if let Some(task) = found {
            let backend_state_before = BackendStateSnapshot::from(&*task);

            if !can_update_source(&task.source, request_source) {
                info!(project_path = %req.project_path, "Ignoring update_state_by_path - lower priority");
                return (
                    StatusCode::OK,
                    Json(ApiResponse::ignored("lower_priority_source")),
                );
            }

            task.source = request_source.to_string();

            if let Some(ref status) = req.status {
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

                apply_task_status_transition(task, status, false, false);
            }

            if has_backend_state_changed(&backend_state_before, task) {
                Some((
                    task.id.clone(),
                    task.status.clone(),
                    task.project_path.clone(),
                    task.active_file.clone(),
                    task.current_stage.clone(),
                    Some(task.window_title.clone()),
                    Some(task.is_focused),
                ))
            } else {
                debug!(project_path = %req.project_path, "Skipping backend state sync for no-op path update");
                None
            }
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Task not found for project_path")),
            );
        }
    };

    if let Some((
        task_id,
        status,
        project_path,
        active_file,
        current_stage,
        window_title,
        is_focused,
    )) = state_sync
    {
        sync_backend_state(
            &task_id,
            &status,
            project_path.as_deref(),
            active_file.as_deref(),
            current_stage.as_deref(),
            window_title.as_deref(),
            is_focused,
        )
        .await;
    }

    (StatusCode::OK, Json(ApiResponse::ok()))
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

use axum::response::sse::{Event, Sse, KeepAlive};
use futures::stream;

async fn mcp_get_handler() -> Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>> {
    // MCP SSE endpoint - sends keepalive comments to keep connection open
    let event = Event::default().comment("MCP SSE connection established");
    Sse::new(stream::once(async move { Ok::<_, std::convert::Infallible>(event) }))
        .keep_alive(KeepAlive::default())
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
                    },
                    {
                        "name": "update_task_progress",
                        "description": "Update task estimated duration and current stage description. Any source can update these fields.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": { "type": "string", "description": "The task ID to update" },
                                "estimated_duration_ms": { "type": "integer", "description": "Estimated total duration in milliseconds" },
                                "current_stage": { "type": "string", "description": "Current stage description (e.g. 'Analyzing code...', 'Modifying files...')" }
                            },
                            "required": ["task_id"]
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
                    let tasks_vec = get_merged_tasks();
                    let now = now_millis();
                    let task_list: Vec<serde_json::Value> = tasks_vec
                        .iter()
                        .map(|t| {
                            let elapsed_ms = if t.start_time > 0 {
                                now.saturating_sub(t.start_time)
                            } else {
                                0
                            };

                            // 进度由 elapsed / effective_estimated 计算
                            // 如果运行时间超过预估时间，预估时间跟随运行时间
                            let calculated_progress = if let Some(estimated) = t.estimated_duration
                            {
                                if estimated > 0 {
                                    if t.status == "completed" {
                                        100
                                    } else {
                                        let effective_estimated =
                                            std::cmp::max(estimated, elapsed_ms);
                                        ((elapsed_ms as f64 / effective_estimated as f64) * 100.0)
                                            .min(99.0)
                                            as u32
                                    }
                                } else {
                                    if t.status == "completed" {
                                        100
                                    } else {
                                        0
                                    }
                                }
                            } else {
                                if t.status == "completed" {
                                    100
                                } else {
                                    0
                                }
                            };

                            serde_json::json!({
                                "id": t.id,
                                "ide": t.ide,
                                "window_title": t.window_title,
                                "project_path": t.project_path,
                                "active_file": t.active_file,
                                "status": t.status,
                                "progress": calculated_progress,
                                "source": t.source,
                                "current_stage": t.current_stage
                            })
                        })
                        .collect();

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
                                "error": {"code": -32602, "message": format!("Invalid status '{}'", status)},
                                "id": req.id
                            })),
                        );
                    }

                    let (old_status, sync_state_result) = {
                        let mut tasks = state.tasks.lock().unwrap();
                        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                            if !can_update_source(&task.source, "mcp") {
                                return (
                                    StatusCode::OK,
                                    Json(serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "result": {
                                            "content": [{ "type": "text", "text": format!("Ignored: higher priority source") }]
                                        },
                                        "id": req.id
                                    })),
                                );
                            }

                            let old_status = task.status.clone();
                            task.status = status.to_string();
                            task.source = "mcp".to_string();

                            if status == "running" {
                                if ["completed", "error", "cancelled"]
                                    .contains(&old_status.as_str())
                                {
                                    task.start_time = now_millis();
                                    task.end_time = None;
                                    task.estimated_duration = None;
                                    task.current_stage = task.active_file.clone();
                                } else if task.start_time == 0 {
                                    task.start_time = now_millis();
                                }
                            } else if ["completed", "error", "cancelled"].contains(&status) {
                                task.end_time = Some(now_millis());
                                if status == "completed" {
                                    task.current_stage = Some("__completed__".to_string());
                                }
                            } else if status == "armed" {
                                task.estimated_duration = None;
                                task.current_stage = None;
                                task.start_time = 0;
                                task.end_time = None;
                            }

                            (
                                old_status,
                                Some((
                                    task.id.clone(),
                                    task.status.clone(),
                                    task.project_path.clone(),
                                    task.active_file.clone(),
                                    task.current_stage.clone(),
                                    Some(task.window_title.clone()),
                                    Some(task.is_focused),
                                )),
                            )
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
                    };

                    if let Some((
                        task_id,
                        status,
                        project_path,
                        active_file,
                        current_stage,
                        window_title,
                        is_focused,
                    )) = sync_state_result
                    {
                        sync_backend_state(
                            &task_id,
                            &status,
                            project_path.as_deref(),
                            active_file.as_deref(),
                            current_stage.as_deref(),
                            window_title.as_deref(),
                            is_focused,
                        )
                        .await;
                    }

                    serde_json::json!({
                        "content": [{ "type": "text", "text": format!("Task {} status: {} -> {}", task_id, old_status, status) }]
                    })
                }
                "update_task_progress" => {
                    let task_id = arguments
                        .get("task_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let estimated_duration = arguments
                        .get("estimated_duration_ms")
                        .and_then(|v| v.as_u64());
                    let current_stage = arguments.get("current_stage").and_then(|v| v.as_str());

                    let sync_progress_result = {
                        let mut tasks = state.tasks.lock().unwrap();
                        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                            if let Some(est) = estimated_duration {
                                task.estimated_duration = Some(est);
                            }

                            if let Some(stage) = current_stage {
                                task.current_stage = Some(stage.to_string());
                            }

                            Some((
                                task.id.clone(),
                                task.estimated_duration,
                                task.current_stage.clone(),
                                task.active_file.clone(),
                                Some(task.window_title.clone()),
                                Some(task.is_focused),
                            ))
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
                    };

                    if let Some((
                        task_id,
                        estimated_duration,
                        current_stage,
                        active_file,
                        window_title,
                        is_focused,
                    )) = sync_progress_result
                    {
                        sync_backend_progress(
                            &task_id,
                            estimated_duration,
                            current_stage.as_deref(),
                            active_file.as_deref(),
                            window_title.as_deref(),
                            is_focused,
                        )
                        .await;
                    }

                    serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Updated task {}", task_id)
                        }]
                    })
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
        .route("/api/task/:task_id/stages", get(get_task_stages))
        .route("/api/task/report", post(report_task))
        .route("/api/task/update_state", post(update_state))
        .route("/api/task/update_state_by_path", post(update_state_by_path))
        .route("/api/task/delete", post(delete_task))
        .route("/api/reset", post(reset_tasks))
        .route("/mcp", get(mcp_get_handler).post(mcp_handler))
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
