# Project Context

## Purpose

Vibe Coding 进度栏是一款专为 AI 辅助编程设计的桌面工具。产品以悬浮窗形式实时监控 AI 代码生成进度，支持窗口快速切换、进度可视化提醒，让开发者在 AI 执行任务时可以进行其他工作，大幅提升开发效率和用户体验。

## Tech Stack

- **框架：** Tauri（Rust + Web 前端）
- **前端：** React + TypeScript
- **构建工具：** Vite
- **目标平台：** macOS, Windows, Linux

## Project Conventions

### Code Style

- TypeScript 严格模式
- ESLint + Prettier 代码格式化
- 组件采用函数式 + Hooks 风格
- 命名使用 camelCase（变量/函数）和 PascalCase（组件）

### Architecture Patterns

- 插件化架构（适配器模式）
- 状态管理：React Context + Zustand
- 进程间通信：Tauri Rust ↔ React
- 事件驱动设计

### Testing Strategy

- 单元测试：Vitest + React Testing Library
- 集成测试：Playwright（E2E）
- 跨平台测试：GitHub Actions CI

### Git Workflow

- 主分支：main（生产就绪）
- 开发分支：develop
- 功能分支：feature/*、fix/*、refactor/*
- 提交信息：Conventional Commits（feat/、fix/、docs/ 等）

## Domain Context

- AI 辅助编程工具生态（GitHub Copilot, Claude Code, Cursor, Continue 等）
- 跨平台桌面应用开发
- 悬浮窗和窗口管理
- 系统通知和托盘集成

## Important Constraints

- 资源占用轻量（内存 < 50MB）
- 启动时间 < 3 秒
- 支持 macOS、Windows、Linux 三大平台
- 悬浮窗需支持透明和置顶

## External Dependencies

- Tauri 框架及生态系统
- React 生态（组件库：Shadcn UI 或 Radix UI）
- 系统通知 API（平台原生）
- AI 编程工具的 API 和扩展机制
