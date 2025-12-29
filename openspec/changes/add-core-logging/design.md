# add-core-logging Design

## 日志策略

### Rust 后端

**选择：`tracing`  crate + `tracing-subscriber`**

理由：
- 原生支持异步代码（Tokio 生态）
- 提供 span 机制，可追踪代码执行路径
- 自动记录函数调用层级关系
- `tracing-subscriber` 提供多种格式化输出
- 相比 `log` + `env_logger` 功能更强大

**http_server.rs 日志点：**
```
1. Server startup - info level
2. HTTP request received - debug level (method, path, body summary)
3. Task state mutations - info level (create, update, complete, error)
4. Connection errors - error level
5. Shutdown signals - info level
```

**status_reporter.rs 日志点：**
```
1. Status report received - debug level
2. Event broadcast - debug level
3. Errors - error level
```

### TypeScript 前端

**选择：自定义 logger 工具，包装 console**

理由：
- 避免为前端添加外部日志依赖
- 集中控制，便于后续扩展（localStorage、远程上报）
- API 一致，易于使用
- 未来可轻松添加日志级别过滤

**Logger API：**
```typescript
// src/utils/logger.ts
export function debug(message: string, data?: object): void
export function info(message: string, data?: object): void
export function warn(message: string, data?: object): void
export function error(message: string, data?: object): void
```

**日志点：**
```
1. progressStore.ts: HTTP sync operations, task mutations
2. useProgressEvent.ts: Event emissions and handler invocations
3. App.tsx: Task selection, IDE activation, window operations
4. notifications.ts: Notification results
5. windowManager.ts: Window operations
```

## 日志级别约定

| 级别 | 用途 |
|------|------|
| `trace` | 最详细的执行路径追踪（tracing 特有） |
| `debug` | 详细流程，请求/响应体 |
| `info` | 关键状态变化，服务生命周期事件 |
| `warn` | 可恢复问题，非致命异常状态 |
| `error` | 失败、异常、连接问题 |

## 日志格式

**Rust：**
```
2024-01-01T12:00:00.000Z INFO  vibeprocessbar::http_server: Server listening on 0.0.0.0:31415
2024-01-01T12:00:01.000Z DEBUG vibeprocessbar::http_server: POST /task/start: task_id=abc method=POST
```

**TypeScript：**
```json
{"timestamp":"2024-01-01T12:00:00Z","level":"info","message":"HTTP sync completed","data":{"tasks":5}}
```

## 配置方式

**Rust：** 通过 `RUST_LOG` 环境变量
```
RUST_LOG=debug              # 所有 debug 及以上
RUST_LOG=info               # info 及以上
RUST_LOG=vibeprocessbar::http_server=trace  # 模块特定
RUST_LOG=trace              # 最详细（包括 trace）
```

**TypeScript：** 编译时或 localStorage 标志（未来扩展）

## 使用 tracing span 示例

```rust
#[tracing::instrument(skip(request))]
async fn handle_task_start(request: Request) -> Result<Response> {
    let task_id = request.task_id.clone();
    tracing::info!(task_id = %task_id, "Starting new task");
    // ... 处理逻辑
    tracing::debug!(task_id = %task_id, progress = 0, "Task started");
    Ok(Response::ok())
}
```

## 权衡

| 决策 | 优点 | 缺点 |
|------|------|------|
| `tracing` 库 | 异步支持，span 追踪，功能强大 | 比 `log` 稍重 |
| 无日志持久化 | 简单，无磁盘 I/O | 无历史日志 |
| 基于 console 的前端 | 无依赖，易于使用 | 无结构化输出 |
| 同步日志 | 简单可靠 | 轻微性能影响 |

## 未来扩展性

当前设计支持未来增强：
- 添加日志持久化（文件滚动）
- 添加远程日志上报端点
- 添加日志查看 UI 组件
- 添加日志级别动态调整 API
