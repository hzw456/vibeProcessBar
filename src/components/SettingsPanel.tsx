import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useProgressStore } from '../stores/progressStore';
import { LanguageSelector } from './LanguageSelector';
import './SettingsPanel.css';

interface SettingsPanelProps {
  onClose: () => void;
  isStandalone?: boolean;
}

export function SettingsPanel({ onClose, isStandalone = false }: SettingsPanelProps) {
  const { t } = useTranslation();
  const {
    settings,
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
    clearHistory,
    history
  } = useProgressStore();
  const [activeTab, setActiveTab] = useState<'general' | 'appearance' | 'notifications' | 'tasks' | 'shortcuts'>('general');
  const [showImportExport, setShowImportExport] = useState(false);

  const handleExportConfig = () => {
    const config = {
      settings: {
        theme: settings.theme,
        opacity: settings.opacity,
        alwaysOnTop: settings.alwaysOnTop,
        notifications: settings.notifications,
        sound: settings.sound,
        soundVolume: settings.soundVolume,
        httpPort: settings.httpPort,
        customColors: settings.customColors,
        reminderThreshold: settings.reminderThreshold,
        doNotDisturb: settings.doNotDisturb,
        doNotDisturbStart: settings.doNotDisturbStart,
        doNotDisturbEnd: settings.doNotDisturbEnd,
      },
      history: history.slice(0, 20),
      exportedAt: new Date().toISOString(),
    };

    const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'vibe-progress-config.json';
    a.click();
    URL.revokeObjectURL(url);
    setShowImportExport(false);
  };

  const handleImportConfig = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const config = JSON.parse(event.target?.result as string);
        if (config.settings) {
          if (config.settings.theme) setTheme(config.settings.theme);
          if (config.settings.fontSize) setFontSize(config.settings.fontSize);
          if (config.settings.opacity) setOpacity(config.settings.opacity);
          if (config.settings.alwaysOnTop !== undefined) setAlwaysOnTop(config.settings.alwaysOnTop);
          if (config.settings.notifications !== undefined) setNotifications(config.settings.notifications);
          if (config.settings.sound !== undefined) setSound(config.settings.sound);
          if (config.settings.soundVolume) setSoundVolume(config.settings.soundVolume);
          if (config.settings.httpPort) setHttpPort(config.settings.httpPort);
          if (config.settings.customColors) setCustomColors(config.settings.customColors);
          if (config.settings.reminderThreshold) setReminderThreshold(config.settings.reminderThreshold);
          if (config.settings.doNotDisturb !== undefined) setDoNotDisturb(config.settings.doNotDisturb);
          if (config.settings.doNotDisturbStart) setDoNotDisturbStart(config.settings.doNotDisturbStart);
          if (config.settings.doNotDisturbEnd) setDoNotDisturbEnd(config.settings.doNotDisturbEnd);
        }
        setShowImportExport(false);
      } catch (error) {
        console.error('Failed to import config:', error);
        alert('Failed to import configuration. Please check the file format.');
      }
    };
    reader.readAsText(file);
  };

  if (isStandalone) {
    return (
      <div className="settings-panel standalone" onClick={e => e.stopPropagation()}>
        {/* No header for standalone window as it has system title bar */}
        {renderContent()}
      </div>
    );
  }

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel" onClick={e => e.stopPropagation()}>
        <div className="settings-header">
          <h2>{t('settings.title')}</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        {renderContent()}
      </div>
    </div>
  );

  function renderContent() {
    return (
      <>
        <div className="settings-tabs">
          <button
            className={`tab ${activeTab === 'general' ? 'active' : ''}`}
            onClick={() => setActiveTab('general')}
          >
            {t('settings.tabs.general')}
          </button>
          <button
            className={`tab ${activeTab === 'appearance' ? 'active' : ''}`}
            onClick={() => setActiveTab('appearance')}
          >
            {t('settings.tabs.appearance')}
          </button>
          <button
            className={`tab ${activeTab === 'notifications' ? 'active' : ''}`}
            onClick={() => setActiveTab('notifications')}
          >
            {t('settings.tabs.notifications')}
          </button>
          <button
            className={`tab ${activeTab === 'tasks' ? 'active' : ''}`}
            onClick={() => setActiveTab('tasks')}
          >
            {t('settings.tabs.tasks')}
          </button>
          <button
            className={`tab ${activeTab === 'shortcuts' ? 'active' : ''}`}
            onClick={() => setActiveTab('shortcuts')}
          >
            {t('settings.tabs.shortcuts')}
          </button>
        </div>

        <div className="settings-content">
          {activeTab === 'general' && (
            <div className="settings-section">
              <LanguageSelector />
              <div className="setting-item">
                <label>{t('settings.general.autoStart')}</label>
                <input
                  type="checkbox"
                  checked={settings.autoStart}
                  onChange={e => setAutoStart(e.target.checked)}
                />
              </div>
              <div className="setting-item">
                <label>{t('settings.general.alwaysOnTop')}</label>
                <input
                  type="checkbox"
                  checked={settings.alwaysOnTop}
                  onChange={e => setAlwaysOnTop(e.target.checked)}
                />
              </div>
              <div className="setting-item">
                <label>{t('settings.general.httpPort')}</label>
                <input
                  type="number"
                  className="http-input"
                  value={settings.httpPort}
                  onChange={e => setHttpPort(parseInt(e.target.value) || 31415)}
                  min="1024"
                  max="65535"
                />
              </div>
              <div className="setting-item">
                <label>{t('settings.general.configManagement')}</label>
                <button className="action-btn" onClick={() => setShowImportExport(true)}>
                  {t('settings.general.manage')}
                </button>
              </div>
            </div>
          )}

          {activeTab === 'appearance' && (
            <div className="settings-section">
              <div className="setting-item">
                <label>{t('settings.appearance.theme')}</label>
                <select
                  value={settings.theme}
                  onChange={e => setTheme(e.target.value as 'dark' | 'light' | 'purple' | 'ocean' | 'forest' | 'midnight')}
                >
                  <option value="dark">{t('settings.appearance.themeOptions.dark')}</option>
                  <option value="light">{t('settings.appearance.themeOptions.light')}</option>
                  <option value="purple">{t('settings.appearance.themeOptions.purple')}</option>
                  <option value="ocean">{t('settings.appearance.themeOptions.ocean')}</option>
                  <option value="forest">{t('settings.appearance.themeOptions.forest')}</option>
                  <option value="midnight">{t('settings.appearance.themeOptions.midnight')}</option>
                </select>
              </div>
              <div className="setting-item">
                <label>{t('settings.appearance.fontSize', { size: settings.fontSize })}</label>
                <input
                  type="range"
                  min="10"
                  max="24"
                  step="1"
                  value={settings.fontSize}
                  onChange={e => setFontSize(parseInt(e.target.value))}
                />
              </div>
              <div className="setting-item">
                <label>{t('settings.appearance.opacity', { percentage: Math.round(settings.opacity * 100) })}</label>
                <input
                  type="range"
                  min="0.3"
                  max="1"
                  step="0.05"
                  value={settings.opacity}
                  onChange={e => setOpacity(parseFloat(e.target.value))}
                />
              </div>
              <div className="setting-item">
                <label>{t('settings.appearance.customPrimaryColor')}</label>
                <div className="color-input-wrapper">
                  <input
                    type="color"
                    value={settings.customColors.primaryColor || '#6366f1'}
                    onChange={e => setCustomColors({ primaryColor: e.target.value })}
                    className="color-input"
                  />
                  <input
                    type="text"
                    className="color-text-input"
                    value={settings.customColors.primaryColor || ''}
                    onChange={e => setCustomColors({ primaryColor: e.target.value })}
                    placeholder="#6366f1"
                  />
                </div>
              </div>
              <div className="setting-item">
                <label>{t('settings.appearance.customBackground')}</label>
                <div className="color-input-wrapper">
                  <input
                    type="color"
                    value={settings.customColors.backgroundColor || '#0f172a'}
                    onChange={e => setCustomColors({ backgroundColor: e.target.value })}
                    className="color-input"
                  />
                  <input
                    type="text"
                    className="color-text-input"
                    value={settings.customColors.backgroundColor || ''}
                    onChange={e => setCustomColors({ backgroundColor: e.target.value })}
                    placeholder="#0f172a"
                  />
                </div>
              </div>
              <div className="setting-item">
                <label>{t('settings.appearance.resetColors')}</label>
                <button
                  className="action-btn small"
                  onClick={() => setCustomColors({ primaryColor: '', backgroundColor: '', textColor: '' })}
                >
                  {t('settings.appearance.reset')}
                </button>
              </div>
            </div>
          )}

          {activeTab === 'notifications' && (
            <div className="settings-section">
              <div className="setting-item">
                <label>{t('settings.notifications.desktopNotifications')}</label>
                <input
                  type="checkbox"
                  checked={settings.notifications}
                  onChange={e => setNotifications(e.target.checked)}
                />
              </div>
              <div className="setting-item">
                <label>{t('settings.notifications.soundAlerts')}</label>
                <input
                  type="checkbox"
                  checked={settings.sound}
                  onChange={e => setSound(e.target.checked)}
                />
              </div>
              {settings.sound && (
                <div className="setting-item indent">
                  <label>{t('settings.notifications.volume', { percentage: Math.round(settings.soundVolume * 100) })}</label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.1"
                    value={settings.soundVolume}
                    onChange={e => setSoundVolume(parseFloat(e.target.value))}
                  />
                </div>
              )}
              <div className="setting-item">
                <label>{t('settings.notifications.completionThreshold', { percentage: settings.reminderThreshold })}</label>
                <input
                  type="range"
                  min="50"
                  max="100"
                  step="5"
                  value={settings.reminderThreshold}
                  onChange={e => setReminderThreshold(parseInt(e.target.value))}
                />
              </div>
              <div className="setting-item">
                <label>{t('settings.notifications.doNotDisturb')}</label>
                <input
                  type="checkbox"
                  checked={settings.doNotDisturb}
                  onChange={e => setDoNotDisturb(e.target.checked)}
                />
              </div>
              {settings.doNotDisturb && (
                <>
                  <div className="setting-item indent">
                    <label>{t('settings.notifications.startTime')}</label>
                    <input
                      type="time"
                      value={settings.doNotDisturbStart}
                      onChange={e => setDoNotDisturbStart(e.target.value)}
                      className="time-input"
                    />
                  </div>
                  <div className="setting-item indent">
                    <label>{t('settings.notifications.endTime')}</label>
                    <input
                      type="time"
                      value={settings.doNotDisturbEnd}
                      onChange={e => setDoNotDisturbEnd(e.target.value)}
                      className="time-input"
                    />
                  </div>
                </>
              )}
            </div>
          )}

          {activeTab === 'tasks' && (
            <div className="settings-section">
              <div className="setting-item">
                <label>{t('settings.tasks.history')}</label>
                <span className="info-text">{t('settings.tasks.historyCount', { count: history.length })}</span>
              </div>
              <div className="setting-item">
                <label>{t('settings.tasks.clearHistory')}</label>
                <button
                  className="action-btn danger small"
                  onClick={clearHistory}
                >
                  {t('settings.tasks.clear')}
                </button>
              </div>
            </div>
          )}

          {activeTab === 'shortcuts' && (
            <div className="settings-section">
              <div className="shortcuts-info">
                <h4>{t('settings.shortcuts.title')}</h4>
                <p>{t('settings.shortcuts.description')}</p>
                <div className="shortcut-list">
                  <div className="shortcut-item">
                    <span className="shortcut-key">{t('settings.shortcuts.rightClick')}</span>
                    <span className="shortcut-desc">Open context menu</span>
                  </div>
                  <div className="shortcut-item">
                    <span className="shortcut-key">{t('settings.shortcuts.drag')}</span>
                    <span className="shortcut-desc">Move window</span>
                  </div>
                  <div className="shortcut-item">
                    <span className="shortcut-key">{t('settings.shortcuts.scroll')}</span>
                    <span className="shortcut-desc">Adjust progress</span>
                  </div>
                  <div className="shortcut-item">
                    <span className="shortcut-key">{t('settings.shortcuts.clickStatus')}</span>
                    <span className="shortcut-desc">Activate IDE window</span>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        {showImportExport && (
          <div className="import-export-modal">
            <div className="modal-content">
              <h3>{t('settings.importExport.title')}</h3>
              <div className="modal-actions">
                <button className="action-btn" onClick={handleExportConfig}>
                  {t('settings.importExport.exportConfig')}
                </button>
                <label className="action-btn">
                  {t('settings.importExport.importConfig')}
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleImportConfig}
                    style={{ display: 'none' }}
                  />
                </label>
                <button className="action-btn secondary" onClick={() => setShowImportExport(false)}>
                  {t('settings.importExport.cancel')}
                </button>
              </div>
            </div>
          </div>
        )}

        <div className="settings-footer">
          <div className="version-info">{t('settings.footer.version', { version: '0.1.0' })}</div>
          <button className="reset-btn" onClick={() => {
            setLanguage('en');
            setTheme('dark');
            setFontSize(14);
            setOpacity(0.85);
            setAlwaysOnTop(true);
            setAutoStart(false);
            setNotifications(true);
            setSound(true);
            setSoundVolume(0.7);
            setHttpPort(31415);
            setCustomColors({ primaryColor: '', backgroundColor: '', textColor: '' });
            setReminderThreshold(100);
            setDoNotDisturb(false);
            setDoNotDisturbStart('22:00');
            setDoNotDisturbEnd('08:00');
          }}>
            {t('settings.footer.resetDefaults')}
          </button>
        </div>
      </>
    );
  }
}
