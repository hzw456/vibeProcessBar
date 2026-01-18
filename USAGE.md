# Vibe Coding Progress Bar 使用教程

## 功能概览

Vibe Coding Progress Bar 是一个悬浮在屏幕上的进度追踪工具，支持：
- 悬浮窗拖动（点击拖动到任意位置）
- 手动调节进度
- VSCode 集成（自动追踪 AI 编程进度）
- 多适配器支持（GitHub Copilot, Claude Code, Cursor 等）
- **双击任务可跳转到对应 IDE 窗口**（需要 macOS 辅助功能权限）

---

## 三种集成模式

Vibe Process Bar 提供三种灵活的状态检测机制：

### 1. Hook 模式（推荐用于 Kiro）

通过 IDE 的 Hook 机制精准同步状态。

**配置步骤：**

1. 将 `rules/on-agent-start.sh` 和 `rules/on-agent-complete.sh` 复制到 `.kiro/hooks/` 目录
2. 确保脚本有执行权限：`chmod +x .kiro/hooks/*.sh`
3. 在 `.kiro/steering/` 中添加 `vibe-hook-mode.md` 规则文件

**Hook 脚本工作原理：**
- `on-agent-start.sh`：AI Agent 开始时自动调用，设置状态为 `running`
- `on-agent-complete.sh`：AI Agent 完成时自动调用，设置状态为 `completed`

**支持的 IDE：** Kiro, Cursor, Windsurf, Trae, CodeBuddy, Antigravity

### 2. MCP 模式（推荐用于 Cursor/Cline/RooCode）

通过 Model Context Protocol 标准直接连接。

**配置步骤：**

1. 在 IDE 的 MCP 配置文件中添加：
```json
{
  "mcpServers": {
    "vibe-process-bar": {
      "url": "http://127.0.0.1:31415/mcp"
    }
  }
}
```

2. 将 `rules/rules.md` 的内容添加到 AI 助手的系统提示或规则配置中

**MCP 提供的工具：**
- `list_tasks`：获取当前活动任务列表
- `update_task_status`：报告任务状态（running, completed, error）
- `update_task_progress`：更新任务进度和当前阶段

### 3. 插件检测模式（代码检测）

通过 VS Code 扩展分析代码变化频率。

