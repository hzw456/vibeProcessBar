export interface AdapterConfig {
  name: string;
  version: string;
  enabled: boolean;
}

export interface TaskInfo {
  id: string;
  name: string;
  status: 'idle' | 'running' | 'completed' | 'error';
  startTime: number;
  endTime?: number;
  adapter: string;
  metadata?: Record<string, unknown>;
  estimated_duration?: number;
  current_stage?: string;
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
  completeTask: (taskId: string) => Promise<void>;
}

export function createAdapter(_name: string): null {
  return null;
}
