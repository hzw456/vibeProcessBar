## ADDED Requirements

### Requirement: Structured Logging Foundation

应用 SHALL 在 Rust 后端和 TypeScript 前端提供结构化日志能力。

Rust 后端 SHALL 使用 `tracing` crate 配合 `tracing-subscriber`。

TypeScript 前端 SHALL 使用位于 `src/utils/logger.ts` 的集中式日志工具。

#### Scenario: Rust 后端日志

- **WHEN** Rust 后端集成 `tracing` crate 并使用 `tracing-subscriber` 初始化
- **THEN** 应用 SHALL 支持日志级别：trace、debug、info、warn、error
- **AND** 日志输出 SHALL 包含时间戳、级别、模块路径、消息
- **AND** 支持通过 `#[tracing::instrument]` 注解自动追踪函数调用

#### Scenario: 前端日志工具

- **WHEN** TypeScript 前端加载并导入 `src/utils/logger.ts`
- **THEN** 应用 SHALL 提供一致的日志 API，级别包括：debug、info、warn、error

### Requirement: HTTP Server Logging

HTTP API 服务器 SHALL 记录所有传入请求和状态变更。

#### Scenario: 请求日志

- **WHEN** 外部 Agent 发送 HTTP 请求到进度 API，请求被 `http_server.rs` 接收
- **THEN** 应用 SHALL 记录 HTTP 方法
- **AND** SHALL 记录请求路径
- **AND** SHALL 记录请求体摘要（前 100 字符用于调试）
- **AND** SHALL 记录时间戳

#### Scenario: 状态变更日志

- **WHEN** 通过 HTTP API 发生任务状态变更，任务被创建、更新、完成或出错
- **THEN** 应用 SHALL 记录操作类型
- **AND** SHALL 记录任务标识符

#### Scenario: 连接错误日志

- **WHEN** HTTP 服务器发生连接错误，错误被捕获
- **THEN** 应用 SHALL 在 error 级别记录错误详情

### Requirement: Status Reporter Logging

状态报告器插件 SHALL 记录状态报告和事件广播。

#### Scenario: 状态报告日志

- **WHEN** `status_reporter.rs` 收到状态报告并处理
- **THEN** 应用 SHALL 记录报告类型
- **AND** SHALL 记录内容摘要

#### Scenario: 事件广播日志

- **WHEN** 事件广播到前端，广播被尝试
- **THEN** 应用 SHALL 记录事件名称
- **AND** SHALL 记录广播结果

### Requirement: Frontend Store Logging

进度 Store SHALL 记录同步和状态变更。

#### Scenario: HTTP 同步日志

- **WHEN** 进度 Store 轮询 HTTP API，同步完成或失败
- **THEN** 应用 SHALL 记录同步结果
- **AND** SHALL 记录任务数量

#### Scenario: 任务变更日志

- **WHEN** 任务被添加到 Store、更新或移除，变更发生
- **THEN** 应用 SHALL 记录操作
- **AND** SHALL 记录任务标识符

### Requirement: Event Hook Logging

进度事件钩子 SHALL 记录事件发射。

#### Scenario: 事件发射日志

- **WHEN** 通过 `useProgressEvent` 发射事件，事件被发布
- **THEN** 应用 SHALL 记录事件类型
- **AND** SHALL 记录负载摘要

#### Scenario: 处理器调用日志

- **WHEN** 事件处理器被调用，处理器执行
- **THEN** 应用 SHALL 在 debug 级别记录处理器执行

### Requirement: Log Configuration

日志系统 SHALL 支持运行时配置。

#### Scenario: Rust 日志级别配置

- **WHEN** 应用运行且设置了 `RUST_LOG` 环境变量
- **THEN** 应用 SHALL 根据指定级别过滤日志

#### Scenario: 默认日志级别

- **WHEN** 未提供日志配置，应用启动
- **THEN** Rust 后端 SHALL 默认为 info 级别
- **AND** TypeScript 前端 SHALL 默认为 info 级别

### Requirement: Log Documentation

日志点 SHALL 记录在文档中以便调试。

#### Scenario: 日志位置文档

- **WHEN** 开发者需要调试应用并查阅文档
- **THEN** 他们 SHALL 找到日志打印位置的清晰说明
- **AND** SHALL 找到每个代码路径适用的日志级别

#### Scenario: API 文档更新

- **WHEN** 日志系统实现，API.md 更新
- **THEN** 它 SHALL 记录日志端点和日志级别
