import { invoke } from '@tauri-apps/api/core';

export async function toggleAlwaysOnTop(): Promise<boolean> {
  return await invoke('toggle_window_always_on_top');
}

export async function hideWindow(): Promise<void> {
  await invoke('hide_window');
}

export async function showWindow(): Promise<void> {
  await invoke('show_window');
}

export async function getWindowPosition(): Promise<{ x: number; y: number }> {
  return await invoke('get_window_position');
}

export async function setWindowPosition(x: number, y: number): Promise<void> {
  await invoke('set_window_position', { x, y });
}

export async function resizeWindow(width: number, height: number): Promise<void> {
  await invoke('resize_window', { width, height });
}

export async function getAllWindows(): Promise<Array<{ label: string; position: { x: number; y: number }; width: number; height: number }>> {
  return await invoke('get_all_windows');
}

export interface MonitorInfo {
  name: string;
  width: number;
  height: number;
  position: { x: number; y: number };
  scaleFactor: number;
}

export async function getMonitors(): Promise<MonitorInfo[]> {
  if ('getAll' in window) {
    try {
      const monitors = await (window as any).getAll();
      return monitors.map((m: any) => ({
        name: m.label || `Monitor ${m.id}`,
        width: m.width,
        height: m.height,
        position: { x: m.x || 0, y: m.y || 0 },
        scaleFactor: m.scaleFactor || 1,
      }));
    } catch {
      return getFallbackMonitors();
    }
  }
  return getFallbackMonitors();
}

function getFallbackMonitors(): MonitorInfo[] {
  return [
    {
      name: 'Primary Monitor',
      width: window.screen.width,
      height: window.screen.height,
      position: { x: 0, y: 0 },
      scaleFactor: window.devicePixelRatio,
    },
  ];
}

export async function moveToMonitor(monitorIndex: number = 0): Promise<void> {
  const monitors = await getMonitors();
  if (monitorIndex >= monitors.length) {
    monitorIndex = 0;
  }

  const targetMonitor = monitors[monitorIndex];
  const windowSize = { width: 200, height: 60 };

  const newX = targetMonitor.position.x + (targetMonitor.width - windowSize.width) / 2;
  const newY = targetMonitor.position.y + 50;

  await setWindowPosition(newX, newY);
}

export async function moveToCorner(corner: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right'): Promise<void> {
  const monitors = await getMonitors();
  const primaryMonitor = monitors[0];
  const windowSize = { width: 200, height: 60 };
  const padding = 20;

  let newX = 0;
  let newY = 0;

  switch (corner) {
    case 'top-left':
      newX = primaryMonitor.position.x + padding;
      newY = primaryMonitor.position.y + padding;
      break;
    case 'top-right':
      newX = primaryMonitor.position.x + primaryMonitor.width - windowSize.width - padding;
      newY = primaryMonitor.position.y + padding;
      break;
    case 'bottom-left':
      newX = primaryMonitor.position.x + padding;
      newY = primaryMonitor.position.y + primaryMonitor.height - windowSize.height - padding;
      break;
    case 'bottom-right':
      newX = primaryMonitor.position.x + primaryMonitor.width - windowSize.width - padding;
      newY = primaryMonitor.position.y + primaryMonitor.height - windowSize.height - padding;
      break;
  }

  await setWindowPosition(newX, newY);
}

export async function centerWindow(): Promise<void> {
  const monitors = await getMonitors();
  const primaryMonitor = monitors[0];
  const windowSize = { width: 200, height: 60 };

  const newX = primaryMonitor.position.x + (primaryMonitor.width - windowSize.width) / 2;
  const newY = primaryMonitor.position.y + (primaryMonitor.height - windowSize.height) / 2;

  await setWindowPosition(newX, newY);
}

export function registerGlobalShortcut(key: string, callback: () => void): () => void {
  const handler = (event: KeyboardEvent) => {
    const modifiers = {
      ctrl: event.ctrlKey || event.metaKey,
      alt: event.altKey,
      shift: event.shiftKey,
      win: event.metaKey,
    };

    const keyLower = key.toLowerCase();
    const pressedKey = event.key.toLowerCase();

    let modifierMatch = false;
    if (keyLower.startsWith('ctrl+') && modifiers.ctrl && pressedKey === keyLower.replace('ctrl+', '')) {
      modifierMatch = true;
    } else if (keyLower.startsWith('alt+') && modifiers.alt && pressedKey === keyLower.replace('alt+', '')) {
      modifierMatch = true;
    } else if (keyLower.startsWith('shift+') && modifiers.shift && pressedKey === keyLower.replace('shift+', '')) {
      modifierMatch = true;
    } else if (keyLower.startsWith('cmd+') && modifiers.win && pressedKey === keyLower.replace('cmd+', '')) {
      modifierMatch = true;
    }

    if (modifierMatch) {
      event.preventDefault();
      callback();
    }
  };

  document.addEventListener('keydown', handler);
  return () => document.removeEventListener('keydown', handler);
}

export const SHORTCUTS = {
  TOGGLE_VISIBILITY: 'ctrl+h',
  TOGGLE_ALWAYS_ON_TOP: 'ctrl+t',
  SHOW_MENU: 'ctrl+m',
  RESET_PROGRESS: 'ctrl+r',
  NEXT_TASK: 'ctrl+n',
  PREV_TASK: 'ctrl+p',
} as const;
