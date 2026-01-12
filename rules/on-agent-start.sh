#!/bin/bash
# Kiro Hook: Agent Start - 当Kiro开始执行时通知Vibe Process Bar

VIBE_API="http://localhost:31415"
PROJECT_PATH="$(pwd)"

# 自动检测 IDE - 使用 __CFBundleIdentifier (macOS)
detect_ide() {
  local bundle_id="${__CFBundleIdentifier:-}"
  case "$bundle_id" in
    com.microsoft.VSCode) echo "vscode" ;;
    com.todesktop.230313mzl4w4u92) echo "cursor" ;;
    dev.kiro.desktop) echo "kiro" ;;
    com.google.antigravity) echo "antigravity" ;;
    com.codeium.windsurf) echo "windsurf" ;;
    com.trae.app) echo "trae" ;;
    com.tencent.codebuddy) echo "codebuddy" ;;
    com.tencent.codebuddycn) echo "codebuddycn" ;;
    *) echo "${TERM_PROGRAM:-unknown}" ;;
  esac
}

IDE=$(detect_ide)

if [ -n "$PROJECT_PATH" ]; then
  curl -s -X POST "$VIBE_API/api/task/update_state_by_path" \
    -H "Content-Type: application/json" \
    -d "{\"project_path\": \"$PROJECT_PATH\", \"ide\": \"$IDE\", \"status\": \"running\", \"source\": \"hook\"}" > /dev/null 2>&1
fi
