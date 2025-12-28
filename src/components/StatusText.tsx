import './StatusText.css';

interface StatusTextProps {
  taskName: string;
  status: 'idle' | 'running' | 'completed' | 'error';
  tokens?: number;
  ide?: string;
  onActivate?: () => void;
}

export function StatusText({ taskName, status, tokens = 0, ide, onActivate }: StatusTextProps) {
  const getStatusIcon = () => {
    switch (status) {
      case 'idle':
        return '○';
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
        return 'Ready';
      case 'running':
        return taskName || 'Running...';
      case 'completed':
        return 'Complete';
      case 'error':
        return 'Error';
      default:
        return 'Ready';
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
