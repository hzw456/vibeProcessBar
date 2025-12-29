# Vibe Process Bar HTTP API

HTTP API for controlling the Vibe Process Bar floating window.

**Base URL:** `http://localhost:31415`

## Logging

The application outputs structured logs to help with debugging:

**Backend Logs (Rust):**
- Output to stdout/stderr
- Controlled by `RUST_LOG` environment variable
- Levels: `trace` > `debug` > `info` > `warn` > `error`

```bash
# Enable debug logging
RUST_LOG=debug ./vibeprocessbar

# Enable verbose logging
RUST_LOG=trace ./vibeprocessbar
```

**Frontend Logs (TypeScript):**
- Output to browser DevTools Console
- Format: `[LEVEL] message`

**Log Locations:**
| Component | Log Points |
|-----------|-----------|
| http_server.rs | HTTP requests, task state changes, errors |
| status_reporter.rs | Status reports, event broadcasts |
| progressStore.ts | HTTP sync, task mutations |
| useProgressEvent.ts | Event emissions, handler calls |
| App.tsx | UI actions (task selection, IDE activation) |

## Endpoints

### Get Status
Get current task status and all tasks.

```
GET /api/status
```

**Response:**
```json
{
  "currentTask": {
    "id": "task-123",
    "name": "Generate user module",
    "progress": 50,
    "tokens": 1500,
    "status": "running",
    "ide": "cursor",
    "window_title": "user.ts - Cursor",
    "start_time": 1703123456789
  },
  "tasks": [...],
  "taskCount": 1
}
```

### Start Task
Start a new task.

```
POST /api/task/start
Content-Type: application/json

{
  "task_id": "unique-task-id",
  "name": "Task name",
  "ide": "cursor",
  "window_title": "file.ts - Cursor"
}
```

**Response:**
```json
{"status": "ok"}
```

### Update Progress
Update task progress (0-100).

```
POST /api/task/progress
Content-Type: application/json

{
  "task_id": "unique-task-id",
  "progress": 50
}
```

### Update Tokens
Update token count.

```
POST /api/task/token
Content-Type: application/json

{
  "task_id": "unique-task-id",
  "tokens": 1500,
  "increment": true
}
```

- `increment: true` - Add to existing token count
- `increment: false` - Set absolute token count

### Complete Task
Mark task as completed.

```
POST /api/task/complete
Content-Type: application/json

{
  "task_id": "unique-task-id",
  "total_tokens": 2500
}
```

### Set Error Status
Mark task as error.

```
POST /api/task/error
Content-Type: application/json

{
  "task_id": "unique-task-id",
  "message": "Error description"
}
```

### Cancel Task
Remove a task.

```
POST /api/task/cancel
Content-Type: application/json

{
  "task_id": "unique-task-id"
}
```

### Reset All
Clear all tasks.

```
POST /api/reset
Content-Type: application/json

{}
```

## Task Status Values

- `running` - Task is in progress
- `completed` - Task finished successfully
- `error` - Task encountered an error

## Supported IDEs

- `cursor` - Cursor IDE
- `vscode` - Visual Studio Code
- `claude` / `claude-code` - Claude Desktop

## Example Usage

```bash
# Start a task
curl -X POST http://localhost:31415/api/task/start \
  -H "Content-Type: application/json" \
  -d '{"task_id": "task-1", "name": "Generate code", "ide": "cursor", "window_title": "main.ts"}'

# Update progress
curl -X POST http://localhost:31415/api/task/progress \
  -H "Content-Type: application/json" \
  -d '{"task_id": "task-1", "progress": 75}'

# Complete task
curl -X POST http://localhost:31415/api/task/complete \
  -H "Content-Type: application/json" \
  -d '{"task_id": "task-1", "total_tokens": 5000}'
```
