import { useCallback, useEffect, useRef } from 'react';

export function useProgressEvent() {
  const eventHandlers = useRef<Map<string, Set<(data: unknown) => void>>>(new Map());

  const on = useCallback((event: string, handler: (data: unknown) => void) => {
    if (!eventHandlers.current.has(event)) {
      eventHandlers.current.set(event, new Set());
    }
    eventHandlers.current.get(event)!.add(handler);
    return () => {
      eventHandlers.current.get(event)?.delete(handler);
    };
  }, []);

  const emit = useCallback((event: string, data: unknown) => {
    eventHandlers.current.get(event)?.forEach(handler => handler(data));
  }, []);

  useEffect(() => {
    return () => {
      eventHandlers.current.clear();
    };
  }, []);

  return { on, emit };
}

export type ProgressEventType = 
  | 'task:started'
  | 'task:progress'
  | 'task:completed'
  | 'task:error'
  | 'task:reset'
  | 'window:toggle'
  | 'settings:changed';

export interface ProgressEventData {
  'task:started': { taskId: string; taskName: string; adapter?: string };
  'task:progress': { taskId: string; progress: number };
  'task:completed': { taskId: string; duration: number };
  'task:error': { taskId: string; error: string };
  'task:reset': { taskId: string };
  'window:toggle': { visible: boolean };
  'settings:changed': { key: string; value: unknown };
}

export function useProgressEvents<T extends ProgressEventType>(
  event: T,
  handler: (data: ProgressEventData[T]) => void
) {
  const { on } = useProgressEvent();
  
  useEffect(() => {
    return on(event, handler);
  }, [event, handler, on]);
}
