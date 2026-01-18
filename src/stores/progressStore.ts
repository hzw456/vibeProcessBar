import { defineStore } from 'pinia';
import { ref } from 'vue';
import { debug, error } from '../utils/logger';
import type { SupportedLanguage } from '../utils/i18n';
import { setLanguage as setI18nLanguage } from '../utils/i18n';

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

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
  tokens: number;
  status: 'armed' | 'running' | 'completed' | 'idle' | 'error' | 'cancelled';
  is_focused?: boolean;
  start_time: number;
  end_time?: number;
  adapter?: string;
  ide?: string;
  window_title?: string;
  project_path?: string;
  active_file?: string;
  estimated_duration?: number; // 预估总时长（毫秒）
  current_stage?: string; // 当前阶段描述
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
  blockPluginStatus: boolean;
  windowX: number | null;
  windowY: number | null;
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
  blockPluginStatus: true,
  windowX: null,
  windowY: null,
};

export const useProgressStore = defineStore('progress', () => {
  const tasks = ref<ProgressTask[]>([]);
  const history = ref<ProgressTask[]>([]);
  const settings = ref<AppSettings>({ ...defaultSettings });
  
  let unlistenTasks: (() => void) | null = null;
  let unlistenSettings: (() => void) | null = null;

  // 更新托盘菜单翻译
  async function updateTrayTranslations(language: SupportedLanguage) {
    try {
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

  // 初始化事件监听
  async function initEventListeners() {
    if (!isTauri) return;

    try {
      const { listen } = await import('@tauri-apps/api/event');

      // 监听任务更新事件
      unlistenTasks = await listen<ProgressTask[]>('tasks-updated', (event) => {
        debug('Received tasks-updated event', { count: event.payload.length });
        tasks.value = event.payload;
      });

      // 监听设置更新事件
      unlistenSettings = await listen<AppSettings>('settings-changed', (event) => {
        debug('Received settings-changed event');
        applySettings(event.payload);
      });

      debug('Event listeners initialized');
    } catch (err) {
      error('Failed to init event listeners', { error: String(err) });
    }
  }

  // 清理事件监听
  function cleanupEventListeners() {
    if (unlistenTasks) {
      unlistenTasks();
      unlistenTasks = null;
    }
    if (unlistenSettings) {
      unlistenSettings();
      unlistenSettings = null;
    }
  }

  // 应用设置
  function applySettings(newSettings: AppSettings) {
    const oldLanguage = settings.value.language;
    settings.value = newSettings;

    setI18nLanguage(newSettings.language);
    document.documentElement.setAttribute('data-theme', newSettings.theme);
    document.documentElement.style.setProperty('--app-font-size', `${newSettings.fontSize}px`);
    document.documentElement.style.setProperty('--app-opacity', newSettings.opacity.toString());

    if (oldLanguage !== newSettings.language) {
      updateTrayTranslations(newSettings.language);
    }
  }

  // 加载设置
  async function loadSettings() {
    try {
      const loadedSettings = await safeInvoke<AppSettings>('get_app_settings');
      if (loadedSettings) {
        applySettings(loadedSettings);
      }
    } catch (err) {
      error('Failed to load settings', { error: String(err) });
    }
  }

  // 从Rust层获取任务 (使用command而非HTTP API)
  async function fetchTasks() {
    try {
      const taskList = await safeInvoke<ProgressTask[]>('get_tasks');
      if (taskList) {
        tasks.value = taskList;
        debug('Fetched tasks from Rust', { count: taskList.length });
      }
    } catch (err) {
      error('Failed to fetch tasks', { error: String(err) });
    }
  }

  // 兼容旧的syncFromHttpApi方法
  async function syncFromHttpApi() {
    await fetchTasks();
  }

  function setSettings(newSettings: AppSettings) {
    applySettings(newSettings);
  }

  function addTask(name: string, adapter?: string, ide?: string, windowTitle?: string): string {
    const id = Date.now().toString();
    const newTask: ProgressTask = {
      id,
      name,
      tokens: 0,
      status: 'armed',
      is_focused: false,
      start_time: Date.now(),
      adapter,
      ide,
      window_title: windowTitle,
    };
    tasks.value.push(newTask);
    return id;
  }

  function removeTask(id: string) {
    tasks.value = tasks.value.filter(t => t.id !== id);
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
        task.end_time = Date.now();
      }
    }
  }

  function completeTask(id: string, totalTokens?: number) {
    const task = tasks.value.find(t => t.id === id);
    if (task) {
      task.status = 'completed';
      task.end_time = Date.now();
      if (totalTokens !== undefined) {
        task.tokens = totalTokens;
      }
    }
  }

  function resetTask(id: string) {
    const task = tasks.value.find(t => t.id === id);
    if (task) {
      task.tokens = 0;
      task.status = 'armed';
      task.is_focused = false;
      task.start_time = Date.now();
      task.end_time = undefined;
      task.estimated_duration = undefined;
      task.current_stage = undefined;
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
    setI18nLanguage(language);
    updateTrayTranslations(language);
  }

  function setTheme(theme: AppSettings['theme']) {
    updateSettingAndSync('theme', theme);
    document.documentElement.setAttribute('data-theme', theme);
  }

  function setFontSize(fontSize: number) {
    const clampedSize = Math.max(12, Math.min(18, fontSize));
    updateSettingAndSync('fontSize', clampedSize);
    document.documentElement.style.setProperty('--app-font-size', `${clampedSize}px`);
  }

  function setOpacity(opacity: number) {
    const clampedOpacity = Math.min(1, Math.max(0.5, opacity));
    updateSettingAndSync('opacity', clampedOpacity);
    document.documentElement.style.setProperty('--app-opacity', clampedOpacity.toString());
  }

  async function setAlwaysOnTop(value: boolean) {
    updateSettingAndSync('alwaysOnTop', value);
    try {
      await safeInvoke('set_always_on_top', { alwaysOnTop: value });
    } catch (err) {
      error('Failed to set always on top', { error: String(err) });
    }
  }

  async function setAutoStart(value: boolean) {
    updateSettingAndSync('autoStart', value);
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

  function setWindowPosition(x: number, y: number) {
    settings.value.windowX = x;
    settings.value.windowY = y;
    // 移动主窗口到新位置
    safeInvoke('set_main_window_position', { x, y });
    // 保存设置
    safeInvoke('update_app_settings', { newSettings: settings.value });
  }

  // 只更新位置数值，不保存到文件（用于拖动时实时显示）
  function updateWindowPositionDisplay(x: number, y: number) {
    settings.value.windowX = x;
    settings.value.windowY = y;
  }

  // 保存当前位置到配置文件
  async function saveWindowPositionToFile() {
    try {
      await safeInvoke('update_app_settings', { newSettings: settings.value });
    } catch (err) {
      error('Failed to save window position', { error: String(err) });
    }
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

  async function refreshSettings() {
    await loadSettings();
  }

  return {
    tasks,
    history,
    settings,
    loadSettings,
    refreshSettings,
    setSettings,
    addTask,
    removeTask,
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
    setWindowPosition,
    updateWindowPositionDisplay,
    saveWindowPositionToFile,
    setWindowVisible,
    addToHistory,
    clearHistory,
    syncFromHttpApi,
    fetchTasks,
    initEventListeners,
    cleanupEventListeners,
  };
});
