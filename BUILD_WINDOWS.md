# Windows 构建指南

## 前置要求

在 Windows 上构建 Vibe Process Bar 需要以下工具：

### 1. Node.js
- 下载并安装 [Node.js](https://nodejs.org/) (推荐 LTS 版本)
- 验证安装：`node --version` 和 `npm --version`

### 2. Rust
- 下载并安装 [Rust](https://www.rust-lang.org/tools/install)
- 使用 rustup 安装：访问 https://rustup.rs/
- 验证安装：`rustc --version` 和 `cargo --version`

### 3. Visual Studio C++ Build Tools
- 下载 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- 安装时选择 "Desktop development with C++" 工作负载
- 或者安装完整的 Visual Studio Community

### 4. WebView2 (通常已预装在 Windows 10/11)
- 如果没有，从 [Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/) 下载

## 构建步骤

### 方法 1: 使用 npm 脚本（推荐）

```bash
# 1. 安装依赖
npm install

# 2. 构建前端
npm run build

# 3. 构建 Tauri 应用（开发版）
npm run tauri build -- --debug

# 4. 构建 Tauri 应用（发布版）
npm run tauri build
```

### 方法 2: 手动构建

```bash
# 1. 安装 Node.js 依赖
npm install

# 2. 构建前端
npm run build

# 3. 进入 Tauri 目录
cd src-tauri

# 4. 构建 Rust 应用（开发版）
cargo build

# 5. 构建 Rust 应用（发布版）
cargo build --release
```

## 输出文件位置

构建完成后，可执行文件和安装包位于：

### 开发版
- 可执行文件：`src-tauri/target/debug/vibe-process-bar.exe`

### 发布版
- 可执行文件：`src-tauri/target/release/vibe-process-bar.exe`
- MSI 安装包：`src-tauri/target/release/bundle/msi/VibeProcessbar_1.0.1_x64_en-US.msi`
- NSIS 安装包：`src-tauri/target/release/bundle/nsis/VibeProcessbar_1.0.1_x64-setup.exe`

## Windows 特定功能

本应用在 Windows 上支持以下功能：

1. **窗口模糊效果** - 使用 Windows Acrylic 效果
2. **系统托盘** - 最小化到系统托盘
3. **开机自启动** - 通过注册表实现
4. **IDE 窗口检测** - 使用 PowerShell 检测 VS Code、Cursor 等 IDE
5. **窗口激活** - 使用 Win32 API 激活指定窗口

## 常见问题

### 1. 构建失败：找不到 MSVC
**解决方案**：安装 Visual Studio Build Tools 并确保包含 C++ 工作负载

### 2. WebView2 错误
**解决方案**：从 Microsoft 官网下载并安装 WebView2 Runtime

### 3. PowerShell 脚本执行策略错误
**解决方案**：以管理员身份运行 PowerShell 并执行：
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 4. 窗口透明效果不工作
**解决方案**：确保 Windows 10 版本 >= 1809 或 Windows 11

## 代码改进

相比 macOS 版本，Windows 版本做了以下改进：

1. **窗口扫描**：使用 PowerShell 的 `Get-Process` 枚举窗口
2. **窗口激活**：使用 Win32 API (`SetForegroundWindow`, `ShowWindow`)
3. **开机自启动**：使用注册表键值 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
4. **窗口效果**：使用 `apply_blur` 替代 macOS 的 `apply_vibrancy`

## 测试

构建完成后，可以直接运行可执行文件测试：

```bash
# 运行开发版
.\src-tauri\target\debug\vibe-process-bar.exe

# 运行发布版
.\src-tauri\target\release\vibe-process-bar.exe
```

## 分发

推荐使用 MSI 或 NSIS 安装包进行分发：

- **MSI**：适合企业环境，支持 GPO 部署
- **NSIS**：更小的文件大小，更快的安装速度

安装包会自动处理：
- 应用程序安装
- 快捷方式创建
- 卸载程序注册
- 文件关联（如需要）
