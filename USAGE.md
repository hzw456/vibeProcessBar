# Vibe Coding Progress Bar 使用教程

## 功能概览

Vibe Coding Progress Bar 是一个悬浮在屏幕上的进度追踪工具，支持：
- 悬浮窗拖动（点击拖动到任意位置）
- 手动调节进度
- VSCode 集成（自动追踪 AI 编程进度）
- 多适配器支持（GitHub Copilot, Claude Code, Cursor 等）

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

## 常见问题

### Q: 悬浮窗无法拖动？
A: 检查是否启用了透明模式。确保在设置中正确配置。

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

进度栏与 VSCode 扩展通过 WebSocket 通信：

```
VSCode Extension <--WebSocket--> Vibe Progress Bar
Port: 31415 (默认)
```

消息格式：
```json
{
  "type": "status" | "progress" | "complete" | "error",
  "timestamp": "ISO-8601",
  "data": {
    "taskId": "string",
    "name": "string",
    "progress": number,
    "status": "idle" | "running" | "completed" | "error",
    "adapter": "copilot" | "claude-code" | "cursor"
  }
}
```

### 文件结构

```
vibeProcessBar/
├── src/
│   ├── components/      # React 组件
│   │   ├── ProgressBar.tsx
│   │   ├── SettingsPanel.tsx
│   │   └── StatusText.tsx
│   ├── stores/          # 状态管理
│   │   └── progressStore.ts
│   ├── utils/
│   │   └── adapters/    # AI 工具适配器
│   ├── App.tsx          # 主应用
│   └── main.tsx
├── src-tauri/           # Tauri 后端
│   ├── src/
│   │   └── main.rs
│   └── tauri.conf.json
└── openspec/            # 规格文档
```

## 反馈与支持

- 报告问题：https://github.com/your-repo/issues
- 功能建议：https://github.com/your-repo/discussions
