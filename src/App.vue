<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useProgressStore, type ProgressTask, type AppSettings } from './stores/progressStore';
import StatusText from './components/StatusText.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import { debug, error } from './utils/logger';
import { playCompletionSound } from './utils/notifications';

debug('App.vue loaded');

// Check if we're running in Tauri
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Safe wrappers for Tauri APIs
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  if (!isTauri) return null;
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

async function getCurrentWindowLabel(): Promise<string> {
  if (!isTauri) return 'main';
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  return getCurrentWindow().label;
}

async function getWindow() {
  if (!isTauri) return null;
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  return getCurrentWindow();
}

// Check if this is the settings window
const isSettingsWindow = ref(window.location.search.includes('type=settings'));
if (isTauri && !isSettingsWindow.value) {
    // Double check with label for main window, just in case, but rely on query param primarily
    getCurrentWindowLabel().then(label => {
        if (label === 'settings') isSettingsWindow.value = true;
        debug('Window label check', { label, isSettings: isSettingsWindow.value });
    });
} else {
    debug('Window type detected from URL', { isSettings: isSettingsWindow.value });
}

interface IdeWindow {
  bundle_id: string;
  ide: string;
  window_title: string;
  window_index: number;
}

const { t } = useI18n();
const store = useProgressStore();

// Reactive state
const containerRef = ref<HTMLDivElement | null>(null);
const showMenu = ref(false);
const completedTask = ref<string | null>(null);
const clickedCompletedTasks = ref<Set<string>>(new Set());
const seenActiveTasks = ref<Set<string>>(new Set());
const isCollapsed = ref(false);
const isCollapseTransition = ref(false);
const ideWindows = ref<IdeWindow[]>([]);
const prevTasks = ref<ProgressTask[]>([]);
const isActivatingRef = ref(false);

// Computed - 确保透明度响应式更新
const windowOpacity = computed(() => store.settings.opacity);

// Computed
const allDisplayItems = computed(() => {
  const items = [...store.tasks];
  
  ideWindows.value.forEach(win => {
    // Extract project name from window title (format: "filename — ProjectName" or just "ProjectName")
    const winTitleParts = win.window_title.split(' — ');
    const projectName = winTitleParts.length > 1 ? winTitleParts[winTitleParts.length - 1] : win.window_title;
    
    const existingTask = store.tasks.find(t =>
      t.ide === win.ide &&
      (
        t.window_title === win.window_title ||
        t.window_title === projectName ||
        win.window_title.includes(t.window_title || '') ||
        projectName.includes(t.window_title || '') ||
        (t.window_title || '').includes(projectName)
      )
    );
    if (!existingTask) {
      const virtualTask: ProgressTask = {
        id: `ide_${win.ide}_${win.window_index}`,
        name: win.window_title,
        progress: 0,
        tokens: 0,
        status: 'idle',
        startTime: 0,
        ide: win.ide,
        window_title: win.window_title,
      };
      items.push(virtualTask);
    }
  });

  // Sort by IDE then name for stable order
  items.sort((a, b) => {
    const ideA = a.ide || '';
    const ideB = b.ide || '';
    if (ideA !== ideB) return ideA.localeCompare(ideB);
    const nameA = a.window_title || a.name || '';
    const nameB = b.window_title || b.name || '';
    return nameA.localeCompare(nameB);
  });

  return items;
});

const displayTasks = computed(() =>
  allDisplayItems.value.filter(t =>
    ['completed', 'running', 'armed', 'idle'].includes(t.status)
  )
);

// 单任务视图直接用第一个任务
const singleTask = computed(() => displayTasks.value[0] || null);

// Scan IDE windows
async function scanIdeWindows() {
  try {
    const windows = await safeInvoke<IdeWindow[]>('get_ide_windows');
    if (windows) {
      debug('Scanned IDE windows', { count: windows.length });
      ideWindows.value = windows;
    }
  } catch (err) {
    error('Failed to scan IDE windows', { error: String(err) });
  }
}



