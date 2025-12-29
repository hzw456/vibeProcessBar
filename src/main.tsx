import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { SettingsPanel } from './components/SettingsPanel';
import { useProgressStore } from './stores/progressStore';
import { useProgressNotifications } from './hooks/useProgressEvent';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import './index.css';

console.log('main.tsx loaded');

function Main() {
  const { settings } = useProgressStore();
  const [windowLabel] = useState<string>(() => getCurrentWindow().label);

  useProgressNotifications();

  useEffect(() => {
    console.log('Main component mounted, label:', windowLabel);

    // Listen for window custom event (from App.tsx context menu)
    const handleOpenSettings = () => {
      // Invoke the Rust command to open the settings window
      invoke('open_settings_window');
    };
    window.addEventListener('open-settings', handleOpenSettings);

    // Initial load of settings
    useProgressStore.getState().loadSettings();

    // Listen for settings changes from backend
    const unlistenSettings = listen<any>('settings-changed', (event) => {
      console.log('Settings changed event received:', event.payload);
      useProgressStore.getState().setSettings(event.payload);
    });

    return () => {
      window.removeEventListener('open-settings', handleOpenSettings);
      unlistenSettings.then(f => f());
    };
  }, []);

  useEffect(() => {
    if (settings.alwaysOnTop !== undefined) {
      // Only apply for main window or if specifically desired. 
      // For settings window, we might usually want it on top or normal.
      // But settings.alwaysOnTop setting usually refers to the BAR.
      // If we are in settings window, maybe we shouldn't force it? 
      // But user might want settings on top too. Let's keep it for now.
      invoke('set_window_always_on_top', { onTop: settings.alwaysOnTop });
    }
  }, [settings.alwaysOnTop]);

  useEffect(() => {
    invoke('set_window_opacity', { opacity: settings.opacity });
  }, [settings.opacity]);

  if (windowLabel === 'settings') {
    return (
      <React.StrictMode>
        <div className="settings-window-container">
          <SettingsPanel onClose={() => getCurrentWindow().close()} isStandalone={true} />
        </div>
      </React.StrictMode>
    );
  }

  return (
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
}

console.log('Creating React root');
const rootElement = document.getElementById('root');
console.log('Root element:', rootElement);

if (rootElement) {
  ReactDOM.createRoot(rootElement).render(<Main />);
  console.log('React rendered');
} else {
  console.error('Root element not found!');
}
