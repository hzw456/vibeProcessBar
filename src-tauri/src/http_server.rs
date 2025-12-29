use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::spawn;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, error, instrument};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub progress: u32,
    pub tokens: u64,
    pub status: String,
    pub ide: String,
    pub window_title: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTaskRequest {
    pub task_id: String,
    pub name: String,
    pub ide: String,
    pub window_title: String,
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
    } else if request.starts_with("POST /api/task/armed") {
        // ARMED state - task is waiting for AI activity, no timer yet
        match serde_json::from_str::<StartTaskRequest>(&body) {
            Ok(req) => {
                info!(task_id = %req.task_id, name = %req.name, ide = %req.ide, "Task armed");
                let task = Task {
                    id: req.task_id.clone(),
                    name: req.name,
                    progress: 0,
                    tokens: 0,
                    status: "armed".to_string(),
                    ide: req.ide,
                    window_title: req.window_title,
                    start_time: 0,
                    end_time: None,
                };
                let mut tasks = state.tasks.lock().unwrap();
                tasks.retain(|t| t.id != req.task_id);
                tasks.push(task);
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
                // Check if task exists and is in armed state
                let existing = tasks.iter_mut().find(|t| t.id == req.task_id);
                if let Some(task) = existing {
                    // Update existing armed task to running
                    task.status = "running".to_string();
                    task.start_time = chrono::Utc::now().timestamp_millis() as u64;
                    task.name = req.name;
                } else {
                    // Create new running task
                    let task = Task {
                        id: req.task_id.clone(),
                        name: req.name,
                        progress: 0,
                        tokens: 0,
                        status: "running".to_string(),
                        ide: req.ide,
                        window_title: req.window_title,
                        start_time: chrono::Utc::now().timestamp_millis() as u64,
                        end_time: None,
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
        // ACTIVE state - window has focus, task is visible but not running
        match serde_json::from_str::<CancelRequest>(&body) {
            Ok(req) => {
                let mut tasks = state.tasks.lock().unwrap();
                let found = tasks.iter_mut().find(|t| t.id == req.task_id);
                if let Some(task) = found {
                    task.status = "active".to_string();
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

pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(SharedState::new());
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

pub fn get_state() -> Arc<SharedState> {
    Arc::new(SharedState::new())
}
