## ADDED Requirements

### Requirement: VSCode Extension Integration

The system SHALL provide a VSCode extension that integrates with Vibe Coding Progress Bar to monitor AI-assisted coding activities.

#### Scenario: Extension activation

- **WHEN** VSCode starts and the Vibe Coding extension is installed
- **THEN** the extension SHALL activate and register AI task status providers
- **AND** the extension SHALL establish a communication channel with the progress bar application

#### Scenario: Extension installation

- **WHEN** user installs the Vibe Coding VSCode extension from the marketplace
- **THEN** the extension SHALL provide configuration options for connection settings
- **AND** the extension SHALL validate connection to the progress bar application

### Requirement: AI Task Status Detection

The VSCode extension SHALL detect and report AI coding task activities from installed AI extensions.

#### Scenario: Detect GitHub Copilot activity

- **WHEN** GitHub Copilot is generating code suggestions
- **THEN** the extension SHALL capture the task status and progress information
- **AND** SHALL transmit the status to the progress bar application

#### Scenario: Detect Continue extension activity

- **WHEN** Continue extension is executing AI code generation
- **THEN** the extension SHALL extract task metadata and progress
- **AND** SHALL update the progress bar in real-time

#### Scenario: Detect Claude Code activity

- **WHEN** Claude Code extension is processing AI requests
- **THEN** the extension SHALL monitor the request lifecycle
- **AND** SHALL report completion status to the progress bar

### Requirement: Window and Project Detection

The system SHALL automatically detect the active VSCode window and associated project context.

#### Scenario: Active window detection

- **WHEN** user switches between multiple VSCode windows
- **THEN** the system SHALL update the monitored context to the active window
- **AND** SHALL sync project-specific AI task states

#### Scenario: Multi-workspace support

- **WHEN** VSCode has multiple workspaces open
- **THEN** the system SHALL track AI tasks across all workspaces
- **AND** SHALL provide workspace-switching UI in the progress bar

### Requirement: Progress Synchronization

The system SHALL maintain real-time synchronization of AI task progress with the progress bar application.

#### Scenario: Real-time progress update

- **WHEN** an AI task is in progress
- **THEN** the progress bar SHALL update at least once per second
- **AND** SHALL display current stage and percentage completion

#### Scenario: Task completion notification

- **WHEN** an AI task completes
- **THEN** the extension SHALL send a completion event with final status
- **AND** the progress bar SHALL display completion state for 3 seconds before resetting

#### Scenario: Connection loss handling

- **WHEN** communication between extension and progress bar is interrupted
- **THEN** the extension SHALL queue status updates locally
- **AND** SHALL attempt reconnection every 5 seconds
- **AND** SHALL notify user of disconnection after 30 seconds

### Requirement: Communication Protocol

The VSCode extension and progress bar application SHALL communicate using a defined protocol.

#### Scenario: WebSocket communication

- **WHEN** both extension and progress bar are running
- **THEN** they SHALL establish a WebSocket connection on localhost port 31415
- **AND** SHALL exchange JSON messages with the defined schema

#### Scenario: Message format

- **WHEN** sending status updates
- **THEN** messages SHALL follow the format:
  ```json
  {
    "type": "status" | "progress" | "complete" | "error",
    "timestamp": "ISO-8601",
    "data": { ... }
  }
  ```

#### Scenario: Heartbeat

- **WHEN** no AI task activity for 10 seconds
- **THEN** the extension SHALL send a heartbeat message
- **AND** the progress bar SHALL respond with acknowledgment

### Requirement: User Configuration

The system SHALL provide user-configurable options for VSCode integration.

#### Scenario: Connection settings

- **WHEN** user opens extension settings
- **THEN** user MAY configure connection port (default: 31415)
- **AND** user MAY enable/disable specific AI extension detection
- **AND** user MAY set progress update frequency

#### Scenario: Notification preferences

- **WHEN** user configures notification settings
- **THEN** user MAY enable/disable system notifications for task completion
- **AND** user MAY choose notification sound
- **AND** user MAY set minimum progress threshold for notifications
