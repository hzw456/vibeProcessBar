import { describe, it, expect, vi } from 'vitest';
import { ClaudeCodeAdapter } from './ClaudeCodeAdapter';
import { TaskInfo } from '../adapters';

describe('ClaudeCodeAdapter', () => {
  it('should have correct name and version', () => {
    const adapter = new ClaudeCodeAdapter();
    expect(adapter.name).toBe('claude-code');
    expect(adapter.version).toBe('1.0.0');
  });

  describe('initialize', () => {
    it('should not throw when initializing', async () => {
      const adapter = new ClaudeCodeAdapter();
      await expect(adapter.initialize()).resolves.not.toThrow();
    });
  });

  describe('destroy', () => {
    it('should not throw when destroying', async () => {
      const adapter = new ClaudeCodeAdapter();
      await expect(adapter.destroy()).resolves.not.toThrow();
    });
  });

  describe('isAvailable', () => {
    it('should return false by default', async () => {
      const adapter = new ClaudeCodeAdapter();
      const available = await adapter.isAvailable();
      expect(available).toBe(false);
    });
  });

  describe('getTasks', () => {
    it('should return empty array by default', async () => {
      const adapter = new ClaudeCodeAdapter();
      const tasks = await adapter.getTasks();
      expect(tasks).toEqual([]);
    });
  });

  describe('startTask', () => {
    it('should return a task id', async () => {
      const adapter = new ClaudeCodeAdapter();
      const taskId = await adapter.startTask('Test Task');
      expect(taskId).toMatch(/^claude-\d+$/);
    });

    it('should notify progress when starting task', async () => {
      const adapter = new ClaudeCodeAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.startTask('Test Task');
      
      expect(callback).toHaveBeenCalledTimes(1);
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.name).toBe('Test Task');
      expect(task.status).toBe('running');
      expect(task.progress).toBe(0);
      expect(task.adapter).toBe('claude-code');
    });
  });

  describe('updateProgress', () => {
    it('should update progress and notify', async () => {
      const adapter = new ClaudeCodeAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.updateProgress('test-id', 50);
      
      expect(callback).toHaveBeenCalledTimes(1);
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.progress).toBe(50);
      expect(task.status).toBe('running');
    });

    it('should set status to completed when progress is 100', async () => {
      const adapter = new ClaudeCodeAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.updateProgress('test-id', 100);
      
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.status).toBe('completed');
    });
  });

  describe('completeTask', () => {
    it('should complete task with 100% progress', async () => {
      const adapter = new ClaudeCodeAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.completeTask('test-id');
      
      expect(callback).toHaveBeenCalledTimes(1);
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.progress).toBe(100);
      expect(task.status).toBe('completed');
      expect(task.endTime).toBeDefined();
    });
  });

  describe('onProgress', () => {
    it('should return unsubscribe function', async () => {
      const adapter = new ClaudeCodeAdapter();
      const callback = vi.fn();
      const unsubscribe = adapter.onProgress(callback);
      
      await adapter.startTask('Task 1');
      expect(callback).toHaveBeenCalledTimes(1);
      
      unsubscribe();
      
      await adapter.startTask('Task 2');
      expect(callback).toHaveBeenCalledTimes(1);
    });
  });
});
