# Windows 快速开始指南

## 🚀 快速构建（3 步）

### 1️⃣ 安装依赖
```powershell
npm install
```

### 2️⃣ 构建应用
```powershell
# 使用 PowerShell 脚本（推荐）
.\build-windows.ps1

# 或使用 npm（创建安装包）
npm run tauri build
```

### 3️⃣ 运行程序
```powershell
# 运行构建的可执行文件
.\src-tauri\target\release\vibe-process-bar.exe

# 或安装 MSI/NSIS 安装包
```

## 📦 输出文件

构建完成后，你会得到：

- **可执行文件**：`src-tauri\target\release\vibe-process-bar.exe`
- **MSI 安装包**：`src-tauri\target\release\bundle\msi\VibeProcessbar_1.0.0_x64_en-US.msi`
- **NSIS 安装包**：`src-tauri\target\release\bundle\nsis\VibeProcessbar_1.0.0_x64-setup.exe`

## 🛠️ 前置要求

如果构建失败，请确保已安装：

1. **Node.js** - https://nodejs.org/
2. **Rust** - https://rustup.rs/
3. **Visual Studio Build Tools** - https://visualstudio.microsoft.com/downloads/
   - 安装时选择 "Desktop development with C++"

## 🎯 PowerShell 脚本选项

```powershell
# 构建发布版（默认）
.\build-windows.ps1

# 构建调试版
.\build-windows.ps1 -Debug

# 跳过前端构建（仅重新编译 Rust）
.\build-windows.ps1 -SkipFrontend
```

## ⚡ 开发模式

```powershell
# 启动开发服务器
npm run dev

# 在另一个终端启动 Tauri
npm run tauri dev
```

## 🐛 常见问题

### PowerShell 脚本无法运行
```powershell
# 以管理员身份运行 PowerShell，然后执行：
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 找不到 MSVC 编译器
安装 Visual Studio Build Tools 并确保包含 C++ 工作负载

### WebView2 错误
从 https://developer.microsoft.com/microsoft-edge/webview2/ 下载安装

## 📚 详细文档

查看 [BUILD_WINDOWS.md](BUILD_WINDOWS.md) 获取完整的构建文档。

## ✨ Windows 特性

本应用在 Windows 上支持：

- ✅ 窗口模糊效果（Acrylic）
- ✅ 系统托盘集成
- ✅ 开机自启动
- ✅ IDE 窗口检测（VS Code、Cursor 等）
- ✅ 窗口激活和聚焦
- ✅ HTTP API 服务器
- ✅ MCP 协议支持

## 🎉 完成！

构建成功后，你可以：

1. 直接运行 `.exe` 文件
2. 安装 `.msi` 或 `-setup.exe` 安装包
3. 分发给其他用户

祝使用愉快！🎊
