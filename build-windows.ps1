# Windows Build Script for Vibe Process Bar
# PowerShell version

param(
    [switch]$Debug,
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"

Write-Host "🚀 Building Vibe Process Bar for Windows..." -ForegroundColor Cyan

# Check Node.js
Write-Host "`n📦 Checking Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version
    Write-Host "   Node.js version: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Node.js not found. Please install from https://nodejs.org/" -ForegroundColor Red
    exit 1
}

# Check Rust
Write-Host "`n🦀 Checking Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version
    Write-Host "   Rust version: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Rust not found. Please install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Install Node.js dependencies
if (-not $SkipFrontend) {
    Write-Host "`n📦 Installing Node.js dependencies..." -ForegroundColor Yellow
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "   ❌ Failed to install dependencies" -ForegroundColor Red
        exit 1
    }

    # Build frontend
    Write-Host "`n🎨 Building frontend..." -ForegroundColor Yellow
    npm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "   ❌ Failed to build frontend" -ForegroundColor Red
        exit 1
    }
}

# Build Tauri app
Write-Host "`n🔨 Building Tauri application..." -ForegroundColor Yellow
Set-Location src-tauri

if ($Debug) {
    Write-Host "   Building DEBUG version..." -ForegroundColor Cyan
    cargo build
    $buildType = "debug"
} else {
    Write-Host "   Building RELEASE version..." -ForegroundColor Cyan
    cargo build --release
    $buildType = "release"
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ Failed to build Tauri app" -ForegroundColor Red
    Set-Location ..
    exit 1
}

Set-Location ..

Write-Host "`n✅ Build complete!" -ForegroundColor Green
Write-Host "`n📦 Output files:" -ForegroundColor Cyan
Write-Host "   Executable: src-tauri\target\$buildType\vibe-process-bar.exe" -ForegroundColor White

if (-not $Debug) {
    Write-Host "`n📦 To create installer, run:" -ForegroundColor Yellow
    Write-Host "   npm run tauri build" -ForegroundColor White
    Write-Host "`n   Installers will be in:" -ForegroundColor Yellow
    Write-Host "   - src-tauri\target\release\bundle\msi\" -ForegroundColor White
    Write-Host "   - src-tauri\target\release\bundle\nsis\" -ForegroundColor White
}

Write-Host "`n🎉 Done!" -ForegroundColor Green
