import { describe, expect, it } from 'vitest';
import type { ProgressTask } from '../src/stores/progressStore';
import { getHistoryMeta, getHistoryTitle, normalizeTaskHistory, type TaskStageRecord } from '../src/utils/history';

function createTask(overrides: Partial<ProgressTask> = {}): ProgressTask {
  return {
    id: 'task-1',
    name: 'Implement history redesign',
    tokens: 0,
    status: 'completed',
    start_time: 1_710_000_000_000,
    end_time: 1_710_000_060_000,
    current_stage: '__completed__',
    window_title: 'history.ts',
    project_path: '/repo',
    active_file: 'src/history.ts',
    ...overrides,
  };
}

describe('history utils', () => {
  it('adds a standalone terminal event and hides terminal sentinel stages', () => {
    const stages: TaskStageRecord[] = [
      {
        stage: 'src/history.ts',
        description: 'Implement timeline model',
        started_at: 1_710_000_000_000,
        ended_at: 1_710_000_030_000,
      },
      {
        stage: '__completed__',
        started_at: 1_710_000_060_000,
        ended_at: 1_710_000_060_000,
      },
    ];

    const events = normalizeTaskHistory(createTask(), stages);

    expect(events).toHaveLength(2);
    expect(events[0]).toMatchObject({
      kind: 'stage',
      title: 'Implement timeline model',
      subtitle: 'src/history.ts',
    });
    expect(events[1]).toMatchObject({
      kind: 'completed',
      title: 'completed',
      startedAt: 1_710_000_060_000,
    });
  });

  it('merges consecutive duplicate stages without collapsing distinct stages', () => {
    const stages: TaskStageRecord[] = [
      {
        stage: 'src/a.ts',
        description: 'Analyze',
        started_at: 1000,
        ended_at: 2000,
      },
      {
        stage: 'src/a.ts',
        description: 'Analyze',
        started_at: 2000,
        ended_at: 3000,
      },
      {
        stage: 'src/b.ts',
        description: 'Implement',
        started_at: 3000,
        ended_at: 5000,
      },
      {
        stage: 'src/a.ts',
        description: 'Analyze',
        started_at: 6000,
        ended_at: 7000,
      },
    ];

    const events = normalizeTaskHistory(createTask({ status: 'running', end_time: undefined }), stages);

    expect(events).toHaveLength(3);
    expect(events[0]).toMatchObject({ title: 'Analyze', startedAt: 1000, endedAt: 3000 });
    expect(events[1]).toMatchObject({ title: 'Implement', startedAt: 3000, endedAt: 5000 });
    expect(events[2]).toMatchObject({ title: 'Analyze', startedAt: 6000, endedAt: 7000 });
  });

  it('normalizes non-monotonic timestamps into a forward-only timeline', () => {
    const stages: TaskStageRecord[] = [
      {
        stage: 'src/a.ts',
        description: 'Plan',
        started_at: 5000,
        ended_at: 9000,
      },
      {
        stage: 'src/b.ts',
        description: 'Code',
        started_at: 3000,
        ended_at: 4000,
      },
      {
        stage: 'src/c.ts',
        description: 'Review',
        started_at: 8000,
        ended_at: 7000,
      },
    ];

    const events = normalizeTaskHistory(createTask({ status: 'running', end_time: undefined }), stages);

    expect(events).toHaveLength(3);
    expect(events[0]).toMatchObject({ title: 'Code', startedAt: 3000, endedAt: 4000 });
    expect(events[1]).toMatchObject({ title: 'Plan', startedAt: 5000, endedAt: 9000 });
    expect(events[2]).toMatchObject({ title: 'Review', startedAt: 9000, endedAt: 9000 });
  });

  it('avoids terminal sentinel values in summary title and meta', () => {
    const task = createTask({
      current_stage: '__completed__',
      name: 'History redesign',
      window_title: 'src/components/HistoryPanel.vue',
      active_file: 'src/utils/history.ts',
    });

    expect(getHistoryTitle(task)).toBe('History redesign');
    expect(getHistoryMeta(task)).toBe('History redesign');
  });
});
