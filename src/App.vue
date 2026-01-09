<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useProgressStore, type ProgressTask } from './stores/progressStore';
import StatusText from './components/StatusText.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import { debug, error } from './utils/logger';

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
const isSettingsWindow = ref(false);
getCurrentWindowLabel().then(label => {
  isSettingsWindow.value = label === 'settings';
  debug('Window label', { label, isSettings: isSettingsWindow.value });
});

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

// Computed
const allDisplayItems = computed(() => {
  const items = [...store.tasks];
  
  ideWindows.value.forEach(win => {
    const existingTask = store.tasks.find(t =>
      t.ide === win.ide &&
      (t.windowTitle === win.window_title || win.window_title.includes(t.windowTitle || ''))
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
        windowTitle: win.window_title,
      };
      items.push(virtualTask);
    }
  });

  // Sort by IDE then name for stable order
  items.sort((a, b) => {
    const ideA = a.ide || '';
    const ideB = b.ide || '';
    if (ideA !== ideB) return ideA.localeCompare(ideB);
    const nameA = a.windowTitle || a.name || '';
    const nameB = b.windowTitle || b.name || '';
    return nameA.localeCompare(nameB);
  });

  return items;
});

const displayTasks = computed(() =>
  allDisplayItems.value.filter(t =>
    ['completed', 'running', 'idle', 'armed', 'active', 'registered'].includes(t.status)
  )
);

const currentTask = computed(() =>
  store.tasks.find(t => t.id === store.currentTaskId) ||
  store.tasks.find(t => t.status === 'running') ||
  displayTasks.value[0] || null
);

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

// Handle task click
function handleTaskClick(task: ProgressTask) {
  debug('Task clicked', { taskId: task.id });
  store.setCurrentTask(task.id);
}

// Handle task double click - navigate to IDE
async function handleTaskDoubleClick(task: ProgressTask) {
  if (isActivatingRef.value) return;
  isActivatingRef.value = true;
  debug('Task double-clicked', { taskId: task.id, ide: task.ide });

  if (task.status === 'completed') {
    clickedCompletedTasks.value = new Set([...clickedCompletedTasks.value, task.id]);
  }

  if (task.ide) {
    try {
      await safeInvoke('activate_ide_window', {
        ide: task.ide,
        windowTitle: task.windowTitle || null,
        projectPath: task.projectPath || null,
        activeFile: task.activeFile || null
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
      await win.startDragging();
    }
  } catch {
    // Ignore drag errors
  }
}

// Get time string for task
function getTimeStr(task: ProgressTask): string {
  if (task.status === 'armed') return '⏳';
  if (task.status === 'active') return '👁';
  if (task.status === 'registered') return '◇';
  if (task.status === 'completed' && task.startTime > 0) {
    const elapsed = (task.endTime || Date.now()) - task.startTime;
    const minutes = Math.floor(elapsed / 60000);
    const seconds = Math.floor((elapsed % 60000) / 1000);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }
  if (task.status === 'running' && task.startTime > 0) {
    const elapsed = Date.now() - task.startTime;
    const minutes = Math.floor(elapsed / 60000);
    const seconds = Math.floor((elapsed % 60000) / 1000);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }
  return '';
}

// Get status icon
function getStatusIcon(status: string): string {
  switch (status) {
    case 'running': return '◉';
    case 'completed': return '✓';
    case 'armed': return '◎';
    case 'active': return '◈';
    case 'registered': return '◇';
    default: return '○';
  }
}

// Track active tasks and detect completions
watch(() => store.tasks, (newTasks) => {
  // Track active/running tasks
  newTasks.forEach(task => {
    if (task.status === 'active' || task.status === 'running') {
      seenActiveTasks.value = new Set([...seenActiveTasks.value, task.id]);
    }
  });

  // Detect completions
  newTasks.forEach(task => {
    const prevTask = prevTasks.value.find(t => t.id === task.id);
    if (prevTask && prevTask.status !== 'completed' && task.status === 'completed') {
      if (!seenActiveTasks.value.has(task.id)) {
        completedTask.value = task.id;
        setTimeout(() => { completedTask.value = null; }, 3000);
      }
      if (seenActiveTasks.value.has(task.id)) {
        clickedCompletedTasks.value = new Set([...clickedCompletedTasks.value, task.id]);
      }
    }
  });

  prevTasks.value = [...newTasks];
}, { deep: true });

// Dynamic window resize
watch([displayTasks, isCollapsed, isCollapseTransition], async () => {
  if (isCollapseTransition.value) return;

  const taskCount = displayTasks.value.length;
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

onMounted(async () => {
  // Initial scan
  await scanIdeWindows();
  await store.syncFromHttpApi();

  // Set up intervals
  syncInterval = window.setInterval(() => store.syncFromHttpApi(), 1000);
  scanInterval = window.setInterval(scanIdeWindows, 5000);
  forceUpdateInterval = window.setInterval(() => {}, 1000); // Force update for time
});

onUnmounted(() => {
  clearInterval(syncInterval);
  clearInterval(scanInterval);
  clearInterval(forceUpdateInterval);
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
    :style="{ opacity: store.settings.opacity }"
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
          { active: task.id === store.currentTaskId },
          { completed: task.status === 'completed' && !clickedCompletedTasks.has(task.id) },
          { 'completed-clicked': clickedCompletedTasks.has(task.id) },
          { armed: task.status === 'armed' },
          { 'active-state': task.status === 'active' }
        ]"
        @click="handleTaskClick(task)"
        @dblclick="handleTaskDoubleClick(task)"
      >
        <span :class="['mini-status', `status-${task.status}`]">
          {{ getStatusIcon(task.status) }}
        </span>
        <span class="task-name-mini">{{ task.name }}</span>
        <span :class="['task-time-mini', { 'completed-time': task.status === 'completed', 'armed-time': task.status === 'armed', 'active-time': task.status === 'active' }]">
          {{ task.status === 'completed' ? `✓ ${getTimeStr(task)}` : getTimeStr(task) }}
        </span>
        <span v-if="task.ide" class="ide-badge-mini">{{ task.ide }}</span>
      </div>
    </div>

    <!-- Single task view -->
    <template v-else>
      <div v-if="displayTasks.length === 0" class="app-header">
        <span class="app-icon">{{ t('app.icon') }}</span>
        <span class="app-title">{{ t('app.title') }}</span>
      </div>
      <StatusText
        v-else-if="currentTask"
        :status="currentTask.status"
        :name="currentTask.name"
        :elapsed-time="getTimeStr(currentTask)"
        :show-icon="true"
      />
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
