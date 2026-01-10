# Vibe Coding Progress Bar

A floating progress bar for AI-assisted coding, built with Tauri and React.

## Prerequisites

- **Rust** (1.70.0 or later): https://rustup.rs/
- **Node.js** (18.0 or later): https://nodejs.org/
- **pnpm** (recommended) or npm

## Installation

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install Node.js (if not installed):
   ```bash
   # Using nvm (recommended)
   nvm install 20
   nvm use 20
   
   # Or download from https://nodejs.org/
   ```

3. Install dependencies:
   ```bash
   npm install
   ```

4. Install Tauri CLI:
   ```bash
   npm install -D @tauri/cli
   ```

## Development

Start development server:
```bash
npm run tauri dev
```

## Build

Build for current platform:
```bash
npm run tauri build
```

Build for all platforms:
```bash
npm run tauri build -- --target universal-apple-darwin  # macOS
npm run tauri build --target x86_64-unknown-linux-gnu   # Linux
npm run tauri build --target x86_64-pc-windows-msvc     # Windows
```

## Project Structure

```
vibeProcessBar/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── stores/             # Zustand state management
│   ├── hooks/              # Custom React hooks
│   ├── utils/              # Utility functions
│   └── App.tsx             # Main app component
├── src-tauri/              # Rust backend
│   ├── src/main.rs         # Tauri entry point
│   ├── tauri.conf.json     # Tauri configuration
│   └── Cargo.toml          # Rust dependencies
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## Features

- Floating window with transparent background
- Circular progress indicator
- Drag to reposition
- Double-click to reset
- Zustand state management with persistence
- Theme support
- Window position persistence
- **MCP (Model Context Protocol) support** for AI task status reporting

## MCP Configuration

Vibe Process Bar 提供 MCP 服务，让 AI 编程助手可以上报任务状态。

### 配置方法

在你的 AI 客户端的 MCP 配置文件中添加（如 `mcp_config.json`）：

```json
{
  "mcpServers": {
    "vibe-process-bar": {
      "url": "http://127.0.0.1:31415/mcp"
    }
  }
}
```

### 可用工具

| 工具 | 描述 |
|------|------|
| `list_tasks` | 获取所有任务列表（ID、IDE、workspace、状态等） |
| `update_task_status` | 更新任务状态 |

### AI 使用指南

MCP 服务会自动向 AI 发送以下使用说明：

```
1. 任务开始时：调用 update_task_status(task_id, "running")
2. 任务完成时：调用 update_task_status(task_id, "completed")
3. 任务出错时：调用 update_task_status(task_id, "error")

task_id 格式为 "{ide}_{workspace名}"，例如 "antigravity_myproject"
可以先调用 list_tasks 获取任务列表
```

### 状态值

| 状态 | 说明 |
|------|------|
| `running` | 进行中 |
| `completed` | 已完成 |
| `error` | 出错 |
| `cancelled` | 已取消 |
| `armed` | 待命 |
| `active` | 活跃 |

### 示例请求

```bash
# 初始化
curl -X POST http://127.0.0.1:31415/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'

# 列出所有任务
curl -X POST http://127.0.0.1:31415/mcp \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_tasks"}}'

# 更新任务状态
curl -X POST http://127.0.0.1:31415/mcp \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"update_task_status","arguments":{"task_id":"antigravity_myproject","status":"completed"}}}'
```

## License

MIT
