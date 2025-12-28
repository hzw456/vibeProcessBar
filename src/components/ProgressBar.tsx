import { useEffect, useRef } from 'react';
import './ProgressBar.css';

interface ProgressBarProps {
  progress: number;
  onChange?: (value: number) => void;
}

export function ProgressBar({ progress, onChange }: ProgressBarProps) {
  const barRef = useRef<HTMLDivElement>(null);
  const progressRef = useRef(progress);

  useEffect(() => {
    progressRef.current = progress;
  }, [progress]);

  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!onChange || !barRef.current) return;

    const rect = barRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const width = rect.width;

    const newProgress = Math.round((x / width) * 100);
    const clampedProgress = Math.max(0, Math.min(100, newProgress));
    onChange(clampedProgress);
  };

  return (
    <div 
      ref={barRef}
      className="progress-bar-container"
      onClick={handleClick}
    >
      <div className="progress-bar-track">
        <div 
          className="progress-bar-fill"
          style={{ width: `${progress}%` }}
        />
      </div>
      <span className="progress-text">{progress}%</span>
    </div>
  );
}
