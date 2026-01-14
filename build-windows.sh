#!/bin/bash

# Windows Build Script for Vibe Process Bar
# This script should be run in a Windows environment with Rust and Node.js installed

set -e

echo "🚀 Building Vibe Process Bar for Windows..."

# Check if running on Windows
if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "win32" && "$OSTYPE" != "cygwin" ]]; then
    echo "⚠️  Warning: This script is designed for Windows. Current OS: $OSTYPE"
    echo "   You may need to run this in Git Bash, MSYS2, or WSL with Windows Rust toolchain."
fi

# Install dependencies
echo "📦 Installing Node.js dependencies..."
npm install

# Build frontend
echo "🎨 Building frontend..."
npm run build

# Build Tauri app for Windows
echo "🔨 Building Tauri app for Windows..."
cd src-tauri

# Build release version
cargo build --release

echo "✅ Build complete!"
echo ""
echo "📦 Windows executable location:"
echo "   src-tauri/target/release/vibe-process-bar.exe"
echo ""
echo "To create installer, run:"
echo "   npm run tauri build"
