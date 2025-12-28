## ADDED Requirements

### Requirement: HTTP API 服务器

系统 SHALL 提供本地 HTTP API 服务器，用于接收外部工具的状态更新。

服务器 SHALL 仅接受本地连接（127.0.0.1）。

服务器 SHALL 默认监听端口 31415。

#### Scenario: 服务器启动

- **WHEN** 应用程序启动
- **THEN** HTTP 服务器在 127.0.0.1:31415 启动
- **AND** 服务器准备好接收请求
- **AND** 如果端口被占用，尝试下一个端口

#### Scenario: 服务器关闭

- **WHEN** 应用程序关闭
- **THEN** HTTP 服务器正确关闭
- **AND** 所有挂起的请求完成或超时

#### Scenario: 连接验证

- **WHEN** 外部工具发送请求
- **THEN** 服务器验证请求来源为本地
- **AND** 拒绝非本地连接

### Requirement: 任务管理 API

系统 SHALL 提供任务生命周期管理的 HTTP API。

#### Scenario: 开始新任务

- **WHEN** POST /api/task/start
```json
{
  "taskId": "uuid",
  "name": "生成用户模块代码",
  "ide": "cursor",
  "windowTitle": "Cursor - user.ts"
}
```
- **THEN** 系统创建新任务
- **AND** 任务状态设为 running
- **AND** 进度设为 0
- **AND** Token 计数器重置
- **AND** 返回任务 ID

#### Scenario: 更新进度

- **WHEN** POST /api/task/progress
```json
{
  "taskId": "uuid",
  "progress": 50
}
```
- **THEN** 系统更新任务进度
- **AND** 悬浮窗显示新进度
- **AND** 进度值范围 0-100

#### Scenario: 更新 Token 数量

- **WHEN** POST /api/task/token
```json
{
  "taskId": "uuid",
  "tokens": 1500,
  "increment": true
}
```
- **THEN** 系统更新 token 数量
- **AND** 如果 increment 为 true，则累加
- **AND** 如果 increment 为 false，则设置为绝对值
- **AND** 悬浮窗显示更新后的 token 数

#### Scenario: 完成任务

- **WHEN** POST /api/task/complete
```json
{
  "taskId": "uuid",
  "totalTokens": 5000
}
```
- **THEN** 系统标记任务为完成
- **AND** 进度设为 100%
- **AND** Token 数量设为最终值
- **AND** 悬浮窗显示完成状态

#### Scenario: 任务错误

- **WHEN** POST /api/task/error
```json
{
  "taskId": "uuid",
  "message": "请求超时"
}
```
- **THEN** 系统标记任务为错误
- **AND** 状态文本显示错误信息
- **AND** 悬浮窗显示错误状态

#### Scenario: 取消任务

- **WHEN** POST /api/task/cancel
```json
{
  "taskId": "uuid"
}
```
- **THEN** 系统取消任务
- **AND** 任务从列表中移除或标记为已取消
- **AND** 悬浮窗返回到空闲状态

### Requirement: 状态查询 API

系统 SHALL 提供状态查询接口。

#### Scenario: 获取当前状态

- **WHEN** GET /api/status
- **THEN** 系统返回当前状态
```json
{
  "currentTask": {
    "id": "uuid",
    "name": "生成用户模块代码",
    "progress": 50,
    "tokens": 2500,
    "status": "running",
    "ide": "cursor",
    "windowTitle": "Cursor - user.ts"
  },
  "tasks": [...]
}
```

### Requirement: 窗口激活 API

系统 SHALL 提供窗口激活接口。

#### Scenario: 激活窗口

- **WHEN** POST /api/window/activate
```json
{
  "taskId": "uuid"
}
```
- **THEN** 系统激活任务关联的 IDE 窗口
- **AND** 窗口移到前台
- **AND** 返回激活结果

#### Scenario: 手动激活

- **WHEN** POST /api/window/activate
```json
{
  "ide": "cursor",
  "windowTitle": "user.ts"
}
```
- **THEN** 系统通过 IDE 类型或窗口标题查找窗口
- **AND** 激活匹配的窗口

### Requirement: API 错误处理

系统 SHALL 提供清晰的错误处理机制。

#### Scenario: 无效请求

- **WHEN** 收到格式错误的请求
- **THEN** 系统返回 400 Bad Request
- **AND** 返回错误描述

#### Scenario: 任务不存在

- **WHEN** 引用不存在的任务 ID
- **THEN** 系统返回 404 Not Found
- **AND** 返回任务未找到错误

#### Scenario: 服务器错误

- **WHEN** 服务器内部错误
- **THEN** 系统返回 500 Internal Server Error
- **AND** 返回错误日志

### Requirement: API 安全

系统 SHALL 确保 API 仅限本地调用。

#### Scenario: 来源验证

- **WHEN** 收到 HTTP 请求
- **THEN** 服务器验证 RemoteAddr 为 127.0.0.1 或 ::1
- **AND** 拒绝其他来源的请求

#### Scenario: 无认证

- **WHEN** 用户配置
- **THEN** API 不需要认证
- **AND** 但仅限于本地调用
