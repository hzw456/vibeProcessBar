import { useCallback, useEffect, useRef } from 'react';
import { useProgressStore } from '../stores/progressStore';
import { showNotification, playCompletionSound, isInDoNotDisturb } from '../utils/notifications';

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
    return on(event, handler as (data: unknown) => void);
  }, [event, handler, on]);
}

export function useProgressNotifications() {
  const { settings, addToHistory } = useProgressStore();
  const lastNotifiedProgress = useRef<Map<string, number>>(new Map());

  useEffect(() => {
    const { on } = useProgressEvent();

    const handleTaskCompleted = (data: unknown) => {
      const eventData = data as { taskId: string; duration: number };
      const { taskId, duration } = eventData;

      if (settings.doNotDisturb && isInDoNotDisturb(settings.doNotDisturbStart, settings.doNotDisturbEnd)) {
        return;
      }

      if (settings.notifications) {
        showNotification({
          title: 'Task Completed',
          body: `Completed in ${(duration / 1000).toFixed(1)}s`,
        });
      }

      if (settings.sound) {
        playCompletionSound(settings.soundVolume);
      }

      addToHistory({
        id: taskId,
        name: 'Task',
        progress: 100,
        status: 'completed',
        startTime: Date.now() - duration,
        endTime: Date.now(),
      });
    };

    const handleTaskProgress = (data: unknown) => {
      const eventData = data as { taskId: string; progress: number };
      const { taskId, progress } = eventData;
      
      const lastProgress = lastNotifiedProgress.current.get(taskId) || 0;
      
      if (progress >= settings.reminderThreshold && lastProgress < settings.reminderThreshold) {
        if (settings.doNotDisturb && isInDoNotDisturb(settings.doNotDisturbStart, settings.doNotDisturbEnd)) {
          return;
        }

        if (settings.notifications) {
          showNotification({
            title: 'Almost Done!',
            body: `${progress}% complete`,
          });
        }

        if (settings.sound) {
          playCompletionSound(settings.soundVolume * 0.5);
        }
      }

      lastNotifiedProgress.current.set(taskId, progress);
    };

    const handleTaskError = (data: unknown) => {
      const eventData = data as { error: string };
      const { error } = eventData;

      if (settings.doNotDisturb && isInDoNotDisturb(settings.doNotDisturbStart, settings.doNotDisturbEnd)) {
        return;
      }

      if (settings.notifications) {
        showNotification({
          title: 'Task Error',
          body: error,
        });
      }

      if (settings.sound) {
        playCompletionSound(settings.soundVolume * 0.3);
      }
    };

    const unsubCompleted = on('task:completed', handleTaskCompleted);
    const unsubProgress = on('task:progress', handleTaskProgress);
    const unsubError = on('task:error', handleTaskError);

    return () => {
      unsubCompleted();
      unsubProgress();
      unsubError();
    };
  }, [settings, addToHistory]);
}
