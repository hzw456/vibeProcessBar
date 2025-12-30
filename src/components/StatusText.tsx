import { useTranslation } from 'react-i18next';
import './StatusText.css';

interface StatusTextProps {
  taskName: string;
  status: 'idle' | 'running' | 'completed' | 'error' | 'armed' | 'active';
  tokens?: number;
  ide?: string;
  onActivate?: () => void;
  elapsedTime?: string;
}

export function StatusText({ taskName, status, tokens = 0, ide, onActivate, elapsedTime }: StatusTextProps) {
  const { t } = useTranslation();

  const getStatusIcon = () => {
    switch (status) {
      case 'idle':
        return '○';
      case 'armed':
        return '◎';
      case 'active':
        return '◈';
      case 'running':
        return '◉';
      case 'completed':
        return '✓';
      case 'error':
        return '✕';
      default:
        return '○';
    }
  };
  
  const getStatusText = () => {
    switch (status) {
      case 'idle':
        return t('status.idle');
      case 'armed':
        return taskName || t('status.armed');
      case 'active':
        return taskName || t('status.active');
      case 'running':
        return taskName || t('status.running');
      case 'completed':
        return elapsedTime ? t('status.completedWithTime', { taskName, elapsedTime }) : taskName || t('status.completed');
      case 'error':
        return t('status.error');
      default:
        return t('status.idle');
    }
  };

  const formatTokens = (num: number) => {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1) + 'M';
    }
    if (num >= 1000) {
      return (num / 1000).toFixed(1) + 'K';
    }
    return num.toString();
  };

  const handleClick = () => {
    if (ide && onActivate) {
      onActivate();
    }
  };

  return (
    <div className="status-container" onClick={handleClick} style={{ cursor: ide ? 'pointer' : 'default' }}>
      <span className={`status-icon status-${status}`}>{getStatusIcon()}</span>
      <span className={`status-text status-${status}`}>
        {getStatusText()}
      </span>
      {tokens > 0 && (
        <span className="token-count">{formatTokens(tokens)}</span>
      )}
      {ide && (
        <span className="ide-badge" title={`Click to activate ${ide}`}>
          {ide}
        </span>
      )}
    </div>
  );
}
