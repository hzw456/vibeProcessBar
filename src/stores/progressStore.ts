import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { debug, error } from '../utils/logger';
import type { SupportedLanguage } from '../utils/i18n';
import { setLanguage as setI18nLanguage } from '../utils/i18n';

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
  displayName?: string;  // 显示名称 (去掉 IDE 前缀)
  progress: number;
  tokens: number;
  status: 'armed' | 'running' | 'completed' | 'idle';  // simplified states
  isFocused?: boolean;  // window has focus
  startTime: number;
  endTime?: number;
  adapter?: string;
  ide?: string;
  windowTitle?: string;
  projectPath?: string;
  activeFile?: string;
}

export interface AppSettings {
  language: SupportedLanguage;
  theme: 'dark' | 'purple' | 'ocean' | 'forest' | 'midnight';
  fontSize: number;
  opacity: number;
  alwaysOnTop: boolean;
  autoStart: boolean;
  sound: boolean;
  soundVolume: number;
  httpHost: string;
  httpPort: number;
  windowVisible: boolean;
  blockPluginStatus: boolean; // 屏蔽插件状态上报
}

const defaultSettings: AppSettings = {
  language: 'en',
  theme: 'dark',
  fontSize: 14,
  opacity: 0.85,
  alwaysOnTop: true,
  autoStart: false,
  sound: true,
  soundVolume: 0.7,
  httpHost: '127.0.0.1',
  httpPort: 31415,
  windowVisible: true,
  blockPluginStatus: true, // 默认开启屏蔽
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

  // 更新托盘菜单翻译
  async function updateTrayTranslations(language: SupportedLanguage) {
    try {
      // 从翻译文件加载托盘翻译
      const response = await fetch(`/locales/${language}/translation.json`);
      if (response.ok) {
        const messages = await response.json();
        const tray = messages.tray || {};
        await safeInvoke('update_tray_translations', {
          translations: {
            showWindow: tray.showWindow || '☀ Show Window',
            hideWindow: tray.hideWindow || '☾ Hide Window',
            settings: tray.settings || 'Settings',
            quit: tray.quit || 'Quit',
            noTasks: tray.noTasks || 'No tasks',
            tasks: tray.tasks || 'Tasks',
          }
        });
      }
    } catch (err) {
      error('Failed to update tray translations', { error: String(err) });
    }
  }

  // Actions
  async function loadSettings() {
    try {
      const loadedSettings = await safeInvoke<AppSettings>('get_app_settings');
      if (loadedSettings) {
        const oldLanguage = settings.value.language;
        settings.value = loadedSettings;

        // 应用加载的设置
        setI18nLanguage(loadedSettings.language);
        document.documentElement.setAttribute('data-theme', loadedSettings.theme);
        document.documentElement.style.setProperty('--app-font-size', `${loadedSettings.fontSize}px`);
        document.documentElement.style.setProperty('--app-opacity', loadedSettings.opacity.toString());

        // 仅当语言改变时才更新托盘翻译，避免重复请求和闪烁
        if (oldLanguage !== loadedSettings.language) {
          updateTrayTranslations(loadedSettings.language);
        }
      }
    } catch (err) {
      error('Failed to load settings', { error: String(err) });
    }
  }

  function setSettings(newSettings: AppSettings) {
    settings.value = newSettings;

    // 总是应用所有设置
    setI18nLanguage(newSettings.language);
    document.documentElement.setAttribute('data-theme', newSettings.theme);
    document.documentElement.style.setProperty('--app-font-size', `${newSettings.fontSize}px`);
    document.documentElement.style.setProperty('--app-opacity', newSettings.opacity.toString());
  }

  function addTask(name: string, adapter?: string, ide?: string, windowTitle?: string): string {
    const id = Date.now().toString();
    const newTask: ProgressTask = {
      id,
      name,
      progress: 0,
      tokens: 0,
      status: 'armed',
      isFocused: false,
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
      if (status === 'completed') {
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
      task.status = 'armed';
      task.isFocused = false;
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
    // Actually change the i18n locale
    setI18nLanguage(language);
    // Update tray menu translations
    updateTrayTranslations(language);
  }

  function setTheme(theme: AppSettings['theme']) {
    updateSettingAndSync('theme', theme);
    // Apply theme to document
    document.documentElement.setAttribute('data-theme', theme);
  }

  function setFontSize(fontSize: number) {
    const clampedSize = Math.max(12, Math.min(18, fontSize));
    updateSettingAndSync('fontSize', clampedSize);
    // Apply font size to document
    document.documentElement.style.setProperty('--app-font-size', `${clampedSize}px`);
  }

  function setOpacity(opacity: number) {
    const clampedOpacity = Math.min(1, Math.max(0.5, opacity));
    updateSettingAndSync('opacity', clampedOpacity);
    // Apply opacity to document
    document.documentElement.style.setProperty('--app-opacity', clampedOpacity.toString());
  }

  async function setAlwaysOnTop(value: boolean) {
    updateSettingAndSync('alwaysOnTop', value);
    // Apply to window
    try {
      await safeInvoke('set_always_on_top', { alwaysOnTop: value });
    } catch (err) {
      error('Failed to set always on top', { error: String(err) });
    }
  }

  async function setAutoStart(value: boolean) {
    updateSettingAndSync('autoStart', value);
    // Apply to system
    try {
      await safeInvoke('set_auto_start', { enabled: value });
    } catch (err) {
      error('Failed to set auto start', { error: String(err) });
    }
  }

  function setSound(value: boolean) {
    updateSettingAndSync('sound', value);
  }

  function setSoundVolume(value: number) {
    updateSettingAndSync('soundVolume', Math.min(1, Math.max(0, value)));
  }

  function setHttpHost(value: string) {
    updateSettingAndSync('httpHost', value);
  }

  function setHttpPort(value: number) {
    updateSettingAndSync('httpPort', Math.max(1024, Math.min(65535, value)));
  }

  function setBlockPluginStatus(value: boolean) {
    updateSettingAndSync('blockPluginStatus', value);
  }

  async function setWindowVisible(value: boolean) {
    settings.value.windowVisible = value;
    try {
      await safeInvoke('set_window_visibility', { visible: value });
      await safeInvoke('update_app_settings', { newSettings: settings.value });
    } catch (err) {
      error('Failed to set window visibility', { error: String(err) });
    }
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
            displayName: apiTask.display_name || apiTask.name,
            progress: apiTask.progress,
            tokens: apiTask.tokens,
            status: apiTask.status as ProgressTask['status'],
            isFocused: apiTask.is_focused || false,
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

  async function refreshSettings() {
    await loadSettings();
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
    refreshSettings,
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
    setSound,
    setSoundVolume,
    setHttpHost,
    setHttpPort,
    setBlockPluginStatus,
    setWindowVisible,
    addToHistory,
    clearHistory,
    syncFromHttpApi,
  };
});
