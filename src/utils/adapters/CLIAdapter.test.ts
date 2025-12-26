import { describe, it, expect, vi } from 'vitest';
import { CLIAdapter } from './CLIAdapter';
import { TaskInfo } from '../adapters';

describe('CLIAdapter', () => {
  it('should have correct name and version', () => {
    const adapter = new CLIAdapter();
    expect(adapter.name).toBe('cli');
    expect(adapter.version).toBe('1.0.0');
  });

  describe('isAvailable', () => {
    it('should return true by default', async () => {
      const adapter = new CLIAdapter();
      const available = await adapter.isAvailable();
      expect(available).toBe(true);
    });
  });

  describe('startTask', () => {
    it('should return a task id with cli prefix', async () => {
      const adapter = new CLIAdapter();
      const taskId = await adapter.startTask('Test Task');
      expect(taskId).toMatch(/^cli-\d+$/);
    });

    it('should notify progress when starting task', async () => {
      const adapter = new CLIAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.startTask('CLI Task');
      
      expect(callback).toHaveBeenCalledTimes(1);
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.name).toBe('Test Task');
      expect(task.adapter).toBe('cli');
    });
  });

  describe('updateProgress', () => {
    it('should update progress correctly', async () => {
      const adapter = new CLIAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.updateProgress('test-id', 80);
      
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.progress).toBe(80);
      expect(task.status).toBe('running');
    });

    it('should complete at 100%', async () => {
      const adapter = new CLIAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.updateProgress('test-id', 100);
      
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.status).toBe('completed');
    });
  });

  describe('completeTask', () => {
    it('should complete task correctly', async () => {
      const adapter = new CLIAdapter();
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