**配置步骤：**
1. 安装 [Vibe Process Bar VS Code Extension](https://github.com/hzw456/vibeProcessBarVSCodeExt)
2. 扩展会自动监测文件修改速度来推断 AI 是否在生成代码

> ⚠️ 此方法准确度较低，仅推荐在无法使用 MCP 或 Hook 时作为补充。

---

## 基础操作

### 拖动悬浮窗
- 在悬浮窗上 **单击并拖动** 即可移动位置
- 悬浮窗会自动保持在顶层

### 手动控制进度
- **点击进度条** 直接跳转到指定进度
- **双击悬浮窗** 重置当前任务

### 打开设置
- 右键点击系统托盘图标 → Settings
- 或使用快捷键（在托盘菜单中查看）

## VSCode 集成配置

### 步骤 1：安装 VSCode 扩展

在 VSCode 中安装 Vibe Coding Progress Bar 扩展：

1. 打开 VSCode
2. 按 `Cmd+Shift+X` 打开扩展面板
3. 搜索 "Vibe Coding Progress Bar"
4. 点击安装

### 步骤 2：配置进度栏

1. 右键点击系统托盘图标 → Settings
2. 切换到 **VSCode** 标签
3. 启用 "Enable VSCode Integration"
4. 配置连接参数（一般保持默认）：
   - Host: `localhost`
   - Port: `31415`

### 步骤 3：验证连接

1. 确保进度栏应用正在运行
2. 在 VSCode 中使用 AI 编程工具（如 Copilot、Claude Code）
3. 进度栏应显示正在进行的 AI 任务

## 支持的 AI 编程工具

| 工具 | 支持状态 | 说明 |
|------|----------|------|
| GitHub Copilot | ✅ | 自动追踪代码建议生成 |
| Claude Code | ✅ | 自动追踪 Claude 请求 |
| Continue | ✅ | 自动追踪 AI 任务 |
| Cursor | ✅ | 自动追踪 AI 编程 |
| CLI | ✅ | 手动启动任务 |

---

## IDE 配置指南

### Kiro 配置（Hook 模式）

1. 创建 `.kiro/hooks/` 目录
2. 复制 Hook 脚本：
```bash
cp rules/on-agent-start.sh .kiro/hooks/
cp rules/on-agent-complete.sh .kiro/hooks/
chmod +x .kiro/hooks/*.sh
```
3. 创建 `.kiro/steering/vibe-hook-mode.md` 添加 AI 规则

### Cursor 配置（MCP 模式）

1. 在 `.cursor/mcp.json` 中添加：
```json
{
  "mcpServers": {
    "vibe-process-bar": {
      "url": "http://127.0.0.1:31415/mcp"
    }
  }
}
```
2. 在 `.cursorrules` 中添加 `rules/rules.md` 的内容

### VS Code 配置（插件模式）

1. 安装 Vibe Process Bar VS Code Extension
2. 扩展会自动连接到进度栏应用

---

## 常见问题

### Q: 悬浮窗无法拖动？
A: 检查是否启用了透明模式。确保在设置中正确配置。

### Q: Hook 模式不生效？
A: 检查以下几点：
1. Hook 脚本是否有执行权限（`chmod +x`）
2. 进度栏应用是否正在运行
3. 脚本路径是否正确（`.kiro/hooks/` 目录）

### Q: MCP 模式无法连接？
A: 检查以下几点：
1. MCP 配置文件格式是否正确
2. URL 是否为 `http://127.0.0.1:31415/mcp`
3. 进度栏应用是否正在运行

### Q: VSCode 扩展无法连接？
A: 检查以下几点：
1. 进度栏应用是否正在运行
2. 端口是否正确（默认 31415）
3. 防火墙是否阻止了连接
4. 尝试重启进度栏应用

### Q: 如何查看日志？
A: 在终端运行：
```bash
# 开发模式日志
npm run dev

# 查看应用日志（macOS）
log show --predicate 'process == "vibe-process-bar"' --info
```

### Q: 如何卸载应用？
A:
1. 关闭应用
2. 删除应用文件
3. 清理配置：`rm -rf ~/Library/Application\ Support/com.vibe.processbar`

## 技术架构

### 通信协议

进度栏支持多种通信方式：

**HTTP API（端口 31415）：**
```
POST /api/task/update_state_by_path  # Hook 模式使用
POST /mcp                             # MCP 模式使用
```

**WebSocket（端口 31415）：**
```
VSCode Extension <--WebSocket--> Vibe Progress Bar
```

### API 请求格式

**Hook 模式请求：**
```json
{
  "project_path": "/path/to/project",
  "ide": "kiro",
  "status": "running",
  "source": "hook"
}
```

**MCP 模式请求：**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "update_task_status",
    "arguments": {
      "task_id": "xxx",
      "status": "running"
    }
  }
}
```

### 文件结构

```
vibeProcessBar/
├── rules/
│   ├── on-agent-start.sh    # Hook 脚本：AI 开始时调用
│   ├── on-agent-complete.sh # Hook 脚本：AI 完成时调用
│   ├── rules.md             # MCP 模式的 AI 规则
│   ├── vibe-hook-mode.md    # Hook 模式的 AI 规则
│   └── vibe-mcp-mode.md     # MCP 模式的 AI 规则
├── src/                     # Vue 前端源码
├── src-tauri/               # Tauri/Rust 后端源码
└── ...
```

## 反馈与支持

- 报告问题：https://github.com/your-repo/issues
- 功能建议：https://github.com/your-repo/discussions
