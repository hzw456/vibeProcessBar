# Change: Add Agent-Based Status Reporting Capability

## Why

当前的 VSCode 集成方案需要开发 VSCode 扩展，通过监听 AI 扩展 API 来检测任务状态。这种方式存在以下局限：

1. **依赖性强**：需要为每个 AI 工具（Copilot、Claude Code、Continue 等）单独开发适配器
2. **检测不精确**：被动监听方式难以准确判断 AI 任务的开始、阶段切换和完成
3. **覆盖不全**：无法覆盖 CLI 模式的 AI 编程工具（如 Claude Code CLI）

用户提出的 agent 模式方案通过在 prompt 中注入上报指令，让 AI 主动上报状态，具有以下优势：

- **主动式**：AI 完成任务阶段后主动调用 API，上报时机精确
- **通用性强**：不依赖特定 IDE 或 AI 工具的实现细节
- **架构简洁**：只需提供 HTTP API，无需开发各平台扩展
- **可扩展**：支持任意支持自定义 prompt 的 AI 工具

## What Changes

- **新增 Agent 状态上报 API**：提供 HTTP 接口接收 AI 上报的状态和窗口 ID
- **新增窗口管理能力**：Rust 后端处理窗口激活和跳转逻辑
- **新增悬浮窗交互**：点击悬浮窗激活对应窗口
- **新增 prompt 模板生成**：为常见 AI 工具生成可注入的 prompt 片段
- **优先实现 VSCode**：验证完整流程后扩展到其他 IDE

## Impact

- Affected specs: 新增 `vscode-agent` 能力
- Affected code: Tauri 后端（Rust）、React 前端、HTTP API 服务
- 依赖：系统窗口管理 API、HTTP 服务器能力
- 风险：需要用户配置 AI 工具的 prompt，上报依赖 AI 的配合

## Out of Scope

- 多 IDE 支持（本阶段只实现 VSCode）
- 高级窗口管理（多标签页、分割窗口等）
- 状态持久化和历史记录
- 自动 prompt 注入工具
