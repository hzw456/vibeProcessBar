import { describe, it, expect, beforeEach } from 'vitest';
import { useProgressStore, ProgressTask } from '../src/stores/progressStore';

describe('ProgressStore', () => {
  beforeEach(() => {
    useProgressStore.setState({
      tasks: [],
      currentTaskId: null,
      history: [],
      settings: {
        language: 'en',
        theme: 'dark',
        fontSize: 14,
        opacity: 0.85,
        alwaysOnTop: true,
        autoStart: false,
        notifications: true,
        sound: true,
        soundVolume: 0.7,
        httpPort: 31415,
        customColors: {
          primaryColor: '',
          backgroundColor: '',
          textColor: '',
        },
        reminderThreshold: 100,
        doNotDisturb: false,
        doNotDisturbStart: '22:00',
        doNotDisturbEnd: '08:00',
        windowVisible: true,
      },
    });
  });

  describe('addTask', () => {
    it('should add a new task with idle status', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');

      const tasks = useProgressStore.getState().tasks;
      expect(tasks).toHaveLength(1);
      expect(tasks[0].name).toBe('Test Task');
      expect(tasks[0].status).toBe('idle');
      expect(tasks[0].progress).toBe(0);
      expect(tasks[0].tokens).toBe(0);
      expect(taskId).toBeDefined();
    });

    it('should set new task as current task', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');

      expect(useProgressStore.getState().currentTaskId).toBe(taskId);
    });

    it('should add task with IDE info', () => {
      useProgressStore.getState().addTask('Test Task', undefined, 'cursor', 'test.ts');

      const tasks = useProgressStore.getState().tasks;
      expect(tasks[0].ide).toBe('cursor');
      expect(tasks[0].windowTitle).toBe('test.ts');
    });
  });

  describe('updateProgress', () => {
    it('should update task progress', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().updateProgress(taskId, 50);

      const tasks = useProgressStore.getState().tasks;
      expect(tasks[0].progress).toBe(50);
    });

    it('should clamp progress to 0-100', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');

      useProgressStore.getState().updateProgress(taskId, 150);
      expect(useProgressStore.getState().tasks[0].progress).toBe(100);

      useProgressStore.getState().updateProgress(taskId, -10);
      expect(useProgressStore.getState().tasks[0].progress).toBe(0);
    });
  });

  describe('updateTokens', () => {
    it('should update task tokens', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().updateTokens(taskId, 100);

      expect(useProgressStore.getState().tasks[0].tokens).toBe(100);
    });

    it('should increment tokens when increment is true', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().updateTokens(taskId, 100);
      useProgressStore.getState().updateTokens(taskId, 50, true);

      expect(useProgressStore.getState().tasks[0].tokens).toBe(150);
    });

    it('should set absolute tokens when increment is false', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().updateTokens(taskId, 100);
      useProgressStore.getState().updateTokens(taskId, 200, false);

      expect(useProgressStore.getState().tasks[0].tokens).toBe(200);
    });
  });

  describe('updateStatus', () => {
    it('should update task status', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().updateStatus(taskId, 'running');

      expect(useProgressStore.getState().tasks[0].status).toBe('running');
    });
  });

  describe('completeTask', () => {
    it('should mark task as completed', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().completeTask(taskId);

      const tasks = useProgressStore.getState().tasks;
      expect(tasks[0].status).toBe('completed');
      expect(tasks[0].progress).toBe(100);
    });

    it('should set total tokens', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().completeTask(taskId, 500);

      expect(useProgressStore.getState().tasks[0].tokens).toBe(500);
    });
  });

  describe('resetTask', () => {
    it('should reset task progress and status', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().updateProgress(taskId, 75);
      useProgressStore.getState().updateTokens(taskId, 100);
      useProgressStore.getState().updateStatus(taskId, 'running');

      useProgressStore.getState().resetTask(taskId);

      const tasks = useProgressStore.getState().tasks;
      expect(tasks[0].progress).toBe(0);
      expect(tasks[0].tokens).toBe(0);
      expect(tasks[0].status).toBe('idle');
    });
  });

  describe('removeTask', () => {
    it('should remove task', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().removeTask(taskId);

      expect(useProgressStore.getState().tasks).toHaveLength(0);
    });

    it('should clear currentTaskId when removing current task', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().removeTask(taskId);

      expect(useProgressStore.getState().currentTaskId).toBeNull();
    });
  });

  describe('settings', () => {
    it('should update theme', () => {
      useProgressStore.getState().setTheme('light');

      expect(useProgressStore.getState().settings.theme).toBe('light');
    });

    it('should update opacity with clamping', () => {
      useProgressStore.getState().setOpacity(1.5);
      expect(useProgressStore.getState().settings.opacity).toBe(1);

      useProgressStore.getState().setOpacity(0.05);
      expect(useProgressStore.getState().settings.opacity).toBe(0.1);
    });

    it('should update alwaysOnTop', () => {
      useProgressStore.getState().setAlwaysOnTop(false);

      expect(useProgressStore.getState().settings.alwaysOnTop).toBe(false);
    });

    it('should update HTTP port with validation', () => {
      useProgressStore.getState().setHttpPort(8080);
      expect(useProgressStore.getState().settings.httpPort).toBe(8080);

      useProgressStore.getState().setHttpPort(80);
      expect(useProgressStore.getState().settings.httpPort).toBe(1024);

      useProgressStore.getState().setHttpPort(70000);
      expect(useProgressStore.getState().settings.httpPort).toBe(65535);
    });

    it('should update soundVolume with clamping', () => {
      useProgressStore.getState().setSoundVolume(0.8);
      expect(useProgressStore.getState().settings.soundVolume).toBe(0.8);

      useProgressStore.getState().setSoundVolume(1.5);
      expect(useProgressStore.getState().settings.soundVolume).toBe(1);

      useProgressStore.getState().setSoundVolume(-0.5);
      expect(useProgressStore.getState().settings.soundVolume).toBe(0);
    });
  });

  describe('history', () => {
    it('should add task to history', () => {
      const task: ProgressTask = {
        id: 'test-id',
        name: 'Completed Task',
        progress: 100,
        tokens: 0,
        status: 'completed',
        startTime: Date.now(),
        endTime: Date.now(),
      };

      useProgressStore.getState().addToHistory(task);

      expect(useProgressStore.getState().history).toHaveLength(1);
      expect(useProgressStore.getState().history[0].name).toBe('Completed Task');
    });

    it('should limit history to 50 tasks', () => {
      for (let i = 0; i < 60; i++) {
        useProgressStore.getState().addToHistory({
          id: `task-${i}`,
          name: `Task ${i}`,
          progress: 100,
          tokens: 0,
          status: 'completed',
          startTime: Date.now(),
          endTime: Date.now(),
        });
      }

      expect(useProgressStore.getState().history).toHaveLength(50);
    });

    it('should clear history', () => {
      useProgressStore.getState().addToHistory({
        id: 'test-id',
        name: 'Test Task',
        progress: 100,
        tokens: 0,
        status: 'completed',
        startTime: Date.now(),
        endTime: Date.now(),
      });

      useProgressStore.getState().clearHistory();

      expect(useProgressStore.getState().history).toHaveLength(0);
    });
  });
});
