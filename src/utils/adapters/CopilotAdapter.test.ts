import { describe, it, expect, vi } from 'vitest';
import { CopilotAdapter } from './CopilotAdapter';
import { TaskInfo } from '../adapters';

describe('CopilotAdapter', () => {
  it('should have correct name and version', () => {
    const adapter = new CopilotAdapter();
    expect(adapter.name).toBe('copilot');
    expect(adapter.version).toBe('1.0.0');
  });

  describe('isAvailable', () => {
    it('should return false by default', async () => {
      const adapter = new CopilotAdapter();
      const available = await adapter.isAvailable();
      expect(available).toBe(false);
    });
  });

  describe('startTask', () => {
    it('should return a task id with copilot prefix', async () => {
      const adapter = new CopilotAdapter();
      const taskId = await adapter.startTask('Test Task');
      expect(taskId).toMatch(/^copilot-\d+$/);
    });

    it('should notify progress when starting task', async () => {
      const adapter = new CopilotAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.startTask('Copilot Task');
      
      expect(callback).toHaveBeenCalledTimes(1);
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.name).toBe('Test Task');
      expect(task.adapter).toBe('copilot');
    });
  });

  describe('updateProgress', () => {
    it('should update progress correctly', async () => {
      const adapter = new CopilotAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.updateProgress('test-id', 75);
      
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.progress).toBe(75);
      expect(task.status).toBe('running');
    });

    it('should set status to completed at 100%', async () => {
      const adapter = new CopilotAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.updateProgress('test-id', 100);
      
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.status).toBe('completed');
    });
  });

  describe('completeTask', () => {
    it('should complete task with endTime', async () => {
      const adapter = new CopilotAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.completeTask('test-id');
      
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.progress).toBe(100);
      expect(task.status).toBe('completed');
      expect(task.endTime).toBeDefined();
    });
  });
});
