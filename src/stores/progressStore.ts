import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface ProgressTask {
  id: string;
  name: string;
  progress: number;
  tokens: number;
  status: 'idle' | 'running' | 'completed' | 'error';
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
}

export const useProgressStore = create<ProgressState>()(
  persist(
    (set, get) => ({
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
        set((state) => ({
          settings: { ...state.settings, theme },
        }));
      },

      setOpacity: (opacity) => {
        set((state) => ({
          settings: { ...state.settings, opacity: Math.min(1, Math.max(0.1, opacity)) },
        }));
      },

      setAlwaysOnTop: (value) => {
        set((state) => ({
          settings: { ...state.settings, alwaysOnTop: value },
        }));
      },

      setAutoStart: (value) => {
        set((state) => ({
          settings: { ...state.settings, autoStart: value },
        }));
      },

      setNotifications: (value) => {
        set((state) => ({
          settings: { ...state.settings, notifications: value },
        }));
      },

      setSound: (value) => {
        set((state) => ({
          settings: { ...state.settings, sound: value },
        }));
      },

      setSoundVolume: (value) => {
        set((state) => ({
          settings: { ...state.settings, soundVolume: Math.min(1, Math.max(0, value)) },
        }));
      },

      setHttpPort: (value) => {
        set((state) => ({
          settings: { ...state.settings, httpPort: Math.max(1024, Math.min(65535, value)) },
        }));
      },

      setCustomColors: (colors) => {
        set((state) => ({
          settings: {
            ...state.settings,
            customColors: {
              ...state.settings.customColors,
              ...colors,
            },
          },
        }));
      },

      setReminderThreshold: (value) => {
        set((state) => ({
          settings: { ...state.settings, reminderThreshold: Math.min(100, Math.max(0, value)) },
        }));
      },

      setDoNotDisturb: (value) => {
        set((state) => ({
          settings: { ...state.settings, doNotDisturb: value },
        }));
      },

      setDoNotDisturbStart: (value) => {
        set((state) => ({
          settings: { ...state.settings, doNotDisturbStart: value },
        }));
      },

      setDoNotDisturbEnd: (value) => {
        set((state) => ({
          settings: { ...state.settings, doNotDisturbEnd: value },
        }));
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
          const response = await fetch(`http://localhost:${port}/api/status`);
          if (response.ok) {
            const data = await response.json();
            if (data.currentTask) {
              const { currentTask } = data;
              const existingTask = get().tasks.find(t => t.id === currentTask.id);
              if (!existingTask) {
                set((state) => ({
                  tasks: [...state.tasks, currentTask],
                  currentTaskId: currentTask.id,
                }));
              } else {
                set((state) => ({
                  tasks: state.tasks.map(t =>
                    t.id === currentTask.id ? currentTask : t
                  ),
                  currentTaskId: currentTask.id,
                }));
              }
            }
          }
        } catch (error) {
          console.error('Failed to sync from HTTP API:', error);
        }
      },
    }),
    {
      name: 'vibe-progress-storage',
      partialize: (state) => ({
        tasks: state.tasks,
        currentTaskId: state.currentTaskId,
        history: state.history,
        settings: state.settings,
      }),
    }
  )
);
