# add-core-logging Tasks

## Phase 1: Rust Backend Logging

- [x] **TASK-1**: Add `tracing` crate dependency to `src-tauri/Cargo.toml`
- [x] **TASK-2**: Add `tracing-subscriber` crate dependency to `src-tauri/Cargo.toml`
- [x] **TASK-3**: Initialize tracing subscriber in `src-tauri/src/main.rs` with `tracing_subscriber::fmt::init()`
- [x] **TASK-4**: Add request logging to `http_server.rs` using `tracing::debug!` for method, path, body
- [x] **TASK-5**: Add state change logging to `http_server.rs` using `tracing::info!` for create/update/complete/error
- [x] **TASK-6**: Add error logging to `http_server.rs` using `tracing::error!` for connection failures
- [x] **TASK-7**: Add status report logging to `status_reporter.rs` using `tracing::debug!`
- [x] **TASK-8**: Add event broadcast logging to `status_reporter.rs` using `tracing::debug!`

## Phase 2: TypeScript Frontend Logging

- [x] **TASK-9**: Create `src/utils/logger.ts` with level-based logging API
- [x] **TASK-10**: Add HTTP sync logging to `src/stores/progressStore.ts`
- [x] **TASK-11**: Add event emission logging to `src/hooks/useProgressEvent.ts`
- [x] **TASK-12**: Add UI action logging to `src/App.tsx` (task selection, IDE activation)

## Phase 3: Documentation

- [x] **TASK-13**: Update `API.md` with logging documentation section

## Phase 4: Validation

- [x] **TASK-14**: Run TypeScript check to verify frontend code compiles
- [ ] **TASK-15**: Test logging output by running the application with various log levels (RUST_LOG=trace/debug/info)
- [ ] **TASK-16**: Verify tracing spans are properly recorded in function calls

## Log Points Summary

### Rust Backend (`src-tauri/src/`)

| File | Log Points |
|------|------------|
| main.rs:293 | Server startup info |
| http_server.rs:90-92 | HTTP request received (method, path, body) |
| http_server.rs:110 | Task armed (info) |
| http_server.rs:135 | Task started (info) |
| http_server.rs:158 | Task progress updated (debug) |
| http_server.rs:172 | Token count updated (debug) |
| http_server.rs:191 | Task completed (info) |
| http_server.rs:210 | Task error (warn) |
| http_server.rs:230 | Task cancelled (info) |
| http_server.rs:305 | Server listening (info) |
| http_server.rs:312 | Connection error (error) |
| http_server.rs:323 | HTTP server error (error) |
| status_reporter.rs:119 | Status report received (debug) |
| status_reporter.rs:125 | Event broadcast sent (debug) |
| status_reporter.rs:131 | Frontend event emitted (debug) |
| status_reporter.rs:154 | Plugin initialization (info) |

### TypeScript Frontend (`src/`)

| File | Log Points |
|------|------------|
| utils/logger.ts | Logger utility with debug/info/warn/error functions |
| stores/progressStore.ts:275 | HTTP sync (debug), sync error (error) |
| hooks/useProgressEvent.ts:16 | Event handler registered (debug) |
| hooks/useProgressEvent.ts:24 | Event emitted (debug) |
| App.tsx:14 | App loaded (debug) |
| App.tsx:79 | App mounted (debug) |
| App.tsx:70 | Window resize error (error) |
| App.tsx:90 | IDE window activated (debug) |
| App.tsx:96 | Task clicked (debug) |
| App.tsx:101 | IDE activation error (error) |
| App.tsx:120 | Window position error (error) |

## Logging Configuration

**Rust Backend:**
```bash
# Set log level
RUST_LOG=debug ./vibeprocessbar
RUST_LOG=trace ./vibeprocessbar  # Most verbose
```

**Frontend:**
- Logs output to browser DevTools Console
- Format: `[LEVEL] message [data]`