// Handle task double click - navigate to IDE
async function handleTaskDoubleClick(task: ProgressTask) {
  if (isActivatingRef.value) return;
  isActivatingRef.value = true;
  debug('Task double-clicked', { taskId: task.id, ide: task.ide });

  if (task.status === 'completed') {
    clickedCompletedTasks.value = new Set([...clickedCompletedTasks.value, task.id]);
    // 双击跳转后重置为 armed 状态
    try {
      await safeInvoke('reset_task_to_armed', { taskId: task.id });
    } catch (err) {
      error('Failed to reset task to armed', { error: String(err) });
    }
  }

  if (task.ide) {
    try {
      await safeInvoke('activate_ide_window', {
        ide: task.ide,
        windowTitle: task.window_title || null,
        projectPath: task.project_path || null,
        activeFile: task.active_file || null
      });
    } catch (err) {
      error('Failed to activate IDE window', { error: String(err) });
    } finally {
      setTimeout(() => { isActivatingRef.value = false; }, 500);
    }
  } else {
    isActivatingRef.value = false;
  }
}

// Handle collapse/expand
async function handleCollapse() {
  const expandedWidth = 280;
  const collapsedWidth = 120;
  const widthDiff = expandedWidth - collapsedWidth;

  isCollapseTransition.value = true;

  try {
    const win = await getWindow();
    if (!win) {
      isCollapsed.value = !isCollapsed.value;
      return;
    }
    const position = await win.innerPosition();
    const scaleFactor = await win.scaleFactor();

    const logicalX = position.x / scaleFactor;
    const logicalY = position.y / scaleFactor;

    const currentSize = await win.innerSize();
    const currentHeight = Math.round(currentSize.height / scaleFactor);

    if (!isCollapsed.value) {
      await safeInvoke('resize_window', { width: collapsedWidth, height: currentHeight });
      await safeInvoke('set_window_position', { x: logicalX + widthDiff, y: logicalY });
    } else {
      await safeInvoke('set_window_position', { x: logicalX - widthDiff, y: logicalY });
      await safeInvoke('resize_window', { width: expandedWidth, height: currentHeight });
    }

    isCollapsed.value = !isCollapsed.value;
  } catch (e) {
    error('Failed to adjust window position', { error: String(e) });
    isCollapsed.value = !isCollapsed.value;
  } finally {
    setTimeout(() => { isCollapseTransition.value = false; }, 100);
  }
}

// Open settings
async function handleOpenSettings() {
  setShowMenu(false);
  try {
    await safeInvoke('open_settings_window');
  } catch (err) {
    error('Failed to open settings', { error: String(err) });
  }
}

function setShowMenu(value: boolean) {
  showMenu.value = value;
}

// Handle mouse down for window dragging
async function handleMouseDown(event: MouseEvent) {
  // Only start drag if clicking on the container itself (not buttons or interactive elements)
  const target = event.target as HTMLElement;
  if (target.closest('button') || target.closest('.task-row') || target.closest('.menu-item') || target.closest('.context-menu')) {
    return;
  }
  try {
    const win = await getWindow();
    if (win) {
      // 标记正在拖动
      isDragging = true;
      await win.startDragging();
    }
  } catch {
    // Ignore drag errors
  }
}

// 标记是否正在拖动
let isDragging = false;

// 监听全局 mouseup 来检测拖动结束
if (typeof window !== 'undefined') {
  window.addEventListener('mouseup', async () => {
    if (isDragging) {
      isDragging = false;
      // 延迟一点确保位置已更新
      setTimeout(async () => {
        await saveWindowPosition();
      }, 100);
    }
  });
}

// 保存窗口位置到配置文件
async function saveWindowPosition() {
  try {
    const win = await getWindow();
    if (win) {
      const position = await win.innerPosition();
      const scaleFactor = await win.scaleFactor();
      const x = Math.round(position.x / scaleFactor);
      const y = Math.round(position.y / scaleFactor);
      store.updateWindowPositionDisplay(x, y);
      await store.saveWindowPositionToFile();
      debug('Window position saved', { x, y });
    }
  } catch (e) {
    error('Failed to save window position', { error: String(e) });
  }
}

// 更新位置显示（不保存到文件）
async function updatePositionDisplay() {
  try {
    const win = await getWindow();
    if (win) {
      const position = await win.innerPosition();
      const scaleFactor = await win.scaleFactor();
      const x = Math.round(position.x / scaleFactor);
      const y = Math.round(position.y / scaleFactor);
      store.updateWindowPositionDisplay(x, y);
    }
  } catch {
    // Ignore errors
  }
}

