import { describe, expect, it } from 'vitest';
import { shouldDisplayTask } from '../src/utils/taskFilters';
import type { ProgressTask } from '../src/stores/progressStore';

function createTask(overrides: Partial<ProgressTask> = {}): ProgressTask {
  return {
    id: 'task-1',
    name: 'Test Task',
    tokens: 0,
    status: 'armed',
    is_focused: false,
    start_time: Date.now(),
    ...overrides,
  };
}

describe('shouldDisplayTask', () => {
  it('shows running tasks even when they are not focused', () => {
    const task = createTask({ status: 'running', is_focused: false });

    expect(shouldDisplayTask(task, new Set(), true)).toBe(true);
  });

  it('keeps non-running tasks focus-gated when showOnlyWhenRunning is enabled', () => {
    const armedTask = createTask({ status: 'armed', is_focused: false });
    const focusedArmedTask = createTask({ id: 'task-2', status: 'armed', is_focused: true });

    expect(shouldDisplayTask(armedTask, new Set(), true)).toBe(false);
    expect(shouldDisplayTask(focusedArmedTask, new Set(), true)).toBe(true);
  });
});
