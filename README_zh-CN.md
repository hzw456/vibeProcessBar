# Vibe Process Bar

<p align="center">
  <a href="README.md">English</a> | <a href="README_zh-CN.md">简体中文</a>
</p>

<p align="center">
  <img src="./src-tauri/icons/512x512.png" alt="Vibe Process Bar Logo" width="128" height="128">
</p>

<p align="center">
  <strong>让 AI 编程助手的工作状态一目了然</strong>
</p>

<p align="center">
  一款专为 AI 辅助编程设计的桌面悬浮进度条工具，实时可视化 AI Agent 的工作状态。
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-blue" alt="Platform">
  <img src="https://img.shields.io/badge/built%20with-Tauri%20%2B%20Vue-green" alt="Built with">
  <img src="https://img.shields.io/badge/license-GPLv3-orange" alt="License">
</p>

---

## 为什么需要 Vibe Process Bar？

当你使用 Cursor、Kiro、Claude 等 AI 编程助手时，是否经常遇到这些困扰？

- 🤔 **不知道 AI 在干什么** — AI 在后台思考，你只能干等
- ⏰ **不确定任务何时完成** — 切换窗口后就失去了进度感知
- 🔄 **多任务难以追踪** — 同时运行多个 AI 任务时容易混乱
- 🪟 **窗口切换麻烦** — 打开多个应用后很难找到对应的 IDE 窗口

**Vibe Process Bar** 解决了这些问题！它是一个轻量级的悬浮窗口，始终显示在屏幕顶层，让你随时掌握 AI 的工作状态。**双击任意任务即可瞬间跳转到对应的 IDE 窗口** — 再也不用在任务栏里翻找了！

> ⚠️ **注意**：窗口跳转功能需要 macOS 辅助功能权限。请在 系统设置 → 隐私与安全性 → 辅助功能 中授权。

---

## 📸 截图展示

| 运行中 | 已完成 | 监控面板 |
|:---:|:---:|:---:|
| ![运行中](./docs/images/runing.png) | ![已完成](./docs/images/complete.png) | ![监控面板](./docs/images/monitor.png) |

---

## ✨ 核心功能

### 🎯 实时状态追踪

| 状态 | 图标 | 含义 |
|:---:|:---:|:---|
| **Armed** | ◎ | 待命中，准备开始 |
| **Running** | ◉ | AI 正在工作，动态进度条 |
| **Completed** | ✓ | 任务完成 |
| **Focused** | 👁 | 窗口获得焦点 |


### 🖥️ 多 IDE 支持

全面支持主流 AI 编程工具与 IDE：

- **VS Code** (包括各衍生版本)
    - 支持所有主流 AI 插件：GitHub Copilot, Cline, RooCode, Claude Code 等
- **Cursor** (原生支持)
- **Kiro** (原生支持)
- **Windsurf** (原生支持)
- **Antigravity** (原生支持)
- **Trae** (原生支持)
- **CodeBuddy / CodeBuddy CN** (原生支持)

### 🎨 优雅的悬浮设计

- 半透明毛玻璃效果，不遮挡工作区
- 支持拖拽移动到任意位置
- **双击跳转到对应任务窗口**（需要 macOS 辅助功能权限）
- 自动置顶，始终可见

### 📊 多任务管理

- 同时追踪多个 AI 任务
- 点击切换查看不同任务
- 独立显示每个任务的进度和状态

---

## 🚀 快速开始

### ⚠️ 前置条件

**必须安装 VS Code 插件才能正常使用 Vibe Process Bar：**

