import * as vscode from 'vscode';
import { TaskInfo, TaskDetector } from '../types';

export class CursorDetector implements TaskDetector {
  name = 'cursor';

  constructor(private config: vscode.WorkspaceConfiguration) {}

  async isAvailable(): Promise<boolean> {
    const enabled = this.config.get<boolean>('cursor.enabled', true);
    if (!enabled) {
      return false;
    }

    const cursorExtension = vscode.extensions.getExtension('Cursor');
    return !!cursorExtension;
  }

  async detectTasks(): Promise<TaskInfo[]> {
    const tasks: TaskInfo[] = [];

    if (!await this.isAvailable()) {
      return tasks;
    }

    const statusBar = vscode.window.createStatusBarItem('cursor-status', vscode.StatusBarAlignment.Right);
    statusBar.show();

    const statusText = statusBar.text;

    if (statusText.includes('AI') || statusText.includes('Generate') || statusText.includes('Edit')) {
      const task: TaskInfo = {
        id: `cursor-${Date.now()}`,
        name: 'Cursor AI',
        progress: 50,
        status: 'running',
        adapter: 'cursor',
        startTime: Date.now(),
        metadata: {
          status: statusText,
        },
      };
      tasks.push(task);
    }

    statusBar.dispose();

    return tasks;
  }
}
