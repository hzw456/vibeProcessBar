<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useProgressStore, type ProgressTask } from '../stores/progressStore';
import { DEFAULT_BACKEND_SERVER_URL } from '../stores/defaultSettings';
import {
  getHistoryMeta,
  getHistoryTitle,
  normalizeTaskHistory,
  type HistoryTimelineEvent,
  type TaskStageRecord,
} from '../utils/history';
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
  return getHistoryTitle(task);
}

function getTaskMeta(task: ProgressTask) {
  return getHistoryMeta(task);
}

function formatDateTime(value?: number) {
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

function formatDateLabel(value?: number) {
  if (!value) {
    return t('settings.tasks.notAvailable');
  }

  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

function formatDurationMs(durationMs?: number) {
  if (durationMs === undefined) {
    return t('settings.tasks.notAvailable');
  }

  const totalSeconds = Math.floor(durationMs / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor(totalSeconds / 60);
  const remainingMinutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${remainingMinutes}m`;
  }

  if (minutes > 0) {
    return t('time.minutesSeconds', { minutes, seconds });
  }

  return t('time.seconds', { seconds });
}

function getTaskDurationMs(task: ProgressTask) {
  if (!task.start_time) {
    return undefined;
  }

  return Math.max(0, (task.end_time || Date.now()) - task.start_time);
}

function formatDuration(task: ProgressTask) {
  return formatDurationMs(getTaskDurationMs(task));
}

function getCompletedAt(task: ProgressTask) {
  return task.end_time || task.last_heartbeat || task.start_time || task.created_at;
}

function formatTokenCount(value?: number) {
  return new Intl.NumberFormat().format(value || 0);
}

function getTaskBadges(task: ProgressTask) {
  return [task.ide, task.adapter].filter(Boolean) as string[];
}

function getTaskDetailRows(task: ProgressTask) {
  return [
    {
      key: 'execution',
      label: t('settings.tasks.fields.executionTime'),
      value: getCompletedAt(task) ? formatDateTime(getCompletedAt(task)) : t('settings.tasks.notAvailable'),
    },
    {
      key: 'started',
      label: t('settings.tasks.fields.startedAt'),
      value: formatDateTime(task.start_time),
    },
    {
      key: 'duration',
      label: t('settings.tasks.fields.duration'),
      value: formatDuration(task),
    },
    {
      key: 'tokens',
      label: t('settings.tasks.fields.tokens'),
      value: formatTokenCount(task.tokens),
    },
    {
      key: 'project',
      label: t('settings.tasks.fields.project'),
      value: task.project_path || t('settings.tasks.notAvailable'),
    },
    {
      key: 'file',
      label: t('settings.tasks.fields.file'),
      value: task.active_file || t('settings.tasks.notAvailable'),
    },
  ];
}

function getHistoryApiBaseUrl() {
  return (store.settings.backendServerUrl?.trim() || DEFAULT_BACKEND_SERVER_URL).replace(/\/+$/, '');
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

    stageRecords.value = {
      ...stageRecords.value,
      [taskId]: payload.stages || [],
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
  return formatDurationMs(durationMs);
}

function getTimeline(task: ProgressTask): HistoryTimelineEvent[] {
  return normalizeTaskHistory(task, stageRecords.value[task.id] || []);
}

function formatEventTitle(event: HistoryTimelineEvent) {
  return event.kind === 'stage' ? event.title : getStatusLabel(event.kind);
}

function formatEventSubtitle(event: HistoryTimelineEvent) {
  return event.subtitle;
}

function formatEventTime(event: HistoryTimelineEvent) {
  if (event.startedAt && event.endedAt && event.startedAt !== event.endedAt) {
    return `${formatDateLabel(event.startedAt)} - ${formatDateLabel(event.endedAt)}`;
  }

  return formatDateLabel(event.startedAt || event.endedAt);
}

function getEventMomentLabel(event: HistoryTimelineEvent) {
  if (event.kind === 'stage' && event.startedAt && event.endedAt && event.startedAt !== event.endedAt) {
    return t('settings.tasks.stages.rangeLabel');
  }

  return t('settings.tasks.stages.momentLabel');
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
  <div :class="['settings-panel', 'history-panel', { standalone: isMainView }]" @click.stop @contextmenu.stop>
    <div class="settings-content history-content">
      <div class="settings-section history-section">
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
          <span v-else>{{ t('settings.tasks.sortedByRecent') }}</span>
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
            <button
              class="history-card-toggle"
              type="button"
              :aria-expanded="isExpanded(task.id)"
              @click="toggleExpanded(task.id)"
            >
              <div class="history-card-header">
                <div class="history-card-main">
                  <div class="history-card-topline">
                    <span :class="['history-status-pill', `status-${task.status}`]">
                      {{ getStatusLabel(task.status) }}
                    </span>
                    <span class="history-run-at">
                      {{ formatDateLabel(getCompletedAt(task)) }}
                    </span>
                  </div>
                  <div class="history-task-title" :title="getTaskTitle(task)">{{ getTaskTitle(task) }}</div>
                  <div class="history-task-meta" :title="getTaskMeta(task)">{{ getTaskMeta(task) }}</div>
                  <div v-if="getTaskBadges(task).length" class="history-card-badges">
                    <span v-for="badge in getTaskBadges(task)" :key="badge" class="history-badge">{{ badge }}</span>
                  </div>
                </div>
                <div class="history-card-side">
                  <span class="history-expand-indicator">{{ isExpanded(task.id) ? '−' : '+' }}</span>
                </div>
              </div>
              <div class="history-metrics">
                <div class="history-metric-card">
                  <span class="history-grid-label">{{ t('settings.tasks.fields.executionTime') }}</span>
                  <span class="history-metric-value">{{ formatDateTime(getCompletedAt(task)) }}</span>
                </div>
                <div class="history-metric-card">
                  <span class="history-grid-label">{{ t('settings.tasks.fields.duration') }}</span>
                  <span class="history-metric-value">{{ formatDuration(task) }}</span>
                </div>
                <div class="history-metric-card">
                  <span class="history-grid-label">{{ t('settings.tasks.fields.tokens') }}</span>
                  <span class="history-metric-value">{{ formatTokenCount(task.tokens) }}</span>
                </div>
              </div>
            </button>

            <div v-if="isExpanded(task.id)" class="history-stages">
              <div class="history-expanded-grid">
                <section class="history-details">
                  <div class="history-stages-title">{{ t('settings.tasks.detailsTitle') }}</div>
                  <div class="history-grid history-details-grid">
                    <div v-for="detail in getTaskDetailRows(task)" :key="detail.key" class="history-grid-item">
                      <span class="history-grid-label">{{ detail.label }}</span>
                      <span :title="detail.value">{{ detail.value }}</span>
                    </div>
                  </div>
                </section>

                <section class="history-timeline-panel">
                  <div class="history-stages-title">{{ t('settings.tasks.stages.title') }}</div>
                  <div v-if="stageLoading[task.id]" class="history-state">
                    {{ t('settings.tasks.loading') }}
                  </div>
                  <div v-else-if="stageErrors[task.id]" class="history-state history-error">
                    {{ t('settings.tasks.stages.loadError', { message: stageErrors[task.id] }) }}
                  </div>
                  <div v-else-if="!(getTimeline(task).length)" class="history-state">
                    {{ t('settings.tasks.stages.empty') }}
                  </div>
                  <div v-else class="history-stage-list history-timeline">
                    <div
                      v-for="event in getTimeline(task)"
                      :key="event.key"
                      :class="['history-stage-card', 'history-timeline-item', `kind-${event.kind}`]"
                    >
                      <div class="history-stage-header">
                        <div class="history-timeline-copy">
                          <span class="history-stage-name">{{ formatEventTitle(event) }}</span>
                          <span v-if="formatEventSubtitle(event)" class="history-stage-description">
                            {{ formatEventSubtitle(event) }}
                          </span>
                        </div>
                        <span class="history-stage-duration">
                          {{ formatStageDuration(event.duration, event.startedAt, event.endedAt) }}
                        </span>
                      </div>
                      <div class="history-timeline-time">
                        <span class="history-timeline-label">{{ getEventMomentLabel(event) }}</span>
                        <span>{{ formatEventTime(event) }}</span>
                      </div>
                    </div>
                  </div>
                </section>
              </div>
            </div>
          </article>
        </div>
      </div>
    </div>
  </div>
</template>
