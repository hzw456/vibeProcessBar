export interface AdapterConfig {
  name: string;
  version: string;
  enabled: boolean;
}

export interface TaskInfo {
  id: string;
  name: string;
  progress: number;
  status: 'idle' | 'running' | 'completed' | 'error';
  startTime: number;
  endTime?: number;
  adapter: string;
  metadata?: Record<string, unknown>;
}

export interface ProgressAdapter {
  name: string;
  version: string;
  initialize: () => Promise<void>;
  destroy: () => Promise<void>;
  isAvailable: () => Promise<boolean>;
  getTasks: () => Promise<TaskInfo[]>;
  onProgress: (callback: (task: TaskInfo) => void) => () => void;
  startTask: (name: string) => Promise<string>;
  updateProgress: (taskId: string, progress: number) => Promise<void>;
  completeTask: (taskId: string) => Promise<void>;
}

export abstract class BaseAdapter implements ProgressAdapter {
  name: string;
  version: string;
  protected callbacks: Set<(task: TaskInfo) => void> = new Set();
  protected isRunning = false;

  constructor(name: string, version: string) {
    this.name = name;
    this.version = version;
  }

  abstract initialize(): Promise<void>;
  abstract destroy(): Promise<void>;
  abstract isAvailable(): Promise<boolean>;
  abstract getTasks(): Promise<TaskInfo[]>;
  abstract startTask(name: string): Promise<string>;
  abstract updateProgress(taskId: string, progress: number): Promise<void>;
  abstract completeTask(taskId: string): Promise<void>;

  onProgress(callback: (task: TaskInfo) => void): () => void {
    this.callbacks.add(callback);
    return () => {
      this.callbacks.delete(callback);
    };
  }

  protected notifyProgress(task: TaskInfo): void {
    this.callbacks.forEach(cb => cb(task));
  }

  protected setRunning(running: boolean): void {
    this.isRunning = running;
  }
}

export function createAdapter(name: string): ProgressAdapter | null {
  const adapters: Record<string, () => ProgressAdapter> = {
    'copilot': () => new CopilotAdapter(),
    'claude-code': () => new ClaudeCodeAdapter(),
    'cursor': () => new CursorAdapter(),
    'cli': () => new CLIAdapter(),
  };

  const factory = adapters[name.toLowerCase()];
  return factory ? factory() : null;
}
