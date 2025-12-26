# Design: Agent-Based Status Reporting Architecture

## Overview

采用 agent 模式实现状态上报的核心架构，包含三个主要组件：

1. **HTTP API 服务**：接收 AI 上报的状态和窗口 ID
2. **窗口管理服务**：Rust 后端处理窗口激活逻辑
3. **悬浮窗 UI**：显示状态并处理点击交互

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   AI Tool       │────▶│  HTTP API       │────▶│  Rust Backend   │
│ (Claude Code,   │     │  (Tauri Plugin) │     │  Window Manager │
│  Continue, etc) │     │  :31416/status  │     │                 │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                                                         ▼
                                                ┌─────────────────┐
                                                │  System Window  │
                                                │  (VSCode)       │
                                                └─────────────────┘
                                                         ▲
                                                         │
                                                ┌────────┴────────┐
                                                │  Floating UI    │
                                                │  (React)        │
                                                │  Click to jump  │
                                                └─────────────────┘
```

## Component Details

### 1. HTTP API Service (Tauri Plugin)

以 Tauri 插件形式运行，与主应用共享进程：

```rust
// src-tauri/src/plugins/status_reporter.rs

pub struct StatusReporterPlugin {
    port: u16,
    tx: Sender<StatusEvent>,
}

impl StatusReporterPlugin {
    pub fn new(port: u16) -> Self {
        let (tx, rx) = channel();
        Self { port, tx }
    }
}

#[tauri::plugin]
impl StatusReporterPlugin {
    // 启动 HTTP 服务器
    async fn start_server(&self) -> Result<(), ServerError> {
        let app = Router::new()
            .route("/status", Post(report_status))
            .route("/heartbeat", Get(heartbeat))
            .with_state(self.tx.clone());
        
        axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], self.port)))
            .serve(app.into_make_service())
            .await?;
        Ok(())
    }
}
```

**API 端点**：
- `POST /status` - 接收状态上报
- `GET /health` - 健康检查

**请求格式**：
```json
{
  "windowId": "vscode-12345",
  "windowTitle": "src/App.tsx - MyProject",
  "stage": "analysis" | "coding" | "review" | "testing" | "completed",
  "progress": 0-100,
  "message": "Analyzing code structure...",
  "timestamp": "2025-12-26T10:00:00Z"
}
```

### 2. Window Manager (Rust Backend)

处理窗口激活和跳转：

```rust
// src-tauri/src/window_manager.rs

#[tauri::command]
async fn activate_window(window_id: String) -> Result<(), String> {
    // macOS: AppleScript
    // Windows: Win32 API / PowerShell
    // Linux: xdotool / wmctrl
    
    #[cfg(target_os = "macos")]
    {
        let script = format!(r#"
            tell application "Visual Studio Code"
                activate
                tell application "System Events"
                    keystroke "`" using command down
                end tell
            end tell
        "#);
        std::process::Command::new("osascript")
            .args(&["-e", &script])
            .output()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
```

### 3. Floating UI (React Frontend)

显示当前状态和进度，点击时调用 Rust 后端：

```typescript
// src/components/StatusText.tsx (modified)

const handleClick = async () => {
  if (activeWindowId) {
    await invoke('activate_window', { windowId: activeWindowId });
  }
};

return (
  <div className="status-text" onClick={handleClick}>
    <span>{statusMessage}</span>
    <ProgressBar value={progress} />
  </div>
);
```

## Platform-Specific Window Activation

### macOS

```rust
#[cfg(target_os = "macos")]
fn activate_vscode(window_id: &str) -> Result<(), String> {
    let script = format!(
        r#"
        tell application "Visual Studio Code"
            activate
            delay 0.5
            tell application "System Events"
                keystroke "`" using command down
            end tell
        end tell
    "#);
    run_apple_script(&script)
}
```

### Windows

```rust
#[cfg(target_os = "windows")]
fn activate_vscode(window_id: &str) -> Result<(), String> {
    // 使用 PowerShell 激活窗口
    let output = std::process::Command::new("powershell")
        .args(&["-Command", &format!(r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('^{}');"`, "`")])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

### Linux

```rust
#[cfg(target_os = "linux")]
fn activate_vscode(window_id: &str) -> Result<(), String> {
    std::process::Command::new("wmctrl")
        .args(&["-a", "Visual Studio Code"])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

## Prompt Template Design

为不同 AI 工具提供可注入的 prompt 片段：

### Claude Code CLI

```markdown
## Status Reporting Protocol

After completing each major stage of the task, you MUST report status by calling:

```
curl -X POST http://127.0.0.1:31416/status \
  -H "Content-Type: application/json" \
  -d '{
    "windowId": "vscode-$(ps aux | grep -o 'code [0-9]*' | head -1)",
    "windowTitle": "$(code --list-extensions 2>/dev/null | head -1 || echo \"VSCode\")",
    "stage": "coding",
    "progress": 50,
    "message": "Writing component logic..."
  }'
```

Stages: analysis (10%), coding (40%), review (70%), testing (90%), completed (100%)
```

### Continue Extension

```typescript
// Continue 扩展的 prompt 注入配置
const statusReportingPrompt = `
## Status Reporting

After each code generation phase, report status:
\`\`\`bash
curl -X POST http://localhost:31416/status \
  -H "Content-Type: application/json" \
  -d '{"windowId": "${windowId}", "stage": "coding", "progress": 50, "message": "..."}'
\`\`\`
`;
```

## Trade-offs

| 方面 | Agent 模式 | 传统扩展模式 |
|------|-----------|-------------|
| 实现复杂度 | 低（只需 API） | 高（多平台扩展） |
| 兼容性 | 通用 | 依赖特定 API |
| 精度 | 精确（主动上报） | 依赖检测逻辑 |
| 用户配置 | 需配置 prompt | 一键安装 |
| 覆盖范围 | 任意 AI 工具 | 仅 IDE 扩展支持 |

## Open Questions

1. **窗口 ID 生成**：如何唯一标识 VSCode 窗口？（考虑使用进程 PID + 窗口标题）
2. **多窗口场景**：当多个 VSCode 窗口打开时，如何确定目标窗口？
3. **超时处理**：AI 未按时上报时如何处理？
4. **安全考虑**：HTTP API 是否需要认证？
