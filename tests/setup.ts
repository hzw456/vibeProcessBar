import { vi, beforeAll, afterAll } from 'vitest';

const storageMock = {
  getItem: vi.fn((_key: string) => null),
  setItem: vi.fn((_key: string, _value: string) => {}),
  removeItem: vi.fn((_key: string) => {}),
  clear: vi.fn(),
  get length(): number { return 0 },
  key: vi.fn((_index: number) => null),
};

beforeAll(() => {
  vi.stubGlobal('localStorage', storageMock);
  vi.stubGlobal('sessionStorage', storageMock);
  vi.stubGlobal('storage', storageMock);
});

afterAll(() => {
  vi.restoreAllMocks();
});
