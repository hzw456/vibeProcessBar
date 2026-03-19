<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useProgressStore, type ProgressTask } from '../stores/progressStore';
import './SettingsPanel.css';

interface Props {
  isMainView?: boolean;
}

withDefaults(defineProps<Props>(), {
  isMainView: false,
});

const store = useProgressStore();
const { t } = useI18n();

type FilterStatus = 'all' | ProgressTask['status'];

const searchQuery = ref('');
const statusFilter = ref<FilterStatus>('all');
const isLoading = ref(false);
const loadError = ref('');
const expandedTaskIds = ref<string[]>([]);
const stageRecords = ref<Record<string, TaskStageRecord[]>>({});
const stageLoading = ref<Record<string, boolean>>({});
const stageErrors = ref<Record<string, string>>({});

const statusOptions: FilterStatus[] = ['all', 'running', 'armed', 'completed', 'cancelled', 'idle', 'error'];

interface TaskStageRecord {
  stage: string;
  description?: string;
  started_at?: number;
  ended_at?: number;
  duration?: number;
}

interface TaskStagesApiResponse {
  stages?: TaskStageRecord[];
}

const filteredHistory = computed(() => {
  const keyword = searchQuery.value.trim().toLowerCase();

  return store.history.filter((task) => {
    const matchesStatus = statusFilter.value === 'all' || task.status === statusFilter.value;
    if (!matchesStatus) {
      return false;
    }

    if (!keyword) {
      return true;
    }

    const haystack = [
      task.name,
      task.current_stage,
      task.ide,
      task.window_title,
      task.project_path,
      task.active_file,
      task.id,
    ]
      .filter(Boolean)
      .join(' ')
      .toLowerCase();

    return haystack.includes(keyword);
  });
});

function getStatusLabel(status: ProgressTask['status'] | 'all') {
  if (status === 'all') {
    return t('settings.tasks.filters.allStatuses');
  }
  return t(`settings.tasks.statuses.${status}`);
}

function getTaskTitle(task: ProgressTask) {
  return task.current_stage || task.name || task.window_title || task.id;
}

function getTaskMeta(task: ProgressTask) {
  return task.window_title || task.project_path || task.active_file || task.id;
}

