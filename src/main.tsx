import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { SettingsPanel } from './components/SettingsPanel';
import { useProgressStore } from './stores/progressStore';
import { useProgressNotifications } from './hooks/useProgressEvent';
import { invoke } from '@tauri-apps/api/core';
import './index.css';

function Main() {
  const [showSettings, setShowSettings] = useState(false);
  const { settings } = useProgressStore();

  useProgressNotifications();

  useEffect(() => {
    const handleOpenSettings = () => setShowSettings(true);
    window.addEventListener('open-settings', handleOpenSettings);
    return () => window.removeEventListener('open-settings', handleOpenSettings);
  }, []);

  useEffect(() => {
    if (settings.alwaysOnTop !== undefined) {
      invoke('set_window_always_on_top', { onTop: settings.alwaysOnTop });
    }
  }, [settings.alwaysOnTop]);

  useEffect(() => {
    invoke('set_window_opacity', { opacity: settings.opacity });
  }, [settings.opacity]);

  return (
    <React.StrictMode>
      <App />
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
    </React.StrictMode>
  );
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <Main />
);
