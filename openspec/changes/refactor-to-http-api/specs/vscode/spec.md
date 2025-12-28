## REMOVED Requirements

### Requirement: VSCode Extension Integration

The VSCode extension integration capability has been removed.

- REMOVED: VSCode extension that monitors AI-assisted coding activities
- REMOVED: Extension activation and communication with progress bar
- REMOVED: Extension installation from marketplace

### Requirement: AI Task Status Detection

The AI task status detection from VSCode extensions has been removed.

- REMOVED: GitHub Copilot activity detection
- REMOVED: Continue extension activity detection
- REMOVED: Claude Code extension activity detection

### Requirement: Window and Project Detection

The automatic VSCode window and project detection has been removed.

- REMOVED: Active window detection across VSCode windows
- REMOVED: Multi-workspace support
- REMOVED: Project-specific AI task state synchronization

### Requirement: Progress Synchronization

The real-time progress synchronization via VSCode extension has been removed.

- REMOVED: Real-time progress update via WebSocket
- REMOVED: Task completion notification
- REMOVED: Connection loss handling and reconnection

### Requirement: Communication Protocol

The WebSocket communication protocol with VSCode extension has been removed.

- REMOVED: WebSocket server on localhost port 31415
- REMOVED: JSON message format for status updates
- REMOVED: Heartbeat mechanism

### Requirement: User Configuration

The VSCode-specific user configuration has been removed.

- REMOVED: VSCode connection settings (host, port)
- REMOVED: AI extension detection enable/disable options
- REMOVED: Progress update frequency configuration

## Related Capabilities

This removal is replaced by the new `http-api` capability which provides a simpler, IDE-agnostic API for any external tool to update progress status.