// Get time string for task
function getTimeStr(task: ProgressTask): string {
  // Focused state does NOT affect time display - only the icon changes
  if (task.status === 'armed') return '⏳';
  if (task.status === 'completed' && task.start_time > 0) {
    const elapsed = (task.end_time || Date.now()) - task.start_time;
    const minutes = Math.floor(elapsed / 60000);
    const seconds = Math.floor((elapsed % 60000) / 1000);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }
  if (task.status === 'running' && task.start_time > 0) {
    const elapsed = Date.now() - task.start_time;
    const minutes = Math.floor(elapsed / 60000);
    const seconds = Math.floor((elapsed % 60000) / 1000);
    const elapsedStr = `${minutes}:${seconds.toString().padStart(2, '0')}`;
    
    // 如果有预估时长，显示 已用时间/总时间
    if (task.estimated_duration && task.estimated_duration > 0) {
      const totalMinutes = Math.floor(task.estimated_duration / 60000);
      const totalSeconds = Math.floor((task.estimated_duration % 60000) / 1000);
      const totalStr = `${totalMinutes}:${totalSeconds.toString().padStart(2, '0')}`;
      return `${elapsedStr}/${totalStr}`;
    }
    return elapsedStr;
  }
  return '';
}

// 根据已用时间/总时间计算进度百分比
function getTimeProgress(task: ProgressTask): number {
  if (task.status !== 'running' || !task.start_time || !task.estimated_duration || task.estimated_duration <= 0) {
    return 0;
  }
  const elapsed = Date.now() - task.start_time;
  const progress = (elapsed / task.estimated_duration) * 100;
  return Math.min(99, Math.max(0, progress)); // 最大99%，完成时才100%
}

// Get status icon - focused state shows eye icon, otherwise based on status
function getStatusIcon(task: ProgressTask): string {
  // Focused window shows eye icon (only icon changes, not other styles)
  if (task.is_focused) return '👁';
  switch (task.status) {
    case 'running': return '◉';
    case 'completed': return '✓';
    case 'armed': return '◎';
    default: return '○';
  }
}

// Get display name: 优先显示 current_stage，否则显示 activeFile - workspace 格式
function getDisplayName(task: ProgressTask): string {
  // 如果有当前阶段描述，优先显示
  if (task.current_stage) {
    // 特殊标记 __completed__ 使用 i18n 翻译
    if (task.current_stage === '__completed__') {
      return t('status.completed');
    }
    return task.current_stage;
  }
  
  const activeFile = task.active_file ? task.active_file.split('/').pop() || task.active_file : null;
  const workspace = task.project_path ? task.project_path.split('/').pop() || task.project_path : null;
  
  if (activeFile && workspace) {
    return `${activeFile} - ${workspace}`;
  }
  if (workspace) {
    return workspace;
  }
  if (activeFile) {
    return activeFile;
  }
  return task.name;
}

// Watch settings changes and apply CSS variables
// Note: Settings application is now handled by the store actions (setSettings, loadSettings, etc.)
// We don't need a watcher here to avoid double application and conflicts

// Track running/focused tasks and detect completions
watch(() => store.tasks, (newTasks) => {
  // If settings window, checking completions might not be critical, but keeping it is harmless
  if (isSettingsWindow.value) return;

  // Track focused/running tasks
  newTasks.forEach(task => {
    if (task.is_focused || task.status === 'running') {
      seenActiveTasks.value = new Set([...seenActiveTasks.value, task.id]);
    }
    
    // 当任务重新开始（armed/running）时，从 clickedCompletedTasks 中移除
    if (task.status === 'armed' || task.status === 'running') {
      if (clickedCompletedTasks.value.has(task.id)) {
        const newSet = new Set(clickedCompletedTasks.value);
        newSet.delete(task.id);
        clickedCompletedTasks.value = newSet;
      }
    }
  });

  // Note: completed + focused -> armed transition is now handled by backend
  // in the /api/task/active endpoint when the extension sends heartbeat

  // Detect completions - show completion notification and play sound
  newTasks.forEach(task => {
    const prevTask = prevTasks.value.find(t => t.id === task.id);
    if (prevTask && prevTask.status !== 'completed' && task.status === 'completed') {
      // Show completion notification only for background tasks (not seen active)
      if (!seenActiveTasks.value.has(task.id)) {
        completedTask.value = task.id;
        setTimeout(() => { completedTask.value = null; }, 3000);
      }
      // Play completion sound for all completed tasks if enabled
      if (store.settings.sound) {
        playCompletionSound(store.settings.soundVolume);
      }
      // Note: Do NOT add to clickedCompletedTasks here
      // Green border should always show for completed tasks
      // clickedCompletedTasks is only updated on handleTaskDoubleClick
    }
  });

  prevTasks.value = [...newTasks];
}, { deep: true });

