import { useEffect, useRef, useState } from 'react';
import { ProgressBar } from './components/ProgressBar';
import { StatusText } from './components/StatusText';
import { appWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/tauri';
import './App.css';

interface Task {
  id: string;
  name: string;
  progress: number;
  status: 'idle' | 'running' | 'completed' | 'error';
  startTime: number;
}

function App() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [currentTaskId, setCurrentTaskId] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  const currentTask = tasks.find(t => t.id === currentTaskId) || tasks[0] || null;

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (isDragging && containerRef.current) {
        const newX = e.clientX - dragOffset.x;
        const newY = e.clientY - dragOffset.y;
        invoke('set_window_position', { x: newX, y: newY });
      }
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    if (isDragging) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
    }

    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, dragOffset]);

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button === 0) {
      setIsDragging(true);
      const rect = containerRef.current?.getBoundingClientRect();
      if (rect) {
        setDragOffset({
          x: e.clientX - rect.left,
          y: e.clientY - rect.top
        });
      }
    }
  };

  const handleProgressChange = (value: number) => {
    if (currentTaskId && currentTask) {
      setTasks(prev => prev.map(t => 
        t.id === currentTaskId ? { ...t, progress: value, status: value >= 100 ? 'completed' : 'running' } : t
      ));
    }
  };

  const startNewTask = (name: string = 'New Task') => {
    const id = Date.now().toString();
    const newTask: Task = {
      id,
      name,
      progress: 0,
      status: 'running',
      startTime: Date.now()
    };
    setTasks(prev => [...prev, newTask]);
    setCurrentTaskId(id);
  };

  const resetTask = () => {
    if (currentTaskId) {
      setTasks(prev => prev.map(t => 
        t.id === currentTaskId ? { ...t, progress: 0, status: 'idle', startTime: Date.now() } : t
      ));
    }
  };

  return (
    <div 
      ref={containerRef}
      className="app-container"
      onMouseDown={handleMouseDown}
      onDoubleClick={resetTask}
    >
      <StatusText 
        taskName={currentTask?.name || 'Ready'}
        status={currentTask?.status || 'idle'}
      />
      <ProgressBar 
        progress={currentTask?.progress || 0}
        onChange={handleProgressChange}
      />
    </div>
  );
}

export default App;
