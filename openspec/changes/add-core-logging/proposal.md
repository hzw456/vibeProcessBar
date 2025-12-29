# add-core-logging

## 为什么

当前应用存在以下问题：
1. 仅有基本的 `println!` 和 `console.log` 日志，无法有效调试
2. 无法追踪任务从 HTTP API 到 UI 的完整流程
3. 难以定位 HTTP 请求失败或连接问题
4. 无法监控任务生命周期的时序和状态变化

通过添加结构化日志：
- 可以追踪外部 Agent 通过 HTTP API 到前端显示的完整数据流
- 便于定位任务同步失败、连接错误等问题
- 支持按模块和级别过滤日志输出
- 为后续问题排查提供可观测性基础
- 使用 `tracing` 库提供更强大的异步支持和 span 追踪能力

## 什么变化

- **Rust 后端**：新增 `tracing` + `tracing-subscriber` 依赖，在 http_server.rs 和 status_reporter.rs 中添加日志
- **TypeScript 前端**：新增 `src/utils/logger.ts` 工具，在 progressStore.ts、useProgressEvent.ts、App.tsx 中添加日志
- **日志级别**：trace（最详细）、debug、info、warn、error（最严重）
- **文档更新**：代码注释说明日志位置，API.md 和 USAGE.md 新增日志说明

## 影响

- 新增的规格：logging 能力
- 受影响的代码：src-tauri/src/http_server.rs、src-tauri/src/plugins/status_reporter.rs、src/stores/progressStore.ts、src/hooks/useProgressEvent.ts、src/App.tsx
- 依赖：tracing、tracing-subscriber（Rust）
