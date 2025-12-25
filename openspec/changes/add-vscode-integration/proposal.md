# Change: Add VSCode Integration Capability

## Why

Vibe Coding 进度栏目前需要支持 VSCode 作为主要的 IDE 集成目标。VSCode 是最流行的代码编辑器，拥有最大的 AI 编程扩展生态系统（GitHub Copilot、Continue、Claude Code 等）。通过集成 VSCode，可以：

- 扩展进度栏的适用范围，覆盖更多 AI 辅助编程用户
- 通过 VSCode 扩展 API 获取 AI 任务状态，实现精确的进度监控
- 与现有的 AI 编程工具生态无缝集成

## What Changes

- **新增 VSCode 能力规格**：定义与 VSCode 集成的技术规范和接口
- **新增 VSCode 扩展集成**：支持通过 VSCode 扩展获取 AI 任务状态
- **新增窗口/项目检测**：自动检测当前 VSCode 窗口和打开的项目
- **新增状态同步**：实时同步 VSCode 中的 AI 任务状态到进度栏
- **支持多工作区**：处理 VSCode 多工作区场景

## Impact

- Affected specs: 新增 `vscode` 能力
- Affected code: Tauri 后端、React 前端、可能的 VSCode 扩展
- 依赖：VSCode Extension API、VSCode Webview API