function formatDate(value?: number) {
  if (!value) {
    return t('settings.tasks.notAvailable');
  }

  return new Intl.DateTimeFormat(undefined, {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

function formatDuration(task: ProgressTask) {
  if (!task.start_time) {
    return t('settings.tasks.notAvailable');
  }

  const end = task.end_time || Date.now();
  const durationMs = Math.max(0, end - task.start_time);
  const totalSeconds = Math.floor(durationMs / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  if (minutes > 0) {
    return t('time.minutesSeconds', { minutes, seconds });
  }

  return t('time.seconds', { seconds });
}

function getHistoryApiBaseUrl() {
  return (store.settings.backendServerUrl?.trim() || 'http://localhost:3010').replace(/\/+$/, '');
}

function getHistoryRequestHeaders() {
  const headers: Record<string, string> = {
    Accept: 'application/json',
  };
  const apiKey = store.settings.apiKey?.trim();
  if (apiKey) {
    headers['x-api-key'] = apiKey;
  }
  return headers;
}

function getStagesApiUrl(taskId: string) {
  return `${getHistoryApiBaseUrl()}/api/task/${encodeURIComponent(taskId)}/stages`;
}

function isExpanded(taskId: string) {
  return expandedTaskIds.value.includes(taskId);
}

function toggleExpanded(taskId: string) {
  if (isExpanded(taskId)) {
    expandedTaskIds.value = expandedTaskIds.value.filter((id) => id !== taskId);
    return;
  }

  expandedTaskIds.value = [...expandedTaskIds.value, taskId];

  if (!stageRecords.value[taskId] && !stageLoading.value[taskId]) {
    void fetchTaskStages(taskId);
  }
}

async function fetchTaskStages(taskId: string) {
  stageLoading.value = {
    ...stageLoading.value,
    [taskId]: true,
  };
  stageErrors.value = {
    ...stageErrors.value,
    [taskId]: '',
  };

  try {
    const response = await fetch(getStagesApiUrl(taskId), {
      method: 'GET',
      headers: getHistoryRequestHeaders(),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const payload = (await response.json()) as TaskStagesApiResponse;
    const stages = [...(payload.stages || [])].sort((a, b) => {
      const timeA = a.started_at || a.ended_at || 0;
      const timeB = b.started_at || b.ended_at || 0;
      return timeB - timeA;
    });

    stageRecords.value = {
      ...stageRecords.value,
      [taskId]: stages,
    };
  } catch (err) {
    stageErrors.value = {
      ...stageErrors.value,
      [taskId]: err instanceof Error ? err.message : String(err),
    };
  } finally {
    stageLoading.value = {
      ...stageLoading.value,
      [taskId]: false,
    };
  }
}

function formatStageDuration(duration?: number, startedAt?: number, endedAt?: number) {
  const durationMs = duration ?? (startedAt && endedAt ? Math.max(0, endedAt - startedAt) : undefined);
  if (durationMs === undefined) {
    return t('settings.tasks.notAvailable');
  }

  const totalSeconds = Math.floor(durationMs / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  if (minutes > 0) {
    return t('time.minutesSeconds', { minutes, seconds });
  }

  return t('time.seconds', { seconds });
}

async function refreshHistory() {
  isLoading.value = true;
  loadError.value = '';
  expandedTaskIds.value = [];
  stageRecords.value = {};
  stageLoading.value = {};
  stageErrors.value = {};

  try {
    await store.fetchHistory();
  } catch (err) {
    loadError.value = err instanceof Error ? err.message : String(err);
  } finally {
    isLoading.value = false;
  }
}

watch(() => [store.settings.backendServerUrl, store.settings.apiKey], refreshHistory);

onMounted(async () => {
  if (store.history.length === 0) {
    await refreshHistory();
  }
});
</script>

<template>
  <div :class="['settings-section', 'history-section', { 'history-panel-main': isMainView }]" @contextmenu.stop>
    <div class="history-toolbar">
      <input
        v-model="searchQuery"
        type="search"
        class="history-search-input"
        :placeholder="t('settings.tasks.searchPlaceholder')"
      />
      <select v-model="statusFilter" class="history-filter-select">
        <option v-for="option in statusOptions" :key="option" :value="option">
          {{ getStatusLabel(option) }}
        </option>
      </select>
      <button class="action-btn" @click="refreshHistory" :disabled="isLoading">
        {{ isLoading ? t('settings.tasks.loading') : t('settings.tasks.refresh') }}
      </button>
    </div>

    <div class="history-summary">
      <span>{{ t('settings.tasks.historyCount', { count: filteredHistory.length }) }}</span>
      <span v-if="store.history.length !== filteredHistory.length">
        {{ t('settings.tasks.filteredFrom', { total: store.history.length }) }}
      </span>
    </div>

    <div v-if="loadError" class="history-state history-error">
      {{ t('settings.tasks.loadError', { message: loadError }) }}
    </div>
    <div v-else-if="isLoading && store.history.length === 0" class="history-state">
      {{ t('settings.tasks.loading') }}
    </div>
    <div v-else-if="filteredHistory.length === 0" class="history-state">
      {{ t('settings.tasks.empty') }}
    </div>

    <div v-else class="history-list">
      <article
        v-for="task in filteredHistory"
        :key="task.id"
        :class="['history-card', { 'history-card-expanded': isExpanded(task.id) }]"
      >
        <button class="history-card-toggle" type="button" @click="toggleExpanded(task.id)">
          <div class="history-card-header">
            <div class="history-card-main">
              <div class="history-task-title" :title="getTaskTitle(task)">{{ getTaskTitle(task) }}</div>
              <div class="history-task-meta" :title="getTaskMeta(task)">{{ getTaskMeta(task) }}</div>
            </div>
            <div class="history-card-side">
              <span :class="['history-status-pill', `status-${task.status}`]">
                {{ getStatusLabel(task.status) }}
              </span>
              <span class="history-expand-indicator">{{ isExpanded(task.id) ? '−' : '+' }}</span>
            </div>
          </div>
        </button>

        <div class="history-grid">
          <div class="history-grid-item">
            <span class="history-grid-label">{{ t('settings.tasks.fields.ide') }}</span>
            <span>{{ task.ide || t('settings.tasks.notAvailable') }}</span>
          </div>
          <div class="history-grid-item">
            <span class="history-grid-label">{{ t('settings.tasks.fields.tokens') }}</span>
            <span>{{ task.tokens }}</span>
          </div>
          <div class="history-grid-item">
            <span class="history-grid-label">{{ t('settings.tasks.fields.startedAt') }}</span>
            <span>{{ formatDate(task.start_time) }}</span>
          </div>
          <div class="history-grid-item">
            <span class="history-grid-label">{{ t('settings.tasks.fields.duration') }}</span>
            <span>{{ formatDuration(task) }}</span>
          </div>
        </div>

        <div v-if="isExpanded(task.id)" class="history-stages">
          <div class="history-stages-title">{{ t('settings.tasks.stages.title') }}</div>
          <div v-if="stageLoading[task.id]" class="history-state">
            {{ t('settings.tasks.loading') }}
          </div>
          <div v-else-if="stageErrors[task.id]" class="history-state history-error">
            {{ t('settings.tasks.stages.loadError', { message: stageErrors[task.id] }) }}
          </div>
          <div v-else-if="!(stageRecords[task.id]?.length)" class="history-state">
            {{ t('settings.tasks.stages.empty') }}
          </div>
          <div v-else class="history-stage-list">
            <div v-for="(record, index) in stageRecords[task.id]" :key="`${task.id}-${index}`" class="history-stage-card">
              <div class="history-stage-header">
                <span class="history-stage-name">{{ record.stage || t('settings.tasks.notAvailable') }}</span>
                <span class="history-stage-duration">
                  {{ formatStageDuration(record.duration, record.started_at, record.ended_at) }}
                </span>
              </div>
              <div class="history-stage-description">
                {{ record.description || t('settings.tasks.notAvailable') }}
              </div>
            </div>
          </div>
        </div>
      </article>
    </div>
  </div>
</template>
