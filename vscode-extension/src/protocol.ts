export interface ProgressMessage {
  type: 'status' | 'progress' | 'complete' | 'error' | 'heartbeat' | 'ack';
  timestamp: string;
  data: {
    taskId: string;
    name: string;
    progress: number;
    status: 'idle' | 'running' | 'completed' | 'error';
    adapter: string;
    workspace?: string;
    file?: string;
    metadata?: Record<string, unknown>;
  };
}

export interface ConnectionConfig {
  host: string;
  port: number;
  reconnectInterval: number;
  heartbeatInterval: number;
}

export const DEFAULT_CONFIG: ConnectionConfig = {
  host: 'localhost',
  port: 31415,
  reconnectInterval: 5000,
  heartbeatInterval: 10000,
};

export function createStatusMessage(
  taskId: string,
  name: string,
  progress: number,
  status: ProgressMessage['data']['status'],
  adapter: string
): ProgressMessage {
  return {
    type: status === 'completed' ? 'complete' : 'status',
    timestamp: new Date().toISOString(),
    data: {
      taskId,
      name,
      progress,
      status,
      adapter,
    },
  };
}

export function createProgressMessage(
  taskId: string,
  progress: number,
  adapter: string
): ProgressMessage {
  return {
    type: 'progress',
    timestamp: new Date().toISOString(),
    data: {
      taskId,
      name: '',
      progress,
      status: progress >= 100 ? 'completed' : 'running',
      adapter,
    },
  };
}

export function createHeartbeatMessage(): ProgressMessage {
  return {
    type: 'heartbeat',
    timestamp: new Date().toISOString(),
    data: {
      taskId: 'heartbeat',
      name: 'Heartbeat',
      progress: 0,
      status: 'idle',
      adapter: 'vscode-extension',
    },
  };
}
