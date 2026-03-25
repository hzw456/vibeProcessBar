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
    // Show running tasks, completed tasks (unless dismissed by user), and focused tasks
    if (task.status === 'completed' && clickedCompletedTasks?.has(task.id)) {
      return false;
    }
    return task.status === 'running' || task.status === 'completed' || task.is_focused === true;
  }

  return true;
}
