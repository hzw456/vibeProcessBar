import { useState } from 'react';
import { useProgressStore } from '../stores/progressStore';
import './SettingsPanel.css';

interface SettingsPanelProps {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const { settings, setTheme, setOpacity, setAlwaysOnTop, setAutoStart, setNotifications, setSound } = useProgressStore();
  const [activeTab, setActiveTab] = useState<'general' | 'appearance' | 'notifications'>('general');

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel" onClick={e => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="settings-tabs">
          <button 
            className={`tab ${activeTab === 'general' ? 'active' : ''}`}
            onClick={() => setActiveTab('general')}
          >
            General
          </button>
          <button 
            className={`tab ${activeTab === 'appearance' ? 'active' : ''}`}
            onClick={() => setActiveTab('appearance')}
          >
            Appearance
          </button>
          <button 
            className={`tab ${activeTab === 'notifications' ? 'active' : ''}`}
            onClick={() => setActiveTab('notifications')}
          >
            Notifications
          </button>
        </div>

        <div className="settings-content">
          {activeTab === 'general' && (
            <div className="settings-section">
              <div className="setting-item">
                <label>Auto-start on login</label>
                <input
                  type="checkbox"
                  checked={settings.autoStart}
                  onChange={e => setAutoStart(e.target.checked)}
                />
              </div>
              <div className="setting-item">
                <label>Always on top</label>
                <input
                  type="checkbox"
                  checked={settings.alwaysOnTop}
                  onChange={e => setAlwaysOnTop(e.target.checked)}
                />
              </div>
            </div>
          )}

          {activeTab === 'appearance' && (
            <div className="settings-section">
              <div className="setting-item">
                <label>Theme</label>
                <select 
                  value={settings.theme}
                  onChange={e => setTheme(e.target.value as 'dark' | 'light')}
                >
                  <option value="dark">Dark</option>
                  <option value="light">Light</option>
                </select>
              </div>
              <div className="setting-item">
                <label>Opacity: {Math.round(settings.opacity * 100)}%</label>
                <input
                  type="range"
                  min="0.1"
                  max="1"
                  step="0.05"
                  value={settings.opacity}
                  onChange={e => setOpacity(parseFloat(e.target.value))}
                />
              </div>
            </div>
          )}

          {activeTab === 'notifications' && (
            <div className="settings-section">
              <div className="setting-item">
                <label>Desktop notifications</label>
                <input
                  type="checkbox"
                  checked={settings.notifications}
                  onChange={e => setNotifications(e.target.checked)}
                />
              </div>
              <div className="setting-item">
                <label>Sound alerts</label>
                <input
                  type="checkbox"
                  checked={settings.sound}
                  onChange={e => setSound(e.target.checked)}
                />
              </div>
            </div>
          )}
        </div>

        <div className="settings-footer">
          <button className="reset-btn" onClick={() => {
            setTheme('dark');
            setOpacity(0.85);
            setAlwaysOnTop(true);
            setAutoStart(false);
            setNotifications(true);
            setSound(true);
          }}>
            Reset to defaults
          </button>
        </div>
      </div>
    </div>
  );
}
