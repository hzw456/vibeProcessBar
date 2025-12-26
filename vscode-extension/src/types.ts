export interface TaskInfo {
  id: string;
  name: string;
  progress: number;
  status: 'idle' | 'running' | 'completed' | 'error';
  adapter: string;
  startTime: number;
  endTime?: number;
  workspace?: string;
  file?: string;
  metadata?: Record<string, unknown>;
}

export interface TaskDetector {
  name: string;
  detectTasks(): Promise<TaskInfo[]>;
  isAvailable(): Promise<boolean>;
}
