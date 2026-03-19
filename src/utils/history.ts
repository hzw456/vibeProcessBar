import type { ProgressTask } from '../stores/progressStore';

export interface TaskStageRecord {
  stage: string;
  description?: string;
  started_at?: number;
  ended_at?: number;
  duration?: number;
}

export interface HistoryTimelineEvent {
  key: string;
  kind: 'stage' | 'completed' | 'error' | 'cancelled';
  title: string;
  subtitle?: string;
  startedAt?: number;
  endedAt?: number;
  duration?: number;
}

const TERMINAL_STAGE_NAMES = new Set(['__completed__', 'completed', 'error', 'cancelled']);
const TERMINAL_EVENT_KINDS = {
  completed: 'completed',
  error: 'error',
  cancelled: 'cancelled',
} satisfies Record<'completed' | 'error' | 'cancelled', HistoryTimelineEvent['kind']>;

type TerminalTaskStatus = keyof typeof TERMINAL_EVENT_KINDS;

function normalizeText(value?: string): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function isTerminalTaskStatus(status: ProgressTask['status']): status is TerminalTaskStatus {
  return status in TERMINAL_EVENT_KINDS;
}

function getStageSortTime(record: TaskStageRecord): number {
  return record.started_at ?? record.ended_at ?? Number.MAX_SAFE_INTEGER;
}

function normalizeStageRecord(
  record: TaskStageRecord,
  previousTime?: number,
): TaskStageRecord | null {
  const stage = normalizeText(record.stage);
  const description = normalizeText(record.description);

  if (!stage) {
    return null;
  }

  let startedAt = record.started_at;
  let endedAt = record.ended_at;

  if (previousTime !== undefined) {
    if (startedAt === undefined || startedAt < previousTime) {
      startedAt = previousTime;
    }
    if (endedAt !== undefined && endedAt < startedAt) {
      endedAt = startedAt;
    }
  } else if (startedAt !== undefined && endedAt !== undefined && endedAt < startedAt) {
    endedAt = startedAt;
  }

  const duration = record.duration ?? (
    startedAt !== undefined && endedAt !== undefined
      ? Math.max(0, endedAt - startedAt)
      : undefined
  );

  return {
    stage,
    description,
    started_at: startedAt,
    ended_at: endedAt,
    duration,
  };
}

function isTerminalStage(record: TaskStageRecord): boolean {
  return TERMINAL_STAGE_NAMES.has(record.stage.toLowerCase());
}

function isSameStage(left: TaskStageRecord, right: TaskStageRecord): boolean {
  return left.stage === right.stage && (left.description || '') === (right.description || '');
}

function mergeStageRecords(left: TaskStageRecord, right: TaskStageRecord): TaskStageRecord {
  const startedAt = [left.started_at, right.started_at]
    .filter((value): value is number => value !== undefined)
    .reduce<number | undefined>((min, value) => (min === undefined ? value : Math.min(min, value)), undefined);
  const endedAt = [left.ended_at, right.ended_at]
    .filter((value): value is number => value !== undefined)
    .reduce<number | undefined>((max, value) => (max === undefined ? value : Math.max(max, value)), undefined);

  return {
    stage: left.stage,
    description: left.description || right.description,
    started_at: startedAt,
    ended_at: endedAt,
    duration: (
      startedAt !== undefined && endedAt !== undefined
        ? Math.max(0, endedAt - startedAt)
        : left.duration ?? right.duration
    ),
  };
}

function toStageEvent(record: TaskStageRecord, index: number): HistoryTimelineEvent {
  const title = record.description || record.stage;
  const subtitle = record.description && record.description !== record.stage ? record.stage : undefined;

  return {
    key: `stage-${index}-${record.stage}-${record.started_at ?? record.ended_at ?? 'na'}`,
    kind: 'stage',
    title,
    subtitle,
    startedAt: record.started_at,
    endedAt: record.ended_at,
    duration: record.duration,
  };
}

function toTerminalEvent(task: ProgressTask): HistoryTimelineEvent | null {
  if (!isTerminalTaskStatus(task.status)) {
    return null;
  }

  return {
    key: `terminal-${task.id}-${task.status}-${task.end_time ?? 'na'}`,
    kind: TERMINAL_EVENT_KINDS[task.status],
    title: task.status,
    startedAt: task.end_time,
    endedAt: task.end_time,
    duration: 0,
  };
}

export function normalizeTaskHistory(task: ProgressTask, stages: TaskStageRecord[]): HistoryTimelineEvent[] {
  const sorted = [...stages].sort((left, right) => getStageSortTime(left) - getStageSortTime(right));

  const normalizedStages: TaskStageRecord[] = [];
  let previousTime: number | undefined;

  for (const record of sorted) {
    const normalized = normalizeStageRecord(record, previousTime);
    if (!normalized || isTerminalStage(normalized)) {
      continue;
    }

    const recordTime = normalized.ended_at ?? normalized.started_at;
    if (recordTime !== undefined) {
      previousTime = recordTime;
    }

    const last = normalizedStages[normalizedStages.length - 1];
    if (last && isSameStage(last, normalized)) {
      normalizedStages[normalizedStages.length - 1] = mergeStageRecords(last, normalized);
      continue;
    }

    normalizedStages.push(normalized);
  }

  const events = normalizedStages.map(toStageEvent);
  const terminalEvent = toTerminalEvent(task);
  if (terminalEvent) {
    events.push(terminalEvent);
  }

  return events;
}

export function getHistoryTitle(task: ProgressTask): string {
  const currentStage = normalizeText(task.current_stage);
  if (currentStage && !TERMINAL_STAGE_NAMES.has(currentStage.toLowerCase())) {
    return currentStage;
  }

  return normalizeText(task.name)
    || normalizeText(task.window_title)
    || normalizeText(task.active_file)
    || task.id;
}

export function getHistoryMeta(task: ProgressTask): string {
  const parts = [
    normalizeText(task.window_title),
    normalizeText(task.project_path),
    normalizeText(task.active_file),
    normalizeText(task.name),
  ].filter((value, index, items) => value && items.indexOf(value) === index);

  return parts.find(Boolean) || task.id;
}
