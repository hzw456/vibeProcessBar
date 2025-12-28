import { useEffect, useRef, useState } from 'react';
// @ts-ignore - ProgressBar may be used in future
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
  const [completedTask, setCompletedTask] = useState<string | null>(null);
  const [, forceUpdate] = useState(0); // For time updates
  const [clickedCompletedTasks, setClickedCompletedTasks] = useState<Set<string>>(new Set()); // Track clicked completed tasks
  const [isCollapsed, setIsCollapsed] = useState(false); // Start expanded
  const [isCollapseTransition, setIsCollapseTransition] = useState(false); // Skip auto-resize during collapse
  const prevTasksRef = useRef<typeof tasks>([]);

  useProgressNotifications();

  // Keep tasks in original order, no sorting
  const displayTasks = tasks.filter(t => t.status === 'completed' || t.status === 'running' || t.status === 'idle' || t.status === 'armed' || t.status === 'active');
  const currentTask = tasks.find(t => t.id === currentTaskId) || tasks.find(t => t.status === 'running') || tasks[0] || null;

  // Detect newly completed tasks
  useEffect(() => {
    const prevTasks = prevTasksRef.current;
    tasks.forEach(task => {
      const prevTask = prevTasks.find(t => t.id === task.id);
      if (prevTask && prevTask.status !== 'completed' && task.status === 'completed') {
        // Task just completed - show notification
        setCompletedTask(task.id);
        setTimeout(() => setCompletedTask(null), 3000);
      }
    });
    prevTasksRef.current = [...tasks];
  }, [tasks]);

  // Dynamically resize window based on task count and collapse state
  useEffect(() => {
    // Skip auto-resize during collapse transition (handled manually)
    if (isCollapseTransition) return;
    
    const resizeWindow = async () => {
      const taskCount = displayTasks.length;
      const taskHeight = 36;
      const padding = 20;
      
      let newHeight = padding;
      if (taskCount === 0) {
        newHeight = 60;
      } else if (taskCount === 1) {
        newHeight = 70;
      } else {
        newHeight = padding + taskCount * taskHeight;
      }
      
      const width = isCollapsed ? 120 : 280;
      
      try {
        await invoke('resize_window', { width, height: Math.max(50, newHeight) });
      } catch (e) {
        console.error('Failed to resize window:', e);
      }
    };
    resizeWindow();
  }, [displayTasks.length, isCollapsed, isCollapseTransition]);

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
      forceUpdate(n => n + 1); // Update time display
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
      // Don't start dragging if clicking on interactive elements
      const target = e.target as HTMLElement;
      if (target.closest('.task-row') || target.closest('.collapsed-task-item') || target.closest('.collapse-btn') || target.closest('button')) {
        return;
      }
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

  // @ts-ignore - handleProgressChange may be used in future
  const handleProgressChange = (_value: number) => {
    if (currentTask) {
      updateProgress(currentTask.id, _value);
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

  const handleTaskClick = async (task: typeof currentTask) => {
    if (!task) return;
    console.log('Task clicked:', task.id, 'ide:', task.ide, 'windowTitle:', task.windowTitle);
    setCurrentTask(task.id);
    
    // Mark completed task as clicked (removes highlight)
    if (task.status === 'completed') {
      setClickedCompletedTasks(prev => new Set([...prev, task.id]));
    }
    
    if (task.ide) {
      try {
        console.log('Activating IDE window:', task.ide, task.windowTitle);
        await invoke('activate_ide_window', {
          ide: task.ide,
          windowTitle: task.windowTitle || null
        });
        console.log('IDE window activated successfully');
      } catch (error) {
        console.error('Failed to activate IDE window:', error);
      }
    } else {
      console.log('No IDE specified for task');
    }
  };

  // Handle collapse/expand with right-align position adjustment
  const handleCollapse = async () => {
    const expandedWidth = 280;
    const collapsedWidth = 120;
    const widthDiff = expandedWidth - collapsedWidth;
    
    // Prevent auto-resize during transition
    setIsCollapseTransition(true);
    
    try {
      const win = getCurrentWindow();
      const position = await win.innerPosition();
      const scaleFactor = await win.scaleFactor();
      
      // Convert physical pixels to logical pixels
      const logicalX = position.x / scaleFactor;
      const logicalY = position.y / scaleFactor;
      
      // Calculate height
      const taskCount = displayTasks.length;
      const taskHeight = 36;
      const padding = 20;
      let newHeight = padding;
      if (taskCount === 0) {
        newHeight = 60;
      } else if (taskCount === 1) {
        newHeight = 70;
      } else {
        newHeight = padding + taskCount * taskHeight;
      }
      newHeight = Math.max(50, newHeight);
      
      if (!isCollapsed) {
        // Collapsing: resize to smaller width, then move right
        await invoke('resize_window', { width: collapsedWidth, height: newHeight });
        const newX = logicalX + widthDiff;
        await invoke('set_window_position', { x: newX, y: logicalY });
      } else {
        // Expanding: move left first, then resize to larger width
        const newX = logicalX - widthDiff;
        await invoke('set_window_position', { x: newX, y: logicalY });
        await invoke('resize_window', { width: expandedWidth, height: newHeight });
      }
      
      setIsCollapsed(!isCollapsed);
    } catch (e) {
      console.error('Failed to adjust window position:', e);
      // Still toggle state even if position adjustment fails
      setIsCollapsed(!isCollapsed);
    } finally {
      // Re-enable auto-resize after a short delay
      setTimeout(() => setIsCollapseTransition(false), 100);
    }
  };

  return (
    <div 
      ref={containerRef}
      className={`app-container ${completedTask ? 'has-completed' : ''} ${displayTasks.length > 1 ? 'multi-task' : ''} ${isCollapsed ? 'collapsed' : ''}`}
      onMouseDown={handleMouseDown}
      onContextMenu={handleRightClick}
      onWheel={handleWheelProgress}
      style={{ opacity: settings.opacity }}
    >
      {/* Collapsed view - narrow bar on right side */}
      {isCollapsed ? (
        <div className="collapsed-view">
          {/* Expand button on left - same position as collapse button */}
          <button className="collapse-btn expanded" onClick={handleCollapse}>‹</button>
          <div className="collapsed-content">
            <div className="collapsed-tasks">
              {displayTasks.length === 0 ? (
                <div className="collapsed-empty">◆</div>
              ) : (
                displayTasks.map(task => (
                  <div key={task.id} className={`collapsed-task-item ${task.status}`} onClick={() => handleTaskClick(task)}>
                    <span className="collapsed-status">
                      {task.status === 'running' ? '◉' : task.status === 'completed' ? '✓' : task.status === 'armed' ? '◎' : task.status === 'active' ? '◈' : '○'}
                    </span>
                    <span className="collapsed-task-name">{task.name?.substring(0, 8) || task.ide}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      ) : (
        <>
          {/* Collapse button on left */}
          <button className="collapse-btn" onClick={handleCollapse}>›</button>
          
          {/* Show completed notification */}
          {completedTask && (
            <div className="completed-banner">
              ✓ {tasks.find(t => t.id === completedTask)?.name || 'Task'} Completed!
            </div>
          )}
      
      {/* Multi-task view */}
      {displayTasks.length > 1 ? (
        <div className="multi-task-list">
          {displayTasks.map((task) => {
            // Calculate elapsed time (only for running/completed, not armed/active)
            let timeStr = '';
            if (task.status === 'armed') {
              timeStr = '⏳'; // Armed: waiting for AI activity
            } else if (task.status === 'active') {
              timeStr = '👁'; // Active: window has focus
            } else if (task.status === 'completed') {
              const elapsed = (task.endTime || Date.now()) - task.startTime;
              const minutes = Math.floor(elapsed / 60000);
              const seconds = Math.floor((elapsed % 60000) / 1000);
              timeStr = minutes > 0 ? `${minutes}m ${seconds}s` : `${seconds}s`;
            } else if (task.status === 'running' && task.startTime > 0) {
              const elapsed = Date.now() - task.startTime;
              const minutes = Math.floor(elapsed / 60000);
              const seconds = Math.floor((elapsed % 60000) / 1000);
              timeStr = minutes > 0 ? `${minutes}m ${seconds}s` : `${seconds}s`;
            }
            
            const isClickedCompleted = task.status === 'completed' && clickedCompletedTasks.has(task.id);
            const showHighlight = task.status === 'completed' && !isClickedCompleted;
            
            return (
              <div 
                key={task.id} 
                className={`task-row ${task.id === currentTaskId ? 'active' : ''} ${showHighlight ? 'completed' : ''} ${isClickedCompleted ? 'completed-clicked' : ''} ${task.status === 'armed' ? 'armed' : ''} ${task.status === 'active' ? 'active-state' : ''}`}
                onClick={() => handleTaskClick(task)}
              >
                <span className={`mini-status status-${task.status}`}>
                  {task.status === 'running' ? '◉' : task.status === 'completed' ? '✓' : task.status === 'armed' ? '◎' : task.status === 'active' ? '◈' : '○'}
                </span>
                <span className="task-name-mini">{task.name}</span>
                <span className={`task-time-mini ${task.status === 'completed' ? 'completed-time' : ''} ${task.status === 'armed' ? 'armed-time' : ''} ${task.status === 'active' ? 'active-time' : ''}`}>
                  {task.status === 'completed' ? `✓ ${timeStr}` : timeStr}
                </span>
                {task.ide && <span className="ide-badge-mini">{task.ide}</span>}
              </div>
            );
          })}
        </div>
      ) : (
        /* Single task view */
        <>
          {displayTasks.length === 0 ? (
            <div className="app-header">
              <span className="app-icon">◆</span>
              <span className="app-title">Vibe Process Bar</span>
            </div>
          ) : (() => {
            // Calculate elapsed time for single task (not for armed/active)
            const task = currentTask;
            let elapsedTime: string | undefined;
            if (task) {
              if (task.status === 'armed') {
                elapsedTime = '⏳'; // Armed: waiting
              } else if (task.status === 'active') {
                elapsedTime = '👁'; // Active: window has focus
              } else if (task.status === 'completed') {
                const elapsed = (task.endTime || Date.now()) - task.startTime;
                const minutes = Math.floor(elapsed / 60000);
                const seconds = Math.floor((elapsed % 60000) / 1000);
                elapsedTime = minutes > 0 ? `${minutes}m ${seconds}s` : `${seconds}s`;
              } else if (task.startTime > 0) {
                const elapsed = Date.now() - task.startTime;
                const minutes = Math.floor(elapsed / 60000);
                const seconds = Math.floor((elapsed % 60000) / 1000);
                elapsedTime = minutes > 0 ? `${minutes}m ${seconds}s` : `${seconds}s`;
              }
            }
            return (
              <StatusText 
                taskName={currentTask?.name || ''}
                status={currentTask?.status || 'idle'}
                tokens={currentTask?.tokens || 0}
                ide={currentTask?.ide}
                onActivate={handleActivateWindow}
                elapsedTime={elapsedTime}
              />
            );
          })()}
        </>
      )}
      
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
        </>
      )}
    </div>
  );
}

export default App;
