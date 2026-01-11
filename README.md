# Vibe Process Bar

<p align="center">
  <img src="./src-tauri/icons/512x512.png" alt="Vibe Process Bar Logo" width="128" height="128">
</p>

<p align="center">
  <strong>Visualize AI Coding Agent's Workflow at a Glance</strong>
</p>

<p align="center">
  A desktop floating progress bar designed for AI-assisted programming, visualizing AI Agent's working status in real-time.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-blue" alt="Platform">
  <img src="https://img.shields.io/badge/built%20with-Tauri%20%2B%20Vue-green" alt="Built with">
  <img src="https://img.shields.io/badge/license-MIT-orange" alt="License">
</p>

---

## Why Vibe Process Bar?

When using AI coding assistants like Cursor, Kiro, or Claude, do you often face these issues?

- 🤔 **Unknown Status** — AI is thinking in the background while you wait blindly
- ⏰ **Uncertain Completion** — Lose track of progress after switching windows
- 🔄 **Multi-task Chaos** — Easy to get confused when running multiple AI tasks

**Vibe Process Bar** solves these problems! It is a lightweight floating window that continuously stays on top, keeping you informed of the AI's working status at all times.

---

## ✨ Core Features

### 🎯 Real-time Status Tracking

| Status | Icon | Meaning |
|:---:|:---:|:---|
| **Armed** | ◎ | Standing by, ready to start |
| **Running** | ◉ | AI is working, dynamic progress bar |
| **Completed** | ✓ | Task completed |
| **Focused** | 👁 | Window gained focus |

### 🖥️ Multi-IDE Support

Fully supports mainstream AI coding tools and IDEs:

- **VS Code** (including derivatives)
    - Supports all major AI plugins: GitHub Copilot, Cline, RooCode, Claude Code, etc.
- **Cursor** (Native Support)
- **Kiro** (Native Support)
- **Windsurf** (Native Support)
- **Antigravity** (Native Support)
- **Trae** (Native Support)
- **CodeBuddy / CodeBuddy CN** (Native Support)

### 🎨 Elegant Floating Design

- Translucent frosted glass effect, doesn't obstruct workspace
- Draggable to any position
- Double-click to quickly reset status
- Always on top, always visible

### 📊 Multi-task Management

- Track multiple AI tasks simultaneously
- Click to switch between different tasks
- Independent display of progress and status for each task

---

## 🚀 Quick Start

### Method 1: Download Pre-built Version (Recommended)

Download from the [Releases](https://github.com/hzw456/vibeProcessBar/releases) page:

| System | Download |
|:---|:---|
| macOS (Apple Silicon) | `VibeProcessBar_x.x.x_aarch64.dmg` |
| macOS (Intel) | `VibeProcessBar_x.x.x_x64.dmg` |

### Method 2: Build from Source

```bash
# 1. Clone the repository
git clone https://github.com/hzw456/vibeProcessBar.git
cd vibeProcessBar

# 2. Install dependencies
npm install

# 3. Run in development mode
npm run tauri dev

# 4. Build for production
npm run tauri build
```

**System Requirements:**
- macOS
- Node.js 18+
- Rust 1.70+
- Tauri CLI (`npm install -D @tauri-apps/cli`)

---

## 🔌 Core Principles & Integration

Vibe Process Bar provides three flexible status detection mechanisms, adapting to different usage scenarios:

### 1️⃣ Hook Detection (Recommended)
Syncs status precisely via the IDE's own Hook mechanism.
- **Principle**: Uses IDE callbacks (like `.kirohooks` start scripts, `.cursorrules`, etc.) to automatically notify Vibe Process Bar when tasks start and end.
- **Supported Apps**: Cursor, Windsurf, Kiro, Trae, CodeBuddy, Antigravity.
- **Features**: Triggered by official or native Hook interfaces, extremely accurate.

### 2️⃣ MCP Protocol (Recommended)
Connects directly via the Model Context Protocol standard.
- **Principle**: AI Agent actively connects to Vibe Process Bar Server via MCP Client.
- **Tools Provided**:
    - `list_tasks`: Get current active task list.
    - `update_task_status`: Report task status (running, completed, error, etc.).
- **Supported Apps**: Supports all plugins and tools compatible with MCP protocol (like Claude Desktop, Cline, RooCode, etc.).
- **Configuration Example**:

```json
{
  "mcpServers": {
    "vibe-process-bar": {
      "url": "http://127.0.0.1:31415/mcp"
    }
  }
}
```

### 3️⃣ Plugin Reporting (Code Detection)
Analyzes code change frequency via VS Code extension.
- **Principle**: Monitors file modification speed and character changes to infer if AI is generating code.
- **Supported Plugins**: GitHub Copilot, RooCode, Cline, Claude Code, and all other AI plugins running inside VS Code.
> [!WARNING]
> **Not recommended for precise tracking**
> This method is a "guess" based on code modification behavior, which is less accurate than MCP or Hook methods. Only recommended as a supplement when MCP or Hook cannot be used.

---

## 🛠️ Supported IDEs & Plugin List

For the best experience, we recommend choosing the integration method that best fits your tool:

| Tool/IDE | Recommended Method | Optimization |
|:---|:---|:---|
| **Cursor** | ✅ Hook | Automatic support, no config needed |
| **Windsurf** | ✅ Hook | Automatic support |
| **Claude Desktop** | ✅ MCP | Add MCP Server to config |
| **Cline** | ✅ MCP | Recommended, best experience |
| **RooCode** | ✅ MCP | Recommended |
| **GitHub Copilot** | ⚠️ Plugin Reporting | Requires [Vibe Process Bar Extension](https://github.com/hzw456/vibeProcessBarVSCodeExt) |
| **VS Code** | 🔌 Plugin | Must install VS Code extension to detect internal AI activity |

**Note**: For AI assistants running in VS Code (like GitHub Copilot), you **must** install the Vibe Process Bar VS Code extension to enable status detection.

---

## ⚙️ Settings

Click the settings icon on the right side of the progress bar to configure:

- 🌐 **Language** — Support Chinese / English
- 🎨 **Theme** — System / Dark / Light
- 📍 **Position** — Remember window position
- 🔔 **Notifications** — Alert when task completes

---

## 📄 License

MIT License © 2024

---

<p align="center">
  <strong>Make AI programming transparent, make waiting anxiety-free ✨</strong>
</p>
