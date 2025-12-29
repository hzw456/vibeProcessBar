import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { debug, error } from '../utils/logger';
import { invoke } from '@tauri-apps/api/core';

export interface ProgressTask {
  id: string;
  name: string;
  progress: number;
  tokens: number;
  status: 'idle' | 'running' | 'completed' | 'error' | 'armed' | 'active';
  startTime: number;
  endTime?: number;
  adapter?: string;
  ide?: string;
  windowTitle?: string;
}

interface ProgressState {
  tasks: ProgressTask[];
  currentTaskId: string | null;
  settings: {
    theme: 'dark' | 'light' | 'purple' | 'ocean' | 'forest' | 'midnight';
    fontSize: number;
    opacity: number;
    alwaysOnTop: boolean;
    autoStart: boolean;
    notifications: boolean;
    sound: boolean;
    soundVolume: number;
    httpPort: number;
    customColors: {
      primaryColor: string;
      backgroundColor: string;
      textColor: string;
    };
    reminderThreshold: number;
    doNotDisturb: boolean;
    doNotDisturbStart: string;
    doNotDisturbEnd: string;
  };
  history: ProgressTask[];
  addTask: (name: string, adapter?: string, ide?: string, windowTitle?: string) => string;
  removeTask: (id: string) => void;
  setCurrentTask: (id: string | null) => void;
  updateProgress: (id: string, progress: number) => void;
  updateTokens: (id: string, tokens: number, increment?: boolean) => void;
  updateStatus: (id: string, status: ProgressTask['status']) => void;
  completeTask: (id: string, totalTokens?: number) => void;
  resetTask: (id: string) => void;
  setTheme: (theme: 'dark' | 'light' | 'purple' | 'ocean' | 'forest' | 'midnight') => void;
  setFontSize: (size: number) => void;
  setOpacity: (opacity: number) => void;
  setAlwaysOnTop: (value: boolean) => void;
  setAutoStart: (value: boolean) => void;
  setNotifications: (value: boolean) => void;
  setSound: (value: boolean) => void;
  setSoundVolume: (value: number) => void;
  setHttpPort: (value: number) => void;
  setCustomColors: (colors: { primaryColor?: string; backgroundColor?: string; textColor?: string }) => void;
  setReminderThreshold: (value: number) => void;
  setDoNotDisturb: (value: boolean) => void;
  setDoNotDisturbStart: (value: string) => void;
  setDoNotDisturbEnd: (value: string) => void;
  addToHistory: (task: ProgressTask) => void;
  clearHistory: () => void;
  syncFromHttpApi: () => Promise<void>;
  loadSettings: () => Promise<void>;
  setSettings: (settings: ProgressState['settings']) => void;
}

