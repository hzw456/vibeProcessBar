type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogData {
  timestamp: string;
  level: LogLevel;
  message: string;
  data?: object;
}

function formatLogData(level: LogLevel, message: string, data?: object): LogData {
  return {
    timestamp: new Date().toISOString(),
    level,
    message,
    data,
  };
}

export function debug(message: string, data?: object): void {
  console.debug(`[DEBUG] ${message}`, data ? JSON.stringify(formatLogData('debug', message, data)) : '');
}

export function info(message: string, data?: object): void {
  console.log(`[INFO] ${message}`, data ? JSON.stringify(formatLogData('info', message, data)) : '');
}

export function warn(message: string, data?: object): void {
  console.warn(`[WARN] ${message}`, data ? JSON.stringify(formatLogData('warn', message, data)) : '');
}

export function error(message: string, data?: object): void {
  console.error(`[ERROR] ${message}`, data ? JSON.stringify(formatLogData('error', message, data)) : '');
}
