#!/bin/bash
# Vibe Process Bar reporter script
# Usage: vibe-report.sh start|complete [task_name]

ACTION=$1
TASK_NAME=${2:-"Kiro AI Task"}
TASK_ID_FILE="/tmp/kiro_vibe_task_id"
API_URL="http://localhost:31415/api/task"

# Get current project name from pwd
PROJECT_NAME=$(basename "$(pwd)")

case $ACTION in
  start)
    TASK_ID="kiro_$(date +%s)"
    echo "$TASK_ID" > "$TASK_ID_FILE"
    curl -s -X POST "$API_URL/start" \
      -H "Content-Type: application/json" \
      -d "{\"task_id\":\"$TASK_ID\",\"name\":\"$TASK_NAME\",\"ide\":\"kiro\",\"window_title\":\"$PROJECT_NAME\"}" > /dev/null 2>&1
    ;;
  complete)
    if [ -f "$TASK_ID_FILE" ]; then
      TASK_ID=$(cat "$TASK_ID_FILE")
      curl -s -X POST "$API_URL/complete" \
        -H "Content-Type: application/json" \
        -d "{\"task_id\":\"$TASK_ID\"}" > /dev/null 2>&1
      rm -f "$TASK_ID_FILE"
    fi
    ;;
esac