// Dynamic window resize
watch([displayTasks, isCollapsed, isCollapseTransition], async () => {
  // Skip resizing for settings window
  if (isSettingsWindow.value) return;
  if (isCollapseTransition.value) return;

  const taskCount = displayTasks.value.length;
  const taskHeight = 36;  // Row height (8px padding * 2 + ~20px content)
  const rowGap = 6;       // Gap between rows (.multi-task-list gap)
  const padding = 10;     // Match CSS padding (10px top + bottom)

  let newHeight = padding;
  if (taskCount === 0) {
    newHeight = 60;
  } else if (taskCount === 1) {
    newHeight = 70;
  } else {
    // Multi-task: padding + (rows * height) + ((rows-1) * gap)
    newHeight = padding + (taskCount * taskHeight) + ((taskCount - 1) * rowGap);
  }

  const width = isCollapsed.value ? 120 : 280;

  try {
    await safeInvoke('resize_window', { width, height: Math.max(50, newHeight) });
  } catch (e) {
    error('Failed to resize window', { error: String(e) });
  }
});

  // Intervals
  let syncInterval: number;
  let scanInterval: number;
  let forceUpdateInterval: number;
  let settingsPollInterval: number;
  let unlistenMove: (() => void) | null = null;

  onMounted(async () => {
    // 初始化事件监听 (tasks-updated, settings-changed)
    await store.initEventListeners();

    // Always load settings to apply theme
    await store.loadSettings();

    // The rest is only for Main Window
    if (!isSettingsWindow.value) {
        // 监听窗口移动事件，实时更新位置显示
        if (isTauri) {
          const { listen } = await import('@tauri-apps/api/event');
          unlistenMove = await listen('tauri://move', () => {
            updatePositionDisplay();
          });
        }

        // Initial scan
        await scanIdeWindows();
        await store.fetchTasks();

        // Set up intervals - 使用 fetchTasks 替代 syncFromHttpApi
        syncInterval = window.setInterval(() => store.fetchTasks(), 1000);
        scanInterval = window.setInterval(scanIdeWindows, 5000);
        forceUpdateInterval = window.setInterval(() => {}, 1000); // Force update for time
        
        // Poll settings every 2 seconds to ensure sync across windows even if events are missed
        settingsPollInterval = window.setInterval(() => {
          store.refreshSettings();
        }, 2000);
    }
  });

  onUnmounted(() => {
    if (syncInterval) clearInterval(syncInterval);
    if (scanInterval) clearInterval(scanInterval);
    if (forceUpdateInterval) clearInterval(forceUpdateInterval);
    if (settingsPollInterval) clearInterval(settingsPollInterval);
    if (unlistenMove) unlistenMove();
    store.cleanupEventListeners();
  });
</script>

