use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::spawn;
use std::io::{Read, Write};

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

fn parse_json<T: serde::Deserialize>(body: &str) -> Result<T, String> {
    serde_json::from_str(body).map_err(|e| e.to_string())
}

async fn handle_connection(
    stream: &mut tokio::net::tcp::TcpStream,
    state: &Arc<SharedState>,
) -> Result<(), std::io::Error> {
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer).await?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    let (status_line, response) = if request.contains("GET /api/status") {
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
        ("HTTP/1.1 200 OK", format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
            resp.to_string()
        ))
    } else if request.contains("POST /api/task/start") {
        let body = extract_body(&request);
        if let Ok(req) = parse_json::<StartTaskRequest>(&body) {
            let task = Task {
                id: req.task_id.clone(),
                name: req.name,
                progress: 0,
                tokens: 0,
                status: "running".to_string(),
                ide: req.ide,
                window_title: req.window_title,
                start_time: chrono::Utc::now().timestamp_millis() as u64,
            };
            let mut tasks = state.tasks.lock().unwrap();
            tasks.retain(|t| t.id != req.task_id);
            tasks.push(task.clone());
            *state.current_task_id.lock().unwrap() = Some(req.task_id);
            ("HTTP/1.1 200 OK", r#"{"status":"ok"}"#.to_string())
        } else {
            ("HTTP/1.1 400 Bad Request", r#"{"error":"Invalid request"}"#.to_string())
        }
    } else if request.contains("POST /api/task/progress") {
        let body = extract_body(&request);
        if let Ok(req) = parse_json::<ProgressRequest>(&body) {
            let mut tasks = state.tasks.lock().unwrap();
            for task in tasks.iter_mut() {
                if task.id == req.task_id {
                    task.progress = req.progress.clamp(0, 100);
                    return Ok(stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\"}").await?);
                }
            }
            ("HTTP/1.1 404 Not Found", r#"{"error":"Task not found"}"#.to_string())
        } else {
            ("HTTP/1.1 400 Bad Request", r#"{"error":"Invalid request"}"#.to_string())
        }
    } else if request.contains("POST /api/task/token") {
        let body = extract_body(&request);
        if let Ok(req) = parse_json::<TokenRequest>(&body) {
            let mut tasks = state.tasks.lock().unwrap();
            for task in tasks.iter_mut() {
                if task.id == req.task_id {
                    let increment = req.increment.unwrap_or(false);
                    if increment {
                        task.tokens += req.tokens;
                    } else {
                        task.tokens = req.tokens;
                    }
                    return Ok(stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\"}").await?);
                }
            }
            ("HTTP/1.1 404 Not Found", r#"{"error":"Task not found"}"#.to_string())
        } else {
            ("HTTP/1.1 400 Bad Request", r#"{"error":"Invalid request"}"#.to_string())
        }
    } else if request.contains("POST /api/task/complete") {
        let body = extract_body(&request);
        if let Ok(req) = parse_json::<CompleteRequest>(&body) {
            let mut tasks = state.tasks.lock().unwrap();
            for task in tasks.iter_mut() {
                if task.id == req.task_id {
                    task.status = "completed".to_string();
                    task.progress = 100;
                    if let Some(tokens) = req.total_tokens {
                        task.tokens = tokens;
                    }
                    return Ok(stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\"}").await?);
                }
            }
            ("HTTP/1.1 404 Not Found", r#"{"error":"Task not found"}"#.to_string())
        } else {
            ("HTTP/1.1 400 Bad Request", r#"{"error":"Invalid request"}"#.to_string())
        }
    } else if request.contains("POST /api/task/error") {
        let body = extract_body(&request);
        if let Ok(req) = parse_json::<ErrorRequest>(&body) {
            let mut tasks = state.tasks.lock().unwrap();
            for task in tasks.iter_mut() {
                if task.id == req.task_id {
                    task.status = "error".to_string();
                    return Ok(stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\"}").await?);
                }
            }
            ("HTTP/1.1 404 Not Found", r#"{"error":"Task not found"}"#.to_string())
        } else {
            ("HTTP/1.1 400 Bad Request", r#"{"error":"Invalid request"}"#.to_string())
        }
    } else if request.contains("POST /api/task/cancel") {
        let body = extract_body(&request);
        if let Ok(req) = parse_json::<CancelRequest>(&body) {
            let mut tasks = state.tasks.lock().unwrap();
            tasks.retain(|t| t.id != req.task_id);
            let current_id = state.current_task_id.lock().unwrap();
            if *current_id == Some(req.task_id.clone()) {
                drop(current_id);
                *state.current_task_id.lock().unwrap() = None;
            }
            ("HTTP/1.1 200 OK", r#"{"status":"ok"}"#.to_string())
        } else {
            ("HTTP/1.1 400 Bad Request", r#"{"error":"Invalid request"}"#.to_string())
        }
    } else if request.contains("POST /api/reset") {
        let mut tasks = state.tasks.lock().unwrap();
        *tasks = Vec::new();
        *state.current_task_id.lock().unwrap() = None;
        ("HTTP/1.1 200 OK", r#"{"status":"ok"}"#.to_string())
    } else {
        ("HTTP/1.1 404 Not Found", r#"{"error":"Not found"}"#.to_string())
    };

    let response = format!(
        "{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        response.len(),
        response
    );
    stream.write_all(response.as_bytes()).await?;

    Ok(())
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
    println!("HTTP server listening on http://127.0.0.1:{}", port);

    loop {
        let (mut stream, _) = listener.accept().await?;
        let state = state.clone();
        spawn(async move {
            if let Err(e) = handle_connection(&mut stream, &state).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

pub fn start_server_background(port: u16) {
    spawn(async move {
        if let Err(e) = start_server(port).await {
            eprintln!("HTTP server error: {}", e);
        }
    });
}

pub fn get_state() -> Arc<SharedState> {
    Arc::new(SharedState::new())
}
