import type { ProgressTask } from '../stores/progressStore';

const DISPLAYABLE_STATUSES = new Set<ProgressTask['status']>([
  'completed',
  'running',
  'armed',
  'idle',
]);

export function shouldDisplayTask(
  task: ProgressTask,
  hiddenTaskIds: Set<string>,
  showOnlyWhenRunning: boolean,
): boolean {
  if (hiddenTaskIds.has(task.id)) {
    return false;
  }

  if (!DISPLAYABLE_STATUSES.has(task.status)) {
    return false;
  }

  if (showOnlyWhenRunning) {
    return task.status === 'running';
  }

  return true;
}
