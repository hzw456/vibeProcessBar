# 变更：重构为 HTTP API 悬浮窗

## 为什么

当前应用存在以下问题：
1. 悬浮窗无法正常显示
2. VSCode 集成过于复杂，需要维护扩展
3. 需要简化架构，让外部 IDE 通过 HTTP API 更新状态

通过改为 HTTP API 方式：
- 架构更简洁，无需维护 VSCode 扩展
- 任何 IDE 都可以通过简单的 HTTP 调用来更新状态
- 悬浮窗专注于显示和交互，状态由外部提供

## 什么变化

- **移除 VSCode 集成**：删除 VSCode 扩展相关代码和依赖
- **修复悬浮窗显示**：修复悬浮窗无法显示的问题
- **新增 HTTP API**：提供本地 HTTP 接口供外部工具调用
- **新增 Token 显示**：悬浮窗显示当前 token 数量
- **新增窗口跳转**：点击悬浮窗激活对应的 IDE 窗口

## 影响

- 删除的规格：vscode 能力
- 新增的规格：floating-window、http-api
- 受影响的代码：Tauri 后端、React 前端、VSCode 扩展
- 依赖：HTTP 服务器库
