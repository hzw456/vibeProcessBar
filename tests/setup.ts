import { vi, beforeEach } from 'vitest';

const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(() => {}),
  clear: vi.fn(),
  removeItem: vi.fn(),
  length: 0,
  key: vi.fn(),
};

beforeEach(() => {
  vi.stubGlobal('localStorage', localStorageMock);
});