<template>
  <!-- Settings Window -->
  <div v-if="isSettingsWindow" class="settings-window-container">
    <SettingsPanel :is-standalone="true" />
  </div>

  <!-- Main Window -->
  <div
    v-else
    ref="containerRef"
    :class="['app-container', { collapsed: isCollapsed, 'multi-task': displayTasks.length > 1, 'has-completed': !!completedTask }]"
    :style="{ opacity: windowOpacity }"
    @mousedown="handleMouseDown"
  >
    <!-- Collapse/Expand button -->
    <button
      :class="['collapse-btn', { expanded: isCollapsed }]"
      @click="handleCollapse"
    >
      {{ isCollapsed ? '‹' : '›' }}
    </button>

    <!-- Completed notification -->
    <div v-if="completedTask" class="completed-banner">
      ✓ {{ t('notification.taskCompleted', { taskName: store.tasks.find(t => t.id === completedTask)?.name || t('menu.title') }) }}
    </div>

    <!-- Multi-task view -->
    <div v-if="displayTasks.length > 1" class="multi-task-list">
      <div
        v-for="task in displayTasks"
        :key="task.id"
        :class="[
          'task-row',
          { completed: task.status === 'completed' && !clickedCompletedTasks.has(task.id) },
          { 'completed-clicked': clickedCompletedTasks.has(task.id) },
          { armed: task.status === 'armed' },
          { 'focused-state': task.is_focused }
        ]"
        :style="{ '--progress': getTimeProgress(task) + '%' }"
        @dblclick="handleTaskDoubleClick(task)"
      >
        <span :class="['mini-status', `status-${task.status}`]">
          {{ getStatusIcon(task) }}
        </span>
        <!-- Collapsed: show status + IDE badge -->
        <template v-if="isCollapsed">
          <span v-if="task.ide" class="ide-badge-mini">{{ task.ide }}</span>
          <span v-else class="task-ide-collapsed" :title="task.name">{{ task.name }}</span>
        </template>
        <!-- Expanded: show task name without IDE prefix -->
        <template v-else>
          <span class="task-name-mini">{{ getDisplayName(task) }}</span>
          <span :class="['task-time-mini', { 'completed-time': task.status === 'completed', 'armed-time': task.status === 'armed' }]">
            {{ task.status === 'completed' ? `✓ ${getTimeStr(task)}` : getTimeStr(task) }}
          </span>
          <span v-if="task.ide" class="ide-badge-mini">{{ task.ide }}</span>
        </template>
      </div>
    </div>

    <!-- Single task view -->
    <template v-else>
      <div v-if="displayTasks.length === 0" class="app-header">
        <span class="app-icon">{{ t('app.icon') }}</span>
        <span class="app-title">{{ t('app.title') }}</span>
      </div>
      <!-- Collapsed: show status + IDE badge -->
      <div v-else-if="isCollapsed && singleTask" class="collapsed-single-task">
        <span :class="['mini-status', `status-${singleTask.status}`]">
          {{ getStatusIcon(singleTask) }}
        </span>
        <span v-if="singleTask.ide" class="ide-badge-mini">{{ singleTask.ide }}</span>
        <span v-else class="task-ide-collapsed" :title="singleTask.name">{{ singleTask.name }}</span>
      </div>
      <!-- Expanded: show full status text (same layout as multi-task) -->
      <div
        v-else-if="singleTask"
        :class="[
          'task-row',
          'single-task-row',
          { completed: singleTask.status === 'completed' && !clickedCompletedTasks.has(singleTask.id) },
          { 'completed-clicked': clickedCompletedTasks.has(singleTask.id) },
          { armed: singleTask.status === 'armed' },
          { 'focused-state': singleTask.is_focused }
        ]"
        :style="{ '--progress': getTimeProgress(singleTask) + '%' }"
        @dblclick="handleTaskDoubleClick(singleTask)"
      >
        <span :class="['mini-status', `status-${singleTask.status}`]">
          {{ getStatusIcon(singleTask) }}
        </span>
        <span class="task-name-mini">{{ getDisplayName(singleTask) }}</span>
        <span :class="['task-time-mini', { 'completed-time': singleTask.status === 'completed', 'armed-time': singleTask.status === 'armed' }]">
          {{ singleTask.status === 'completed' ? `✓ ${getTimeStr(singleTask)}` : getTimeStr(singleTask) }}
        </span>
        <span v-if="singleTask.ide" class="ide-badge-mini">{{ singleTask.ide }}</span>
      </div>
    </template>

    <!-- Context menu -->
    <div v-if="showMenu" class="context-menu">
      <div class="menu-item" @click="handleOpenSettings">
        {{ t('menu.settings') }}
      </div>
      <div class="menu-item" @click="setShowMenu(false)">
        {{ t('menu.closeMenu') }}
      </div>
    </div>
  </div>
</template>
