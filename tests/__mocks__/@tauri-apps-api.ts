import { vi } from 'vitest';

export const invoke = vi.fn().mockResolvedValue(undefined);
export const emit = vi.fn().mockResolvedValue(undefined);

export const getCurrentWindow = vi.fn().mockReturnValue({
  startDragging: vi.fn(),
});

export const exit = vi.fn();
export const relaunch = vi.fn();
