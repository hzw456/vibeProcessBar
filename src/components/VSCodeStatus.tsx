import { useEffect, useState, useCallback } from 'react';
import { useProgressStore } from '../stores/progressStore';

interface VSCodeStatusProps {
  onConnectionChange?: (connected: boolean) => void;
}

export function VSCodeStatus({ onConnectionChange }: VSCodeStatusProps) {
  const { settings } = useProgressStore();
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  const checkConnection = useCallback(async () => {
    if (!settings.vscodeEnabled) {
      setConnectionStatus('disconnected');
      return;
    }

    setConnectionStatus('connecting');

    try {
      const response = await fetch(`http://${settings.vscodeHost}:${settings.vscodePort}/health`, {
        method: 'GET',
        signal: AbortSignal.timeout(2000),
      });

      if (response.ok) {
        setConnectionStatus('connected');
        setLastUpdate(new Date());
        onConnectionChange?.(true);
      } else {
        setConnectionStatus('error');
        onConnectionChange?.(false);
      }
    } catch {
      setConnectionStatus('disconnected');
      onConnectionChange?.(false);
    }
  }, [settings.vscodeEnabled, settings.vscodeHost, settings.vscodePort, onConnectionChange]);

  useEffect(() => {
    checkConnection();

    const interval = setInterval(checkConnection, 10000);
    return () => {
      clearInterval(interval);
    };
  }, [checkConnection]);

  const statusLabels = {
    connecting: 'Connecting...',
    connected: 'Connected',
    disconnected: 'Disconnected',
    error: 'Connection Error',
  };

  const statusColors = {
    connecting: '#f59e0b',
    connected: '#10b981',
    disconnected: '#6b7280',
    error: '#ef4444',
  };

  return (
    <div className="vscode-status">
      <div className="vscode-status-header">
        <span className="vscode-icon">$(code)</span>
        <span>VSCode Integration</span>
      </div>
      <div className="vscode-status-content">
        <div 
          className="connection-indicator"
          style={{ backgroundColor: statusColors[connectionStatus] }}
        >
          <span className="status-dot"></span>
          <span className="status-text">{statusLabels[connectionStatus]}</span>
        </div>
        {lastUpdate && (
          <div className="last-update">
            Last update: {lastUpdate.toLocaleTimeString()}
          </div>
        )}
        <div className="connection-info">
          <span className="host">{settings.vscodeHost}</span>
          <span className="separator">:</span>
          <span className="port">{settings.vscodePort}</span>
        </div>
      </div>
      <button 
        className="reconnect-btn"
        onClick={checkConnection}
        disabled={connectionStatus === 'connecting'}
      >
        {connectionStatus === 'connecting' ? '$(sync~spin) Connecting...' : '$(refresh) Reconnect'}
      </button>
    </div>
  );
}
