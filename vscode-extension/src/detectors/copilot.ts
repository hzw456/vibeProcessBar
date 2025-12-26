import * as vscode from 'vscode';
import { TaskInfo, TaskDetector } from '../types';

export class CopilotDetector implements TaskDetector {
  name = 'copilot';

  constructor(private config: vscode.WorkspaceConfiguration) {}

  async isAvailable(): Promise<boolean> {
    const enabled = this.config.get<boolean>('copilot.enabled', true);
    if (!enabled) {
      return false;
    }

    const copilotExtension = vscode.extensions.getExtension('GitHub.copilot');
    return !!copilotExtension;
  }

  async detectTasks(): Promise<TaskInfo[]> {
    const tasks: TaskInfo[] = [];

    if (!await this.isAvailable()) {
      return tasks;
    }

    try {
      const statusBar = vscode.window.createStatusBarItem('copilot-status', vscode.StatusBarAlignment.Right);
      statusBar.show();

      const copilotStatus = statusBar.text;

      if (copilotStatus.includes('$(loading~spin)') || copilotStatus.includes('$(sync~spin)')) {
        const task: TaskInfo = {
          id: `copilot-${Date.now()}`,
          name: 'GitHub Copilot',
          progress: 50,
          status: 'running',
          adapter: 'copilot',
          startTime: Date.now(),
          metadata: {
            status: copilotStatus,
          },
        };
        tasks.push(task);
      }

      statusBar.dispose();
    } catch {
      console.log('Could not detect Copilot status');
    }

    return tasks;
  }
}
