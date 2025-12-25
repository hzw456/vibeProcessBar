import './StatusText.css';

interface StatusTextProps {
  taskName: string;
  status: 'idle' | 'running' | 'completed' | 'error';
}

export function StatusText({ taskName, status }: StatusTextProps) {
  const getStatusText = () => {
    switch (status) {
      case 'idle':
        return 'Ready';
      case 'running':
        return taskName;
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

  return (
    <div className="status-container">
      <span className={getStatusClass()}>{getStatusText()}</span>
    </div>
  );
}
