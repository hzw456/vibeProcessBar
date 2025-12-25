import { BaseAdapter, TaskInfo } from './adapters';

export class CursorAdapter extends BaseAdapter {
  constructor() {
    super('cursor', '1.0.0');
  }

  async initialize(): Promise<void> {
    console.log('Initializing Cursor adapter');
  }

  async destroy(): Promise<void> {
    console.log('Destroying Cursor adapter');
  }

  async isAvailable(): Promise<boolean> {
    return false;
  }

  async getTasks(): Promise<TaskInfo[]> {
    return [];
  }

  async startTask(name: string): Promise<string> {
    const id = `cursor-${Date.now()}`;
    const task: TaskInfo = {
      id,
      name,
      progress: 0,
      status: 'running',
      startTime: Date.now(),
      adapter: 'cursor',
    };
    this.notifyProgress(task);
    return id;
  }

  async updateProgress(taskId: string, progress: number): Promise<void> {
    const task: TaskInfo = {
      id: taskId,
      name: 'Cursor Task',
      progress,
      status: progress >= 100 ? 'completed' : 'running',
      startTime: Date.now(),
      adapter: 'cursor',
    };
    this.notifyProgress(task);
  }

  async completeTask(taskId: string): Promise<void> {
    const task: TaskInfo = {
      id: taskId,
      name: 'Cursor Task',
      progress: 100,
      status: 'completed',
      startTime: Date.now(),
      endTime: Date.now(),
      adapter: 'cursor',
    };
    this.notifyProgress(task);
  }
}
