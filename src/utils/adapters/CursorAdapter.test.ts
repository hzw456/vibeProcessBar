import { describe, it, expect, vi } from 'vitest';
import { CursorAdapter } from './CursorAdapter';
import { TaskInfo } from '../adapters';

describe('CursorAdapter', () => {
  it('should have correct name and version', () => {
    const adapter = new CursorAdapter();
    expect(adapter.name).toBe('cursor');
    expect(adapter.version).toBe('1.0.0');
  });

  describe('isAvailable', () => {
    it('should return false by default', async () => {
      const adapter = new CursorAdapter();
      const available = await adapter.isAvailable();
      expect(available).toBe(false);
    });
  });

  describe('startTask', () => {
    it('should return a task id with cursor prefix', async () => {
      const adapter = new CursorAdapter();
      const taskId = await adapter.startTask('Test Task');
      expect(taskId).toMatch(/^cursor-\d+$/);
    });

    it('should notify progress when starting task', async () => {
      const adapter = new CursorAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.startTask('Cursor Task');
      
      expect(callback).toHaveBeenCalledTimes(1);
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.name).toBe('Test Task');
      expect(task.adapter).toBe('cursor');
    });
  });

  describe('updateProgress', () => {
    it('should update progress correctly', async () => {
      const adapter = new CursorAdapter();
      const callback = vi.fn();
      adapter.onProgress(callback);
      
      await adapter.updateProgress('test-id', 60);
      
      const task = callback.mock.calls[0][0] as TaskInfo;
      expect(task.progress).toBe(60);
      expect(task.status).toBe('running');
    });
  });

  describe('completeTask', () => {
    it('should complete task correctly', async () => {
      const adapter = new CursorAdapter();
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
