export interface NotificationOptions {
  title: string;
  body: string;
  icon?: string;
  silent?: boolean;
}

export async function showNotification(options: NotificationOptions): Promise<boolean> {
  if (!('Notification' in window)) {
    console.log('Notifications not supported');
    return false;
  }

  if (Notification.permission === 'granted') {
    new Notification(options.title, {
      body: options.body,
      icon: options.icon || '/icon.png',
      silent: options.silent ?? false,
    });
    return true;
  }

  if (Notification.permission !== 'denied') {
    const permission = await Notification.requestPermission();
    if (permission === 'granted') {
      new Notification(options.title, {
        body: options.body,
        icon: options.icon || '/icon.png',
        silent: options.silent ?? false,
      });
      return true;
    }
  }

  return false;
}

export async function checkNotificationPermission(): Promise<NotificationPermission> {
  if (!('Notification' in window)) {
    return 'denied';
  }
  return Notification.permission;
}

export function playSound(volume: number = 0.7): void {
  // 使用与 playCompletionSound 相同的音效
  playCompletionSound(volume);
}

export function playCompletionSound(volume: number = 0.7): void {
  try {
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
    
    // 简单清晰的两个音符提示音
    const notes = [
      { freq: 800, duration: 0.15 },  // 高音
      { freq: 1200, duration: 0.15 }  // 更高音
    ];
    
    notes.forEach((note, index) => {
      const oscillator = audioContext.createOscillator();
      const gainNode = audioContext.createGain();
      
      oscillator.connect(gainNode);
      gainNode.connect(audioContext.destination);
      
      oscillator.type = 'sine';
      oscillator.frequency.setValueAtTime(note.freq, audioContext.currentTime + index * 0.2);
      
      const startTime = audioContext.currentTime + index * 0.2;
      // 快速淡入淡出
      gainNode.gain.setValueAtTime(0, startTime);
      gainNode.gain.linearRampToValueAtTime(volume * 0.3, startTime + 0.01);
      gainNode.gain.linearRampToValueAtTime(0, startTime + note.duration);
      
      oscillator.start(startTime);
      oscillator.stop(startTime + note.duration);
    });
  } catch (error) {
    console.error('Failed to play completion sound:', error);
  }
}

export function isInDoNotDisturb(start: string, end: string): boolean {
  const now = new Date();
  const currentTime = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
  
  if (start <= end) {
    return currentTime >= start && currentTime <= end;
  } else {
    return currentTime >= start || currentTime <= end;
  }
}
