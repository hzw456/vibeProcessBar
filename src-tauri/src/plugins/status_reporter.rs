use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use axum::{
    routing::{post, get},
    Router,
    Json,
    extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use futures::StreamExt;
use tauri::{
    plugin::{Plugin, Result as PluginResult},
    AppHandle, Runtime, State as TauriState,
};

pub const DEFAULT_PORT: u16 = 31416;
pub const DEFAULT_HOST: &str = "127.0.0.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressStage {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "analysis")]
    Analysis,
    #[serde(rename = "coding")]
    Coding,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "testing")]
    Testing,
    #[serde(rename = "completed")]
    Completed,
}

impl From<&str> for ProgressStage {
    fn from(s: &str) -> Self {
        match s {
            "analysis" => ProgressStage::Analysis,
            "coding" => ProgressStage::Coding,
            "review" => ProgressStage::Review,
            "testing" => ProgressStage::Testing,
            "completed" => ProgressStage::Completed,
            _ => ProgressStage::Idle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub window_id: String,
    pub window_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    pub window_id: String,
    pub window_title: Option<String>,
    pub stage: String,
    pub progress: u8,
    pub message: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusEvent {
    pub window_id: String,
    pub window_title: Option<String>,
    pub stage: ProgressStage,
    pub progress: u8,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

struct ServerState {
    tx: broadcast::Sender<StatusEvent>,
    app_handle: AppHandle,
}

#[derive(Clone)]
pub struct StatusReporterPlugin {
    port: u16,
    host: String,
    tx: broadcast::Sender<StatusEvent>,
}

impl StatusReporterPlugin {
    pub fn new(port: u16, host: String) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { port, host, tx }
    }

    async fn start_server(&self, app_handle: AppHandle) -> Result<(), String> {
        let state = Arc::new(ServerState {
            tx: self.tx.clone(),
            app_handle: app_handle.clone(),
        });

        let addr: SocketAddr = format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|e| e.to_string())?;

        let app = Router::new()
            .route("/status", post(report_status))
            .route("/health", get(health_check))
            .with_state(state);

        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });

        Ok(())
    }
}

async fn report_status(
    State(state): State<Arc<ServerState>>,
    Json(report): Json<StatusReport>,
) -> Result<Json<()>, StatusCode> {
    let event = StatusEvent {
        window_id: report.window_id,
        window_title: report.window_title,
        stage: report.stage.as_str().into(),
        progress: report.progress,
        message: report.message,
        timestamp: chrono::Utc::now(),
    };

    let _ = state.tx.send(event);

    if let Some(window) = state.app_handle.get_webview_window("main") {
        let _ = window.emit("agent-status-update", &event);
    }

    Ok(Json(()))
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "status-reporter"
    })))
}

#[tauri::plugin]
impl<R: Runtime> Plugin<R> for StatusReporterPlugin {
    fn name(&self) -> &'static str {
        "status-reporter"
    }

    fn init(&self, app: &AppHandle<R>) -> PluginResult<Self> {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let _ = self.start_server(app_handle).await;
        });
        Ok(Self::new(self.port, self.host.clone()))
    }

    fn extensions(&self) -> tauri::webview::ExtensionBuilder<R> {
        let mut extensions = tauri::webview::ExtensionBuilder::new(self.name());
        extensions
    }
}

pub fn create_plugin(port: u16, host: String) -> StatusReporterPlugin {
    StatusReporterPlugin::new(port, host)
}