👉 **[Vibe Process Bar VS Code 插件](https://github.com/hzw456/vibeProcessBarVSCodeExt)**

桌面应用通过 VS Code 插件来检测 AI 编程活动。没有安装插件，进度条将无法正常工作。

---

### 方式一：下载预编译版本（推荐）

前往 [Releases](https://github.com/hzw456/vibeProcessBar/releases) 页面下载：

| 系统 | 下载 |
|:---|:---|
| macOS (Apple Silicon) | `VibeProcessBar_x.x.x_aarch64.dmg` |
| macOS (Intel) | `VibeProcessBar_x.x.x_x64.dmg` |

### 方式二：从源码构建

```bash
# 1. 克隆项目
git clone https://github.com/hzw456/vibeProcessBar.git
cd vibeProcessBar

# 2. 安装依赖
npm install

# 3. 开发模式运行
npm run tauri dev

# 4. 构建生产版本
npm run tauri build
```

**系统要求：**
- macOS
- Node.js 18+
- Rust 1.70+
- Tauri CLI (`npm install -D @tauri-apps/cli`)

---

## 🔌 核心原理与接入

Vibe Process Bar 提供了三种灵活的状态检测机制，适配不同的使用场景：

### 1️⃣ Hook 检测 (推荐)
通过 IDE 自身的 Hook 机制进行精确状态同步。
- **原理**：利用 IDE 的回调机制（如 `.kirohooks` 启动脚本、 `.cursorrules` 等），在任务开始和结束时自动调用脚本通知 Vibe Process Bar。
- **支持应用**：Cursor, Windsurf, Kiro, Trae, CodeBuddy, Antigravity。
- **特点**：官方或原生 Hook 接口触发，准确性极高。

**Hook 脚本位置**：`rules/on-agent-start.sh` 和 `rules/on-agent-complete.sh`

将这些脚本复制到你的 IDE 的 hook 配置目录：
- **Kiro**：复制到 `.kiro/hooks/` 目录
- **Cursor**：在 `.cursorrules` 文件中引用
- **其他 IDE**：按照各自的 hook 配置方式进行配置

### 2️⃣ MCP 协议 (推荐)
通过 Model Context Protocol 标准协议直接连接。
- **原理**：AI Agent 通过 MCP Client 主动连接 Vibe Process Bar Server。
- **提供工具**：
    - `list_tasks`: 获取当前活跃任务列表。
    - `update_task_status`: 汇报任务状态 (running, completed, error 等)。
- **支持应用**：支持所有兼容 MCP 协议的插件与工具（如 Claude Desktop, Cline, RooCode 等）。

**重要提示**：使用 MCP 时，建议同时使用 `rules/rules.md` 中的规则，以确保 AI Agent 正确上报状态。将 `rules/rules.md` 的内容添加到你的 AI 助手的系统提示词或规则配置中。

**配置示例**：

```json
{
  "mcpServers": {
    "vibe-process-bar": {
      "url": "http://127.0.0.1:31415/mcp"
    }
  }
}
```

### 3️⃣ 插件上报 (代码检测)
通过 VS Code 插件分析代码变更频率。
- **原理**：监测文件修改速度、字符变化量来推断 AI 是否正在生成代码。
- **支持插件**：GitHub Copilot, RooCode, Cline, Claude Code 等所有在 VS Code 内运行的 AI 插件。

**⚠️ 必须安装**：使用此功能需要安装 [Vibe Process Bar VS Code 插件](https://github.com/hzw456/vibeProcessBarVSCodeExt)。

> [!WARNING]
> **不推荐用于精确监测**
> 此方式是基于代码修改行为的"猜测"，准确度不如 MCP 或 Hook 方式。仅建议在无法使用 MCP 或 Hook 时作为补充手段使用。


---

## 📁 项目结构

```
vibeProcessBar/
├── docs/
│   └── images/              # 截图和文档图片
├── rules/
│   ├── on-agent-start.sh    # Hook 脚本：AI agent 启动时调用
│   ├── on-agent-complete.sh # Hook 脚本：AI agent 完成时调用
│   └── rules.md             # MCP 规则，供 AI 助手使用
├── src/                     # Vue 前端源码
├── src-tauri/               # Tauri/Rust 后端源码
└── ...
```

---

## 🛠️ 支持的 IDE与插件列表

为了获得最佳体验，建议根据你的工具选择最佳接入方式：

| 工具/IDE | 推荐方式 | 说明 |
|:---|:---|:---|
| **Cursor** | ✅ Hook | 自动支持，无需额外配置 |
| **Windsurf** | ✅ Hook | 自动支持 |
| **Claude Desktop** | ✅ MCP | 需在配置中添加 MCP Server |
| **Cline** | ✅ MCP | 推荐使用 MCP，体验最佳 |
| **RooCode** | ✅ MCP | 推荐使用 MCP |
| **VS Code** | 🔌 插件 | 必需安装 VS Code 插件才能检测内部 AI 行为 |

**注意**：对于在 VS Code 中运行的 AI 助手（如 GitHub Copilot），**必须** 安装配合 Vibe Process Bar 的 VS Code 插件才能实现状态检测。

---

## ⚙️ 设置

点击进度条右侧的设置图标，可以配置：

- 🌐 **语言** — 支持中文 / English
- 🎨 **主题** — 跟随系统 / 深色 / 浅色
- 📍 **位置** — 记住窗口位置
- 🔔 **通知** — 任务完成时提醒

---

## ☕ 支持与捐赠

如果你觉得 Vibe Process Bar 对你有帮助，欢迎请我喝杯咖啡！

<a href="https://ko-fi.com/hzw456" target="_blank">
  <img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="在 Ko-fi 上支持我">
</a>

---

## 📄 许可证

本项目代码采用 [GNU General Public License v3.0 (GPLv3)](https://www.gnu.org/licenses/gpl-3.0.html) 进行许可。

**本软件开源免费，严禁二次售卖。**

### ⚠️ 商标保护声明

**"Vibe Process Bar" 名称及 Logo 不包含在开源许可范围内。**

- 软件名称 "Vibe Process Bar" 及相关 Logo 为本项目专有标识
- 未经书面授权，禁止在衍生作品或分发版本中使用本项目名称和 Logo
- 如需使用，请联系作者获取授权

### 你可以：
- ✅ 自由使用、修改和分发源代码
- ✅ 基于本项目进行二次开发
- ✅ 在遵守 GPLv3 的前提下发布衍生作品

### 你不可以：
- ❌ 将本软件或衍生作品进行售卖
- ❌ 未经授权使用 "Vibe Process Bar" 名称和 Logo
- ❌ 闭源分发修改后的版本

---

<p align="center">
  <strong>让 AI 编程更透明，让等待不再焦虑 ✨</strong>
</p>
