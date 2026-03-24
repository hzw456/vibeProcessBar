import type { ProgressTask } from '../stores/progressStore';

const DISPLAYABLE_STATUSES = new Set<ProgressTask['status']>([
  'completed',
  'cancelled',
  'running',
  'armed',
  'idle',
]);

export function shouldDisplayTask(
  task: ProgressTask,
  hiddenTaskIds: Set<string>,
  showOnlyWhenRunning: boolean,
  clickedCompletedTasks?: Set<string>,
): boolean {
  if (hiddenTaskIds.has(task.id)) {
    return false;
  }

  if (!DISPLAYABLE_STATUSES.has(task.status)) {
    return false;
  }

  if (showOnlyWhenRunning) {
    // Completed tasks are visible unless the user has double-clicked on them
    if (task.status === 'completed' && clickedCompletedTasks?.has(task.id)) {
      return false;
    }
    return task.status === 'running';
  }

  return true;
}
