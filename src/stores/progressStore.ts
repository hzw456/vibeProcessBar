import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface ProgressTask {
  id: string;
  name: string;
  progress: number;
  status: 'idle' | 'running' | 'completed' | 'error';
  startTime: number;
  endTime?: number;
  adapter?: string;
}

interface ProgressState {
  tasks: ProgressTask[];
  currentTaskId: string | null;
  settings: {
    theme: 'dark' | 'light';
    opacity: number;
    alwaysOnTop: boolean;
    autoStart: boolean;
    notifications: boolean;
    sound: boolean;
  };
  addTask: (name: string, adapter?: string) => string;
  removeTask: (id: string) => void;
  setCurrentTask: (id: string | null) => void;
  updateProgress: (id: string, progress: number) => void;
  updateStatus: (id: string, status: ProgressTask['status']) => void;
  completeTask: (id: string) => void;
  resetTask: (id: string) => void;
  setTheme: (theme: 'dark' | 'light') => void;
  setOpacity: (opacity: number) => void;
  setAlwaysOnTop: (value: boolean) => void;
  setAutoStart: (value: boolean) => void;
  setNotifications: (value: boolean) => void;
  setSound: (value: boolean) => void;
}

export const useProgressStore = create<ProgressState>()(
  persist(
    (set) => ({
      tasks: [],
      currentTaskId: null,
      settings: {
        theme: 'dark',
        opacity: 0.85,
        alwaysOnTop: true,
        autoStart: false,
        notifications: true,
        sound: true,
      },

      addTask: (name, adapter) => {
        const id = Date.now().toString();
        const newTask: ProgressTask = {
          id,
          name,
          progress: 0,
          status: 'idle',
          startTime: Date.now(),
          adapter,
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

      updateStatus: (id, status) => {
        set((state) => ({
          tasks: state.tasks.map((t) =>
            t.id === id
              ? { ...t, status, endTime: status === 'completed' || status === 'error' ? Date.now() : undefined }
              : t
          ),
        }));
      },

      completeTask: (id) => {
        set((state) => ({
          tasks: state.tasks.map((t) =>
            t.id === id
              ? { ...t, progress: 100, status: 'completed', endTime: Date.now() }
              : t
          ),
        }));
      },

      resetTask: (id) => {
        set((state) => ({
          tasks: state.tasks.map((t) =>
            t.id === id
              ? { ...t, progress: 0, status: 'idle', startTime: Date.now(), endTime: undefined }
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
    }),
    {
      name: 'vibe-progress-storage',
      partialize: (state) => ({
        tasks: state.tasks,
        currentTaskId: state.currentTaskId,
        settings: state.settings,
      }),
    }
  )
);
