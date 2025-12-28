import './StatusText.css';

interface StatusTextProps {
  taskName: string;
  status: 'idle' | 'running' | 'completed' | 'error';
  tokens?: number;
  ide?: string;
  onActivate?: () => void;
}

export function StatusText({ taskName, status, tokens = 0, ide, onActivate }: StatusTextProps) {
  const getStatusText = () => {
    switch (status) {
      case 'idle':
        return 'Ready';
      case 'running':
        return taskName || 'Running';
      case 'completed':
        return 'Complete';
      case 'error':
        return 'Error';
      default:
        return 'Ready';
    }
  };

  const getStatusClass = () => {
    return `status status-${status}`;
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
    if (ide && status === 'running' && onActivate) {
      onActivate();
    }
  };

  return (
    <div className="status-container">
      <span className={getStatusClass()} onClick={handleClick} style={{ cursor: ide && status === 'running' ? 'pointer' : 'default' }}>
        {getStatusText()}
      </span>
      {tokens > 0 && (
        <span className="token-count">{formatTokens(tokens)} tokens</span>
      )}
      {ide && status === 'running' && (
        <span className="ide-indicator" onClick={handleClick} title={`Click to activate ${ide}`}>
          {ide}
        </span>
      )}
    </div>
  );
}
