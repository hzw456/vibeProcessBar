import * as vscode from 'vscode';
import { TaskInfo, TaskDetector } from '../types';

export class ClaudeCodeDetector implements TaskDetector {
  name = 'claude-code';

  constructor(private config: vscode.WorkspaceConfiguration) {}

  async isAvailable(): Promise<boolean> {
    const enabled = this.config.get<boolean>('claudeCode.enabled', true);
    if (!enabled) {
      return false;
    }

    const claudeExtension = vscode.extensions.getExtension('Anthropic.claude-code');
    return !!claudeExtension;
  }

  async detectTasks(): Promise<TaskInfo[]> {
    const tasks: TaskInfo[] = [];

    if (!await this.isAvailable()) {
      return tasks;
    }

    const terminals = vscode.window.terminals;
    for (const terminal of terminals) {
      if (terminal.name.toLowerCase().includes('claude') || 
          terminal.name.toLowerCase().includes('anthropic')) {
        const task: TaskInfo = {
          id: `claude-code-${Date.now()}`,
          name: 'Claude Code',
          progress: 75,
          status: 'running',
          adapter: 'claude-code',
          startTime: Date.now(),
          metadata: {
            terminal: terminal.name,
          },
        };
        tasks.push(task);
        break;
      }
    }

    const outputChannel = vscode.window.createOutputChannel('Claude Code');
    const lines = outputChannel.lines || [];
    for (const line of lines.slice(-5)) {
      if (line.includes('Thinking') || line.includes('Processing')) {
        const task: TaskInfo = {
          id: `claude-code-output-${Date.now()}`,
          name: 'Claude Code Processing',
          progress: 50,
          status: 'running',
          adapter: 'claude-code',
          startTime: Date.now(),
          metadata: {
            lastOutput: line,
          },
        };
        tasks.push(task);
        break;
      }
    }
    outputChannel.dispose();

    return tasks;
  }
}
