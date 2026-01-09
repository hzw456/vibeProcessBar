import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { debug, error } from '../utils/logger';
import type { SupportedLanguage } from '../utils/i18n';

// Check if we're running in Tauri
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Safe invoke wrapper that only works in Tauri environment
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  if (!isTauri) {
    debug(`Skipping invoke "${cmd}" - not in Tauri environment`);
    return null;
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

export interface ProgressTask {
  id: string;
  name: string;
  progress: number;
  tokens: number;
  status: 'idle' | 'running' | 'completed' | 'error' | 'armed' | 'active' | 'registered';
  startTime: number;
  endTime?: number;
  adapter?: string;
  ide?: string;
  windowTitle?: string;
  projectPath?: string;
  activeFile?: string;
}

interface AppSettings {
  language: SupportedLanguage;
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
  windowVisible: boolean;
}

const defaultSettings: AppSettings = {
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
};

export const useProgressStore = defineStore('progress', () => {
  // State
  const tasks = ref<ProgressTask[]>([]);
  const currentTaskId = ref<string | null>(null);
  const history = ref<ProgressTask[]>([]);
  const settings = ref<AppSettings>({ ...defaultSettings });

  // Getters
  const currentTask = computed(() =>
    tasks.value.find(t => t.id === currentTaskId.value) || null
  );

  // Actions
  async function loadSettings() {
    try {
      const loadedSettings = await safeInvoke<AppSettings>('get_app_settings');
      if (loadedSettings) {
        settings.value = loadedSettings;
      }
    } catch (err) {
      error('Failed to load settings', { error: String(err) });
    }
  }

  function setSettings(newSettings: AppSettings) {
    settings.value = newSettings;
  }

  function addTask(name: string, adapter?: string, ide?: string, windowTitle?: string): string {
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
    tasks.value.push(newTask);
    currentTaskId.value = id;
    return id;
  }

  function removeTask(id: string) {
    tasks.value = tasks.value.filter(t => t.id !== id);
    if (currentTaskId.value === id) {
      currentTaskId.value = null;
    }
  }

  function setCurrentTask(id: string | null) {
    currentTaskId.value = id;
  }

  function updateProgress(id: string, progress: number) {
    const task = tasks.value.find(t => t.id === id);
    if (task) {
      task.progress = Math.min(100, Math.max(0, progress));
    }
  }

  function updateTokens(id: string, tokens: number, increment?: boolean) {
    const task = tasks.value.find(t => t.id === id);
    if (task) {
      task.tokens = increment ? task.tokens + tokens : tokens;
    }
  }

  function updateStatus(id: string, status: ProgressTask['status']) {
    const task = tasks.value.find(t => t.id === id);
    if (task) {
      task.status = status;
      if (status === 'completed' || status === 'error') {
        task.endTime = Date.now();
      }
    }
  }

  function completeTask(id: string, totalTokens?: number) {
    const task = tasks.value.find(t => t.id === id);
    if (task) {
      task.progress = 100;
      task.status = 'completed';
      task.endTime = Date.now();
      if (totalTokens !== undefined) {
        task.tokens = totalTokens;
      }
    }
  }

  function resetTask(id: string) {
    const task = tasks.value.find(t => t.id === id);
    if (task) {
      task.progress = 0;
      task.tokens = 0;
      task.status = 'idle';
      task.startTime = Date.now();
      task.endTime = undefined;
    }
  }

  async function updateSettingAndSync<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    settings.value[key] = value;
    try {
      await safeInvoke('update_app_settings', { newSettings: settings.value });
    } catch (err) {
      error('Failed to update settings', { error: String(err) });
    }
  }

  function setLanguage(language: SupportedLanguage) {
    updateSettingAndSync('language', language);
  }

  function setTheme(theme: AppSettings['theme']) {
    updateSettingAndSync('theme', theme);
  }

  function setFontSize(fontSize: number) {
    updateSettingAndSync('fontSize', Math.max(10, Math.min(24, fontSize)));
  }

  function setOpacity(opacity: number) {
    updateSettingAndSync('opacity', Math.min(1, Math.max(0.1, opacity)));
  }

  function setAlwaysOnTop(value: boolean) {
    updateSettingAndSync('alwaysOnTop', value);
  }

  function setAutoStart(value: boolean) {
    updateSettingAndSync('autoStart', value);
  }

  function setNotifications(value: boolean) {
    updateSettingAndSync('notifications', value);
  }

  function setSound(value: boolean) {
    updateSettingAndSync('sound', value);
  }

  function setSoundVolume(value: number) {
    updateSettingAndSync('soundVolume', Math.min(1, Math.max(0, value)));
  }

  function setHttpPort(value: number) {
    updateSettingAndSync('httpPort', Math.max(1024, Math.min(65535, value)));
  }

  function setCustomColors(colors: Partial<AppSettings['customColors']>) {
    settings.value.customColors = { ...settings.value.customColors, ...colors };
    safeInvoke('update_app_settings', { newSettings: settings.value }).catch((err: unknown) =>
      error('Failed to update settings', { error: String(err) })
    );
  }

  function setReminderThreshold(value: number) {
    updateSettingAndSync('reminderThreshold', Math.min(100, Math.max(0, value)));
  }

  function setDoNotDisturb(value: boolean) {
    updateSettingAndSync('doNotDisturb', value);
  }

  function setDoNotDisturbStart(value: string) {
    updateSettingAndSync('doNotDisturbStart', value);
  }

  function setDoNotDisturbEnd(value: string) {
    updateSettingAndSync('doNotDisturbEnd', value);
  }

  function setWindowVisible(value: boolean) {
    settings.value.windowVisible = value;
    safeInvoke('set_window_visibility', { visible: value }).catch((err: unknown) =>
      error('Failed to set window visibility', { error: String(err) })
    );
  }

  function addToHistory(task: ProgressTask) {
    history.value = [task, ...history.value].slice(0, 50);
  }

  function clearHistory() {
    history.value = [];
  }

  async function syncFromHttpApi() {
    try {
      const port = settings.value.httpPort;
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
            projectPath: apiTask.project_path,
            activeFile: apiTask.active_file,
          }));

          const mergedTasks = apiTasks.map(apiTask => {
            const existing = tasks.value.find(t => t.id === apiTask.id);
            if (existing) {
              let endTime = apiTask.endTime || existing.endTime;
              if (apiTask.status === 'completed' && !endTime) {
                endTime = Date.now();
              }
              const startTime = apiTask.startTime > 0 ? apiTask.startTime : existing.startTime;
              return { ...existing, ...apiTask, startTime, endTime };
            }
            if (apiTask.status === 'completed' && !apiTask.endTime) {
              return { ...apiTask, endTime: Date.now() };
            }
            return apiTask;
          });

          tasks.value = mergedTasks;
          if (data.currentTask?.id) {
            currentTaskId.value = data.currentTask.id;
          }
        }
      }
    } catch (err) {
      error('Failed to sync from API', { error: String(err) });
    }
  }

  return {
    // State
    tasks,
    currentTaskId,
    history,
    settings,
    // Getters
    currentTask,
    // Actions
    loadSettings,
    setSettings,
    addTask,
    removeTask,
    setCurrentTask,
    updateProgress,
    updateTokens,
    updateStatus,
    completeTask,
    resetTask,
    setLanguage,
    setTheme,
    setFontSize,
    setOpacity,
    setAlwaysOnTop,
    setAutoStart,
    setNotifications,
    setSound,
    setSoundVolume,
    setHttpPort,
    setCustomColors,
    setReminderThreshold,
    setDoNotDisturb,
    setDoNotDisturbStart,
    setDoNotDisturbEnd,
    setWindowVisible,
    addToHistory,
    clearHistory,
    syncFromHttpApi,
  };
});
