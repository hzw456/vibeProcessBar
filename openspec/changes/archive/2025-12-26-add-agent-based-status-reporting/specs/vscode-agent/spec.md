## ADDED Requirements

### Requirement: HTTP Status Reporting API

The system SHALL provide an HTTP API endpoint for AI tools to report task status.

#### Scenario: Receive status report from AI tool

- **WHEN** an AI tool sends a POST request to `http://127.0.0.1:31416/status` with JSON body
- **THEN** the system SHALL parse the JSON and extract windowId, stage, progress, and message
- **AND** SHALL emit a status change event to the frontend
- **AND** SHALL respond with HTTP 200 and JSON `{"status": "ok"}`

#### Scenario: Validate required fields

- **WHEN** an AI tool sends an incomplete status report (missing windowId or stage)
- **THEN** the system SHALL respond with HTTP 400 and error details
- **AND** SHALL NOT update the displayed status

#### Scenario: Health check endpoint

- **WHEN** an HTTP client sends a GET request to `http://127.0.0.1:31416/health`
- **THEN** the system SHALL respond with HTTP 200 and `{"status": "healthy"}`

### Requirement: Window Activation via System Calls

The system SHALL provide functionality to activate a specific application window when requested.

#### Scenario: Activate VSCode window on macOS

- **WHEN** the user clicks the floating window and the target is VSCode on macOS
- **THEN** the system SHALL execute AppleScript to bring VSCode to front
- **AND** SHALL activate the VSCode window

#### Scenario: Activate VSCode window on Windows

- **WHEN** the user clicks the floating window and the target is VSCode on Windows
- **THEN** the system SHALL execute PowerShell commands to activate VSCode
- **AND** SHALL bring the VSCode window to front

#### Scenario: Activate VSCode window on Linux

- **WHEN** the user clicks the floating window and the target is VSCode on Linux
- **THEN** the system SHALL execute wmctrl to activate VSCode
- **AND** SHALL bring the VSCode window to front

### Requirement: Floating Window Click Interaction

The floating window SHALL respond to user clicks by activating the associated application window.

#### Scenario: Click on status text with active window

- **WHEN** the user clicks on the status text area
- **AND** there is an active window ID stored
- **THEN** the system SHALL call the Rust backend to activate that window
- **AND** SHALL clear any pending visual feedback

#### Scenario: Click on status text without active window

- **WHEN** the user clicks on the status text area
- **AND** there is no active window ID stored
- **THEN** the system SHALL log a warning
- **AND** SHALL NOT attempt window activation

### Requirement: Multi-Stage Progress Tracking

The system SHALL track and display progress across multiple task stages.

#### Scenario: Track stage progression

- **WHEN** an AI tool reports status with stage "analysis" and progress 10
- **THEN** the system SHALL display "Analysis" stage with 10% progress
- **AND** SHALL store the stage information for later reference

#### Scenario: Update progress within same stage

- **WHEN** an AI tool reports status with stage "coding" and progress changes from 40 to 60
- **THEN** the system SHALL update the progress bar without changing the stage label
- **AND** SHALL emit an update event to the frontend

#### Scenario: Complete task reporting

- **WHEN** an AI tool reports status with stage "completed" and progress 100
- **THEN** the system SHALL display 100% completion
- **AND** SHALL show completion state for 5 seconds
- **AND** SHALL reset to idle state after timeout

### Requirement: Platform-Specific Window Identification

The system SHALL support platform-specific methods for identifying and activating windows.

#### Scenario: Extract window ID from process info on macOS

- **WHEN** receiving a status report on macOS
- **THEN** the system MAY extract window info from VSCode process
- **AND** SHALL use AppleScript for activation

#### Scenario: Extract window ID from process info on Windows

- **WHEN** receiving a status report on Windows
- **THEN** the system MAY use window title matching
- **AND** SHALL use PowerShell for activation

#### Scenario: Extract window ID from process info on Linux

- **WHEN** receiving a status report on Linux
- **THEN** the system MAY use X11 window properties
- **AND** SHALL use wmctrl for activation

### Requirement: Prompt Template Generation

The system SHALL provide configurable prompt templates for different AI tools.

#### Scenario: Generate Claude Code CLI template

- **WHEN** the user requests a prompt template for Claude Code CLI
- **THEN** the system SHALL provide a shell script template with curl commands
- **AND** SHALL include instructions for status reporting after each stage

#### Scenario: Generate Continue extension template

- **WHEN** the user requests a prompt template for Continue VSCode extension
- **THEN** the system SHALL provide a JSON or markdown template
- **AND** SHALL include example curl commands

#### Scenario: Generate universal template

- **WHEN** the user requests a universal template
- **THEN** the system SHALL provide a markdown template compatible with any AI chat
- **AND** SHALL include clear examples for status reporting

### Requirement: Frontend State Display

The React frontend SHALL display the current status information from the Rust backend.

#### Scenario: Display current stage and progress

- **WHEN** the Rust backend emits a status change event
- **THEN** the frontend SHALL update the stage label
- **AND** SHALL animate the progress bar to the new value
- **AND** SHALL display the status message

#### Scenario: Display window information

- **WHEN** the Rust backend provides window information
- **THEN** the frontend SHALL display the window title
- **AND** SHALL show a clickable indicator for window activation

#### Scenario: Handle connection loss

- **WHEN** the WebSocket or Tauri event connection is lost
- **THEN** the frontend SHALL display a disconnected state
- **AND** SHALL attempt reconnection every 5 seconds
- **AND** SHALL notify user after 30 seconds of disconnection

### Requirement: Multiple Task Tracking

The system SHALL track status for multiple concurrent AI tasks.

#### Scenario: Handle multiple simultaneous reports

- **WHEN** multiple AI tools report status simultaneously
- **THEN** the system SHALL track each task separately by windowId
- **AND** SHALL display the most recently active task by default
- **AND** SHALL allow switching between tasks

#### Scenario: Task timeout handling

- **WHEN** no status update is received for an active task within 60 seconds
- **THEN** the system SHALL mark the task as stale
- **AND** SHALL notify the user
- **AND** SHALL allow manual task reset
