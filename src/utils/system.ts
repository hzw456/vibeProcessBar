import { invoke } from '@tauri-apps/api/core';

export async function enableAutoStart(): Promise<boolean> {
  try {
    await invoke('enable_auto_start');
    return true;
  } catch {
    return false;
  }
}

export async function disableAutoStart(): Promise<boolean> {
  try {
    await invoke('disable_auto_start');
    return true;
  } catch {
    return false;
  }
}

export async function isAutoStartEnabled(): Promise<boolean> {
  try {
    return await invoke('is_auto_start_enabled');
  } catch {
    return false;
  }
}

export async function getAppVersion(): Promise<string> {
  try {
    return await invoke('get_app_version');
  } catch {
    return '0.1.0';
  }
}

export async function minimizeToTray(): Promise<void> {
  await invoke('hide_window');
}

export async function quitApp(): Promise<void> {
  await invoke('close_window');
}

export async function openUrl(url: string): Promise<boolean> {
  try {
    await invoke('open_url', { url });
    return true;
  } catch {
    return false;
  }
}

export function getSystemInfo(): { platform: string; version: string; arch: string } {
  return {
    platform: navigator.userAgent.includes('Mac') ? 'macos' : 
              navigator.userAgent.includes('Windows') ? 'windows' : 'linux',
    version: navigator.appVersion,
    arch: navigator.userAgent.includes('x64') ? 'x64' : 'x86',
  };
}

export async function checkForUpdates(): Promise<{ current: string; latest: string; url: string } | null> {
  try {
    const response = await fetch('https://api.github.com/repos/yourusername/vibe-process-bar/releases/latest');
    if (!response.ok) return null;
    
    const data = await response.json();
    const currentVersion = await getAppVersion();
    
    return {
      current: currentVersion,
      latest: data.tag_name,
      url: data.html_url,
    };
  } catch {
    return null;
  }
}

export function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  
  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  } else {
    return `${seconds}s`;
  }
}

export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;
  
  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      timeout = null;
      func(...args);
    };
    
    if (timeout !== null) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(later, wait);
  };
}

export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean = false;
  
  return function executedFunction(...args: Parameters<T>) {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}