export const useProgressStore = create<ProgressState>()(
  persist(
    (set, get) => ({
      tasks: [],
      currentTaskId: null,
      history: [],
      settings: {
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
      },

      loadSettings: async () => {
        try {
          const settings = await invoke<ProgressState['settings']>('get_app_settings');
          set({ settings });
        } catch (err) {
          error('Failed to load settings', { error: String(err) });
        }
      },

      setSettings: (settings) => {
        set({ settings });
      },

      addTask: (name, adapter, ide, windowTitle) => {
        const id = Date.now().toString();
        const newTask: ProgressTask = {
          id,
          name,
          progress: 0,
          tokens: 0,
          status: 'idle',
          startTime: Date.now(),
          adapter,
          ide,
          windowTitle,
        };
        set((state) => ({
          tasks: [...state.tasks, newTask],
          currentTaskId: id,
        }));
        return id;
      },

      removeTask: (id) => {
        set((state) => ({
          tasks: state.tasks.filter((t) => t.id !== id),
          currentTaskId: state.currentTaskId === id ? null : state.currentTaskId,
        }));
      },

      setCurrentTask: (id) => {
        set({ currentTaskId: id });
      },

      updateProgress: (id, progress) => {
        set((state) => ({
          tasks: state.tasks.map((t) =>
            t.id === id
              ? { ...t, progress: Math.min(100, Math.max(0, progress)) }
              : t
          ),
        }));
      },

      updateTokens: (id, tokens, increment) => {
        set((state) => ({
          tasks: state.tasks.map((t) =>
            t.id === id
              ? { ...t, tokens: increment ? t.tokens + tokens : tokens }
              : t
          ),
        }));
      },

      updateStatus: (id, status) => {
        set((state) => ({
          tasks: state.tasks.map((t) =>
            t.id === id
              ? { ...t, status, endTime: status === 'completed' || status === 'error' ? Date.now() : undefined }
              : t
          ),
        }));
      },

      completeTask: (id, totalTokens) => {
        set((state) => ({
          tasks: state.tasks.map((t) =>
            t.id === id
              ? { ...t, progress: 100, status: 'completed', endTime: Date.now(), tokens: totalTokens ?? t.tokens }
              : t
          ),
        }));
      },

      resetTask: (id) => {
        set((state) => ({
          tasks: state.tasks.map((t) =>
            t.id === id
              ? { ...t, progress: 0, tokens: 0, status: 'idle', startTime: Date.now(), endTime: undefined }
              : t
          ),
        }));
      },

      setTheme: (theme) => {
        const settings = get().settings;
        const newSettings = { ...settings, theme };
        set({ settings: newSettings }); // Optimistic update
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setFontSize: (fontSize) => {
        const settings = get().settings;
        const newSettings = { ...settings, fontSize: Math.max(10, Math.min(24, fontSize)) };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setOpacity: (opacity) => {
        const settings = get().settings;
        const newSettings = { ...settings, opacity: Math.min(1, Math.max(0.1, opacity)) };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setAlwaysOnTop: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, alwaysOnTop: value };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setAutoStart: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, autoStart: value };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setNotifications: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, notifications: value };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setSound: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, sound: value };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setSoundVolume: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, soundVolume: Math.min(1, Math.max(0, value)) };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setHttpPort: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, httpPort: Math.max(1024, Math.min(65535, value)) };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setCustomColors: (colors) => {
        const settings = get().settings;
        const newSettings = {
          ...settings,
          customColors: {
            ...settings.customColors,
            ...colors,
          },
        };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setReminderThreshold: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, reminderThreshold: Math.min(100, Math.max(0, value)) };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setDoNotDisturb: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, doNotDisturb: value };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setDoNotDisturbStart: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, doNotDisturbStart: value };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      setDoNotDisturbEnd: (value) => {
        const settings = get().settings;
        const newSettings = { ...settings, doNotDisturbEnd: value };
        set({ settings: newSettings });
        invoke('update_app_settings', { newSettings }).catch(err => error('Failed to update settings', { error: String(err) }));
      },

      addToHistory: (task) => {
        set((state) => ({
          history: [task, ...state.history].slice(0, 50),
        }));
      },

      clearHistory: () => {
        set({ history: [] });
      },

      syncFromHttpApi: async () => {
        try {
          const port = get().settings.httpPort;
          const response = await fetch(`http://127.0.0.1:${port}/api/status`);
          if (response.ok) {
            const data = await response.json();
            debug('Synced from API', { taskCount: data.taskCount });
            if (data.tasks && Array.isArray(data.tasks)) {
              const apiTasks: ProgressTask[] = data.tasks.map((apiTask: any) => ({
                id: apiTask.id,
                name: apiTask.name,
                progress: apiTask.progress,
                tokens: apiTask.tokens,
                status: apiTask.status as ProgressTask['status'],
                startTime: apiTask.start_time,
                endTime: apiTask.end_time,
                ide: apiTask.ide,
                windowTitle: apiTask.window_title,
              }));

              set((state) => {
                const mergedTasks = apiTasks.map(apiTask => {
                  const existing = state.tasks.find(t => t.id === apiTask.id);
                  if (existing) {
                    let endTime = apiTask.endTime || existing.endTime;
                    if (apiTask.status === 'completed' && !endTime) {
                      endTime = Date.now();
                    }
                    return { ...existing, ...apiTask, endTime };
                  }
                  if (apiTask.status === 'completed' && !apiTask.endTime) {
                    return { ...apiTask, endTime: Date.now() };
                  }
                  return apiTask;
                });

                return {
                  tasks: mergedTasks,
                  currentTaskId: data.currentTask?.id || state.currentTaskId,
                };
              });
            }
          }
        } catch (err) {
          error('Failed to sync from API', { error: String(err) });
        }
      },
    }),
    {
      name: 'vibe-progress-storage',
      partialize: (state) => ({
        tasks: state.tasks,
        currentTaskId: state.currentTaskId,
        history: state.history,
        // Settings are now managed by backend
        // settings: state.settings,
      }),
    }
  )
);
