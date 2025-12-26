import * as vscode from 'vscode';
import { WebSocket } from 'ws';
import { ProgressMessage, DEFAULT_CONFIG, createStatusMessage, createHeartbeatMessage } from './protocol';
import { CopilotDetector } from './detectors/copilot';
import { ClaudeCodeDetector } from './detectors/claudeCode';
import { CursorDetector } from './detectors/cursor';
import { TaskInfo } from './types';

interface ExtensionState {
  ws: WebSocket | null;
  reconnectTimer: ReturnType<typeof setTimeout> | null;
  heartbeatTimer: ReturnType<typeof setInterval> | null;
  activeTasks: Map<string, TaskInfo>;
  lastActivity: number;
}

let state: ExtensionState | null = null;

export function activate(context: vscode.ExtensionContext): void {
  state = {
    ws: null,
    reconnectTimer: null,
    heartbeatTimer: null,
    activeTasks: new Map(),
    lastActivity: Date.now(),
  };

  const config = vscode.workspace.getConfiguration('vibeCodingProgressBar');
  const detectors = [
    new CopilotDetector(config),
    new ClaudeCodeDetector(config),
    new CursorDetector(config),
  ];

  const statusBar = vscode.window.createStatusBarItem('vibe-coding', vscode.StatusBarAlignment.Right, 100);
  statusBar.text = '$(sync~spin) Vibe Coding';
  statusBar.tooltip = 'Vibe Coding Progress Bar';
  statusBar.command = 'vibeCodingProgressBar.toggle';
  statusBar.show();

  context.subscriptions.push(statusBar);

  function connect(): void {
    if (!state) return;

    const host = config.get<string>('host', DEFAULT_CONFIG.host);
    const port = config.get<number>('port', DEFAULT_CONFIG.port);
    const url = `ws://${host}:${port}`;

    try {
      state.ws = new WebSocket(url);

      state.ws.on('open', () => {
        vscode.window.showInformationMessage('Connected to Vibe Coding Progress Bar');
        startHeartbeat();
      });

      state.ws.on('message', (data: Buffer) => {
        try {
          const message: ProgressMessage = JSON.parse(data.toString());
          if (message.type === 'ack') {
            state!.lastActivity = Date.now();
          }
        } catch {
          console.error('Failed to parse message from progress bar');
        }
      });

      state.ws.on('close', () => {
        stopHeartbeat();
        scheduleReconnect();
      });

      state.ws.on('error', (error: Error) => {
        console.error('WebSocket error:', error);
        state?.ws?.close();
      });
    } catch (error) {
      console.error('Failed to connect:', error);
      scheduleReconnect();
    }
  }

  function scheduleReconnect(): void {
    if (!state) return;

    if (state.reconnectTimer) {
      return;
    }

    state.reconnectTimer = setTimeout(() => {
      if (state) {
        state.reconnectTimer = null;
        connect();
      }
    }, DEFAULT_CONFIG.reconnectInterval);
  }

  function startHeartbeat(): void {
    if (!state) return;

    state.heartbeatTimer = setInterval(() => {
      if (state.ws && state.ws.readyState === WebSocket.OPEN) {
        const heartbeat = createHeartbeatMessage();
        state.ws.send(JSON.stringify(heartbeat));
      }
    }, DEFAULT_CONFIG.heartbeatInterval);
  }

  function stopHeartbeat(): void {
    if (state?.heartbeatTimer) {
      clearInterval(state.heartbeatTimer);
      state.heartbeatTimer = null;
    }
  }

  function sendMessage(message: ProgressMessage): void {
    if (!state) return;

    if (state.ws && state.ws.readyState === WebSocket.OPEN) {
      state.ws.send(JSON.stringify(message));
    }
  }

  function handleTaskUpdate(task: TaskInfo): void {
    if (!state) return;

    state.activeTasks.set(task.id, task);
    state.lastActivity = Date.now();

    const message = createStatusMessage(
      task.id,
      task.name,
      task.progress,
      task.status,
      task.adapter
    );
    sendMessage(message);

    if (task.status === 'completed' || task.status === 'error') {
      setTimeout(() => {
        state?.activeTasks.delete(task.id);
      }, 3000);
    }
  }

  function checkForTasks(): void {
    if (!state) return;

    const interval = config.get<number>('updateInterval', DEFAULT_CONFIG.updateInterval);
    const updateInterval = Math.max(500, interval);

    setInterval(() => {
      detectors.forEach(detector => {
        detector.detectTasks().then(tasks => {
          tasks.forEach(task => {
            const existingTask = state!.activeTasks.get(task.id);
            if (!existingTask || existingTask.progress !== task.progress || existingTask.status !== task.status) {
              handleTaskUpdate(task);
            }
          });
        });
      });
    }, updateInterval);
  }

  context.subscriptions.push(
    vscode.commands.registerCommand('vibeCodingProgressBar.toggle', () => {
      if (state?.ws) {
        state.ws.close();
        state.ws = null;
        vscode.window.showInformationMessage('Progress Bar disconnected');
      } else {
        connect();
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('vibeCodingProgressBar.resetAll', () => {
      if (state) {
        state.activeTasks.clear();
        statusBar.text = '$(check) Vibe Coding';
        setTimeout(() => {
          statusBar.text = '$(sync~spin) Vibe Coding';
        }, 2000);
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('vibeCodingProgressBar.connect', () => {
      connect();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('vibeCodingProgressBar.disconnect', () => {
      if (state?.ws) {
        state.ws.close();
        state.ws = null;
      }
    })
  );

  connect();
  checkForTasks();
}

export function deactivate(): void {
  if (state?.ws) {
    state.ws.close();
  }
}
