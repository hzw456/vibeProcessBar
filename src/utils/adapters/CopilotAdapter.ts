import { BaseAdapter, TaskInfo } from '../adapters';

export class CopilotAdapter extends BaseAdapter {
  constructor() {
    super('copilot', '1.0.0');
  }

  async initialize(): Promise<void> {
    console.log('Initializing Copilot adapter');
  }

  async destroy(): Promise<void> {
    console.log('Destroying Copilot adapter');
  }

  async isAvailable(): Promise<boolean> {
    return false;
  }

  async getTasks(): Promise<TaskInfo[]> {
    return [];
  }

  async startTask(name: string): Promise<string> {
    const id = `copilot-${Date.now()}`;
    const task: TaskInfo = {
      id,
      name,
      progress: 0,
      status: 'running',
      startTime: Date.now(),
      adapter: 'copilot',
    };
    this.notifyProgress(task);
    return id;
  }

  async updateProgress(taskId: string, progress: number): Promise<void> {
    const task: TaskInfo = {
      id: taskId,
      name: 'Copilot Task',
      progress,
      status: progress >= 100 ? 'completed' : 'running',
      startTime: Date.now(),
      adapter: 'copilot',
    };
    this.notifyProgress(task);
  }

  async completeTask(taskId: string): Promise<void> {
    const task: TaskInfo = {
      id: taskId,
      name: 'Copilot Task',
      progress: 100,
      status: 'completed',
      startTime: Date.now(),
      endTime: Date.now(),
      adapter: 'copilot',
    };
    this.notifyProgress(task);
  }
}
