import { useEffect, useRef } from 'react';
import './ProgressBar.css';

interface ProgressBarProps {
  progress: number;
  onChange?: (value: number) => void;
}

export function ProgressBar({ progress, onChange }: ProgressBarProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>();
  const currentProgress = useRef(progress);

  useEffect(() => {
    currentProgress.current = progress;
  }, [progress]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;
    const radius = height / 2 - 2;
    const centerX = width / 2;
    const centerY = height / 2;

    const draw = () => {
      ctx.clearRect(0, 0, width, height);

      const startAngle = Math.PI * 0.75;
      const endAngle = Math.PI * 2.25;
      const progressAngle = startAngle + (endAngle - startAngle) * (currentProgress.current / 100);

      ctx.beginPath();
      ctx.arc(centerX, centerY, radius, startAngle, endAngle);
      ctx.strokeStyle = 'rgba(148, 163, 184, 0.3)';
      ctx.lineWidth = 4;
      ctx.lineCap = 'round';
      ctx.stroke();

      const gradient = ctx.createLinearGradient(0, 0, width, 0);
      gradient.addColorStop(0, '#6366f1');
      gradient.addColorStop(0.5, '#8b5cf6');
      gradient.addColorStop(1, '#a855f7');

      ctx.beginPath();
      ctx.arc(centerX, centerY, radius, startAngle, progressAngle);
      ctx.strokeStyle = gradient;
      ctx.lineWidth = 4;
      ctx.lineCap = 'round';
      ctx.stroke();

      animationRef.current = requestAnimationFrame(draw);
    };

    draw();

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  const handleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!onChange) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const width = rect.width;

    const newProgress = Math.round((x / width) * 100);
    const clampedProgress = Math.max(0, Math.min(100, newProgress));
    onChange(clampedProgress);
  };

  return (
    <canvas
      ref={canvasRef}
      width={176}
      height={24}
      className="progress-canvas"
      onClick={handleClick}
    />
  );
}
