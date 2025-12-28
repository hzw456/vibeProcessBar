import { useEffect, useRef, useState } from 'react';
import { ProgressBar } from './components/ProgressBar';
import { StatusText } from './components/StatusText';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { useProgressStore } from './stores/progressStore';
import { useProgressNotifications } from './hooks/useProgressEvent';
import { SHORTCUTS, registerGlobalShortcut, moveToCorner } from './utils/windowManager';
import './App.css';

console.log('App.tsx loaded');

function App() {
  const { tasks, currentTaskId, settings, setCurrentTask, updateProgress, resetTask, removeTask, syncFromHttpApi } = useProgressStore();
  const containerRef = useRef<HTMLDivElement>(null);
  const [showMenu, setShowMenu] = useState(false);
  const [isResizing, setIsResizing] = useState(false);

  useProgressNotifications();

  const currentTask = tasks.find(t => t.id === currentTaskId) || tasks[0] || null;

  useEffect(() => {
    console.log('App mounted, tasks:', tasks.length, 'currentTask:', currentTask?.name);
  }, [tasks, currentTask]);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', settings.theme);
  }, [settings.theme]);

  useEffect(() => {
    if (settings.customColors.primaryColor) {
      document.documentElement.style.setProperty('--primary-color', settings.customColors.primaryColor);
    }
    if (settings.customColors.backgroundColor) {
      document.documentElement.style.setProperty('--bg-color', settings.customColors.backgroundColor);
    }
    if (settings.customColors.textColor) {
      document.documentElement.style.setProperty('--text-color', settings.customColors.textColor);
    }
  }, [settings.customColors]);

  useEffect(() => {
    const syncInterval = setInterval(() => {
      syncFromHttpApi();
    }, 1000);

    return () => clearInterval(syncInterval);
  }, [syncFromHttpApi]);

  useEffect(() => {
    const unregisterShortcuts = [
      registerGlobalShortcut(SHORTCUTS.TOGGLE_ALWAYS_ON_TOP, async () => {
        await invoke('toggle_window_always_on_top');
      }),
      registerGlobalShortcut(SHORTCUTS.RESET_PROGRESS, () => {
        if (currentTask) {
          resetTask(currentTask.id);
        }
      }),
      registerGlobalShortcut(SHORTCUTS.NEXT_TASK, () => {
        if (tasks.length > 1) {
          const currentIndex = tasks.findIndex(t => t.id === currentTaskId);
          const nextIndex = (currentIndex + 1) % tasks.length;
          setCurrentTask(tasks[nextIndex].id);
        }
      }),
      registerGlobalShortcut(SHORTCUTS.PREV_TASK, () => {
        if (tasks.length > 1) {
          const currentIndex = tasks.findIndex(t => t.id === currentTaskId);
          const prevIndex = (currentIndex - 1 + tasks.length) % tasks.length;
          setCurrentTask(tasks[prevIndex].id);
        }
      }),
      registerGlobalShortcut(SHORTCUTS.SHOW_MENU, () => {
        setShowMenu(prev => !prev);
      }),
    ];

    return () => {
      unregisterShortcuts.forEach(unregister => unregister());
    };
  }, [tasks, currentTaskId, currentTask, setCurrentTask, resetTask]);

  const handleMouseDown = async (e: React.MouseEvent) => {
    if (e.button === 0) {
      if (isResizing) return;
      await getCurrentWindow().startDragging();
    }
  };

  const handleResizeStart = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsResizing(true);
    
    const startX = e.clientX;
    const startY = e.clientY;
    const container = containerRef.current;
    if (!container) return;
    
    const startWidth = container.offsetWidth;
    const startHeight = container.offsetHeight;
    
    const handleMouseMove = (moveEvent: MouseEvent) => {
      const newWidth = Math.max(150, Math.min(400, startWidth + (moveEvent.clientX - startX)));
      const newHeight = Math.max(50, Math.min(150, startHeight + (moveEvent.clientY - startY)));
      container.style.width = `${newWidth}px`;
      container.style.height = `${newHeight}px`;
    };
    
    const handleMouseUp = () => {
      setIsResizing(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  const handleRightClick = (e: React.MouseEvent) => {
    e.preventDefault();
    setShowMenu(!showMenu);
  };

  const handleProgressChange = (value: number) => {
    if (currentTask) {
      updateProgress(currentTask.id, value);
    }
  };

  const handleTaskSelect = (taskId: string) => {
    setCurrentTask(taskId);
    setShowMenu(false);
  };

  const handleDeleteTask = (taskId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    removeTask(taskId);
    setShowMenu(false);
  };

  const handleResetTask = () => {
    if (currentTask) {
      resetTask(currentTask.id);
    }
    setShowMenu(false);
  };

  const handleOpenSettings = () => {
    setShowMenu(false);
    window.dispatchEvent(new CustomEvent('open-settings'));
  };

  const handleMoveToCorner = (corner: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right') => {
    moveToCorner(corner);
    setShowMenu(false);
  };

  const handleWheelProgress = (e: React.WheelEvent) => {
    if (currentTask) {
      const delta = e.deltaY > 0 ? -5 : 5;
      const newProgress = Math.max(0, Math.min(100, currentTask.progress + delta));
      updateProgress(currentTask.id, newProgress);
    }
  };

  const handleActivateWindow = async () => {
    if (currentTask?.ide) {
      try {
        await invoke('activate_ide_window', {
          ide: currentTask.ide,
          windowTitle: currentTask.windowTitle || null
        });
      } catch (error) {
        console.error('Failed to activate IDE window:', error);
      }
    }
  };

  return (
    <div 
      ref={containerRef}
      className="app-container"
      onMouseDown={handleMouseDown}
      onContextMenu={handleRightClick}
      onWheel={handleWheelProgress}
      style={{ opacity: settings.opacity }}
    >
      <StatusText 
        taskName={currentTask?.name || 'Ready'}
        status={currentTask?.status || 'idle'}
        tokens={currentTask?.tokens || 0}
        ide={currentTask?.ide}
        onActivate={handleActivateWindow}
      />
      <ProgressBar 
        progress={currentTask?.progress || 0}
        onChange={handleProgressChange}
      />
      <div className="resize-handle" onMouseDown={handleResizeStart}></div>
      {showMenu && (
        <div className="context-menu" onClick={(e) => e.stopPropagation()}>
          <div className="menu-header">Tasks</div>
          {tasks.length === 0 ? (
            <div className="menu-item disabled">No tasks</div>
          ) : (
            tasks.map(task => (
              <div 
                key={task.id} 
                className={`menu-item ${task.id === currentTaskId ? 'active' : ''}`}
                onClick={() => handleTaskSelect(task.id)}
              >
                <span className={`status-dot status-${task.status}`}></span>
                <span className="task-name">{task.name}</span>
                <span className="task-progress">{task.progress}%</span>
                <button 
                  className="delete-btn"
                  onClick={(e) => handleDeleteTask(task.id, e)}
                >
                  ×
                </button>
              </div>
            ))
          )}
          <div className="menu-divider"></div>
          <div className="menu-item" onClick={handleResetTask}>
            Reset Current Task
          </div>
          <div className="menu-item submenu">
            <span>Position</span>
            <div className="submenu-content">
              <div className="menu-item" onClick={() => handleMoveToCorner('top-left')}>Top Left</div>
              <div className="menu-item" onClick={() => handleMoveToCorner('top-right')}>Top Right</div>
              <div className="menu-item" onClick={() => handleMoveToCorner('bottom-left')}>Bottom Left</div>
              <div className="menu-item" onClick={() => handleMoveToCorner('bottom-right')}>Bottom Right</div>
            </div>
          </div>
          <div className="menu-item" onClick={handleOpenSettings}>
            Settings
          </div>
          <div className="menu-item" onClick={() => setShowMenu(false)}>
            Close Menu
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
