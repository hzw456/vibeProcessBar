## 1. 项目初始化和依赖准备

- [ ] 1.1 在 `src-tauri/Cargo.toml` 添加 `axum` 和 `tokio` 依赖
- [ ] 1.2 创建 Tauri 插件结构 `src-tauri/src/plugins/`
- [ ] 1.3 验证 Rust 编译通过

## 2. HTTP API 服务开发

- [ ] 2.1 创建 `StatusReporterPlugin` 插件结构
- [ ] 2.2 实现 `/status` POST 端点，接收状态上报 JSON
- [ ] 2.3 实现 `/health` GET 端点用于健康检查
- [ ] 2.4 实现事件转发机制，将状态变更发送到前端
- [ ] 2.5 配置 CORS 和本地地址绑定 (127.0.0.1:31416)
- [ ] 2.6 编写 API 集成测试

## 3. 窗口管理服务

- [ ] 3.1 创建 `window_manager.rs` 模块
- [ ] 3.2 实现 macOS 窗口激活（AppleScript）
- [ ] 3.3 实现 Windows 窗口激活（PowerShell/Win32）
- [ ] 3.4 实现 Linux 窗口激活（wmctrl）
- [ ] 3.5 暴露 `activate_window` Tauri 命令
- [ ] 3.6 编写窗口激活测试

## 4. 状态数据模型

- [ ] 4.1 定义 `StatusReport` 结构体（JSON 反序列化）
- [ ] 4.2 定义 `WindowInfo` 结构体
- [ ] 4.3 定义 `ProgressStage` 枚举
- [ ] 4.4 创建事件通道用于前后端通信

## 5. 前端状态展示组件

- [ ] 5.1 修改 `StatusText.tsx` 显示当前状态和消息
- [ ] 5.2 修改 `ProgressBar.tsx` 显示阶段进度
- [ ] 5.3 实现悬浮窗点击事件处理（调用 Rust 命令）
- [ ] 5.4 添加活跃窗口信息展示
- [ ] 5.5 编写前端组件测试

## 6. 状态存储集成

- [ ] 6.1 扩展 `progressStore.ts` 支持窗口 ID 和阶段信息
- [ ] 6.2 实现多任务状态管理
- [ ] 6.3 添加窗口激活历史记录

## 7. Prompt 模板生成

- [ ] 7.1 创建 Claude Code CLI 的 prompt 模板
- [ ] 7.2 创建 Continue 扩展的 prompt 模板
- [ ] 7.3 创建通用 Markdown 模板（适用于任意 AI 工具）
- [ ] 7.4 在前端 UI 中提供模板复制功能

## 8. 完整流程测试

- [ ] 8.1 启动 Tauri 应用，验证 API 服务启动
- [ ] 8.2 使用 curl 测试状态上报接口
- [ ] 8.3 验证前端状态更新
- [ ] 8.4 测试点击悬浮窗跳转到 VSCode 窗口
- [ ] 8.5 测试多阶段状态上报流程
- [ ] 8.6 验证编译通过（Debug 和 Release 模式）

## 9. 文档和示例

- [ ] 9.1 编写快速开始指南
- [ ] 9.2 编写 prompt 模板使用说明
- [ ] 9.3 录制演示视频（可选）
