import { describe, it, expect, beforeEach } from 'vitest';
import { useProgressStore, ProgressTask } from '../stores/progressStore';

describe('ProgressStore', () => {
  beforeEach(() => {
    useProgressStore.setState({
      tasks: [],
      currentTaskId: null,
      history: [],
      settings: {
        theme: 'dark',
        opacity: 0.85,
        alwaysOnTop: true,
        autoStart: false,
        notifications: true,
        sound: true,
        soundVolume: 0.7,
        vscodeEnabled: false,
        vscodePort: 31415,
        vscodeHost: 'localhost',
        customColors: {
          primaryColor: '',
          backgroundColor: '',
          textColor: '',
        },
        reminderThreshold: 100,
        doNotDisturb: false,
        doNotDisturbStart: '22:00',
        doNotDisturbEnd: '08:00',
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
      expect(taskId).toBeDefined();
    });

    it('should set new task as current task', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      
      expect(useProgressStore.getState().currentTaskId).toBe(taskId);
    });

    it('should accept adapter parameter', () => {
      const taskId = useProgressStore.getState().addTask('Test Task', 'claude-code');
      const task = useProgressStore.getState().tasks.find(t => t.id === taskId);
      
      expect(task?.adapter).toBe('claude-code');
    });
  });

  describe('removeTask', () => {
    it('should remove a task by id', () => {
      const taskId = useProgressStore.getState().addTask('Task 1');
      useProgressStore.getState().addTask('Task 2');
      
      useProgressStore.getState().removeTask(taskId);
      
      const tasks = useProgressStore.getState().tasks;
      expect(tasks).toHaveLength(1);
      expect(tasks[0].name).toBe('Task 2');
    });

    it('should clear currentTaskId when removing current task', () => {
      const taskId = useProgressStore.getState().addTask('Task 1');
      
      useProgressStore.getState().removeTask(taskId);
      
      expect(useProgressStore.getState().currentTaskId).toBeNull();
    });
  });

  describe('updateProgress', () => {
    it('should update task progress', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      
      useProgressStore.getState().updateProgress(taskId, 50);
      
      expect(useProgressStore.getState().tasks[0].progress).toBe(50);
    });

    it('should clamp progress to maximum 100', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      
      useProgressStore.getState().updateProgress(taskId, 150);
      
      expect(useProgressStore.getState().tasks[0].progress).toBe(100);
    });

    it('should clamp progress to minimum 0', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      
      useProgressStore.getState().updateProgress(taskId, -50);
      
      expect(useProgressStore.getState().tasks[0].progress).toBe(0);
    });
  });

  describe('updateStatus', () => {
    it('should update task status', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      
      useProgressStore.getState().updateStatus(taskId, 'running');
      
      expect(useProgressStore.getState().tasks[0].status).toBe('running');
    });

    it('should set endTime when status is completed', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      
      useProgressStore.getState().updateStatus(taskId, 'completed');
      
      expect(useProgressStore.getState().tasks[0].endTime).toBeDefined();
    });

    it('should set endTime when status is error', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      
      useProgressStore.getState().updateStatus(taskId, 'error');
      
      expect(useProgressStore.getState().tasks[0].endTime).toBeDefined();
    });
  });

  describe('completeTask', () => {
    it('should set task progress to 100 and status to completed', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      
      useProgressStore.getState().completeTask(taskId);
      
      const task = useProgressStore.getState().tasks[0];
      expect(task.progress).toBe(100);
      expect(task.status).toBe('completed');
      expect(task.endTime).toBeDefined();
    });
  });

  describe('resetTask', () => {
    it('should reset task to initial state', () => {
      const taskId = useProgressStore.getState().addTask('Test Task');
      useProgressStore.getState().updateProgress(taskId, 75);
      useProgressStore.getState().updateStatus(taskId, 'running');
      
      useProgressStore.getState().resetTask(taskId);
      
      const task = useProgressStore.getState().tasks[0];
      expect(task.progress).toBe(0);
      expect(task.status).toBe('idle');
      expect(task.endTime).toBeUndefined();
    });
  });

  describe('settings', () => {
    it('should update theme', () => {
      useProgressStore.getState().setTheme('purple');
      
      expect(useProgressStore.getState().settings.theme).toBe('purple');
    });

    it('should update opacity with clamping', () => {
      useProgressStore.getState().setOpacity(0.5);
      expect(useProgressStore.getState().settings.opacity).toBe(0.5);
      
      useProgressStore.getState().setOpacity(1.5);
      expect(useProgressStore.getState().settings.opacity).toBe(1);
      
      useProgressStore.getState().setOpacity(0.05);
      expect(useProgressStore.getState().settings.opacity).toBe(0.1);
    });

    it('should update alwaysOnTop', () => {
      useProgressStore.getState().setAlwaysOnTop(false);
      
      expect(useProgressStore.getState().settings.alwaysOnTop).toBe(false);
    });

    it('should update VSCode port with validation', () => {
      useProgressStore.getState().setVSCodePort(8080);
      expect(useProgressStore.getState().settings.vscodePort).toBe(8080);
      
      useProgressStore.getState().setVSCodePort(80);
      expect(useProgressStore.getState().settings.vscodePort).toBe(1024);
      
      useProgressStore.getState().setVSCodePort(70000);
      expect(useProgressStore.getState().settings.vscodePort).toBe(65535);
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
        status: 'completed',
        startTime: Date.now(),
        endTime: Date.now(),
      });
      
      useProgressStore.getState().clearHistory();
      
      expect(useProgressStore.getState().history).toHaveLength(0);
    });
  });
});
