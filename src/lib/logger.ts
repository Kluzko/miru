import {
  trace as tauriTrace,
  debug as tauriDebug,
  info as tauriInfo,
  warn as tauriWarn,
  error as tauriError,
} from "@tauri-apps/plugin-log";

/**
 * Logger levels for type safety
 */
export enum LogLevel {
  TRACE = 0,
  DEBUG = 1,
  INFO = 2,
  WARN = 3,
  ERROR = 4,
  FATAL = 5,
}

/**
 * Logger configuration interface
 */
interface LoggerConfig {
  level: LogLevel;
  enableColors: boolean;
  enableTimestamp: boolean;
  isDevelopment: boolean;
}

/**
 * Base Logger class - Singleton pattern
 * Browser-native logger optimized for Tauri desktop applications
 */
export class Logger {
  private static instance: Logger;
  private config: LoggerConfig;

  private constructor(config?: Partial<LoggerConfig>) {
    this.config = {
      level: import.meta.env.DEV ? LogLevel.DEBUG : LogLevel.INFO,
      enableColors: true,
      enableTimestamp: true,
      isDevelopment: import.meta.env.DEV,
      ...config,
    };
  }

  /**
   * Get singleton instance
   */
  public static getInstance(config?: Partial<LoggerConfig>): Logger {
    if (!Logger.instance) {
      Logger.instance = new Logger(config);
    }
    return Logger.instance;
  }

  /**
   * Create a child logger with context (Factory pattern)
   */
  public createChild(context: string): FeatureLogger {
    return new FeatureLogger(context, this.config);
  }

  /**
   * Check if log level should be output
   */
  private shouldLog(level: LogLevel): boolean {
    return level >= this.config.level;
  }

  /**
   * Get styled prefix for console
   */
  private getStyledPrefix(level: LogLevel): [string, string] {
    const styles = {
      [LogLevel.TRACE]: "color: #6B7280; font-weight: bold", // gray
      [LogLevel.DEBUG]: "color: #3B82F6; font-weight: bold", // blue
      [LogLevel.INFO]: "color: #10B981; font-weight: bold", // green
      [LogLevel.WARN]: "color: #F59E0B; font-weight: bold", // orange
      [LogLevel.ERROR]: "color: #EF4444; font-weight: bold", // red
      [LogLevel.FATAL]: "color: #DC2626; font-weight: bold", // dark red
    };

    const levelNames = {
      [LogLevel.TRACE]: "TRACE",
      [LogLevel.DEBUG]: "DEBUG",
      [LogLevel.INFO]: "INFO",
      [LogLevel.WARN]: "WARN",
      [LogLevel.ERROR]: "ERROR",
      [LogLevel.FATAL]: "FATAL",
    };

    const style = styles[level];
    const levelName = levelNames[level];
    const timestamp = this.config.enableTimestamp
      ? new Date().toISOString().split("T")[1].slice(0, 12)
      : "";

    const prefix = timestamp
      ? `%c[MIRU:FRONTEND] [${timestamp}] [${levelName}]`
      : `%c[MIRU:FRONTEND] [${levelName}]`;

    return [prefix, style];
  }

  /**
   * Core logging methods
   */
  public trace(message: string, data?: Record<string, any>): void {
    if (!this.shouldLog(LogLevel.TRACE)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.TRACE);
    const fullMessage =
      data && Object.keys(data).length > 0
        ? `${message} ${JSON.stringify(data)}`
        : message;

    // Log to browser console with colors
    if (data && Object.keys(data).length > 0) {
      console.log(prefix, style, message, data);
    } else {
      console.log(prefix, style, message);
    }

    // Forward to Tauri backend for terminal output
    tauriTrace(`[FRONTEND] ${fullMessage}`);
  }

  public debug(message: string, data?: Record<string, any>): void {
    if (!this.shouldLog(LogLevel.DEBUG)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.DEBUG);
    const fullMessage =
      data && Object.keys(data).length > 0
        ? `${message} ${JSON.stringify(data)}`
        : message;

    // Log to browser console with colors
    if (data && Object.keys(data).length > 0) {
      console.log(prefix, style, message, data);
    } else {
      console.log(prefix, style, message);
    }

    // Forward to Tauri backend for terminal output
    tauriDebug(`[FRONTEND] ${fullMessage}`);
  }

  public info(message: string, data?: Record<string, any>): void {
    if (!this.shouldLog(LogLevel.INFO)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.INFO);
    const fullMessage =
      data && Object.keys(data).length > 0
        ? `${message} ${JSON.stringify(data)}`
        : message;

    // Log to browser console with colors
    if (data && Object.keys(data).length > 0) {
      console.log(prefix, style, message, data);
    } else {
      console.log(prefix, style, message);
    }

    // Forward to Tauri backend for terminal output
    tauriInfo(`[FRONTEND] ${fullMessage}`);
  }

  public warn(message: string, data?: Record<string, any>): void {
    if (!this.shouldLog(LogLevel.WARN)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.WARN);
    const fullMessage =
      data && Object.keys(data).length > 0
        ? `${message} ${JSON.stringify(data)}`
        : message;

    // Log to browser console with colors
    if (data && Object.keys(data).length > 0) {
      console.warn(prefix, style, message, data);
    } else {
      console.warn(prefix, style, message);
    }

    // Forward to Tauri backend for terminal output
    tauriWarn(`[FRONTEND] ${fullMessage}`);
  }

  public error(
    message: string,
    errorObj?: Error | unknown,
    data?: Record<string, any>,
  ): void {
    if (!this.shouldLog(LogLevel.ERROR)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.ERROR);
    const errorData = this.serializeError(errorObj);
    const allData = { ...errorData, ...data };
    const fullMessage =
      Object.keys(allData).length > 0
        ? `${message} ${JSON.stringify(allData)}`
        : message;

    // Log to browser console with colors
    if (Object.keys(allData).length > 0) {
      console.error(prefix, style, message, allData);
    } else {
      console.error(prefix, style, message);
    }

    // Forward to Tauri backend for terminal output
    tauriError(`[FRONTEND] ${fullMessage}`);
  }

  public fatal(
    message: string,
    errorObj?: Error | unknown,
    data?: Record<string, any>,
  ): void {
    if (!this.shouldLog(LogLevel.FATAL)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.FATAL);
    const errorData = this.serializeError(errorObj);
    const allData = { ...errorData, ...data };
    const fullMessage =
      Object.keys(allData).length > 0
        ? `${message} ${JSON.stringify(allData)}`
        : message;

    // Log to browser console with colors
    if (Object.keys(allData).length > 0) {
      console.error(prefix, style, message, allData);
    } else {
      console.error(prefix, style, message);
    }

    // Forward to Tauri backend for terminal output
    tauriError(`[FRONTEND] [FATAL] ${fullMessage}`);
  }

  /**
   * Serialize error object for logging
   */
  private serializeError(error?: Error | unknown): Record<string, any> {
    if (!error) return {};
    if (error instanceof Error) {
      return {
        errorMessage: error.message,
        errorName: error.name,
        errorStack: error.stack,
      };
    }
    return {
      error: String(error),
    };
  }

  /**
   * Group logging for console (development only)
   */
  public group(title: string, callback: () => void): void {
    if (this.config.isDevelopment) {
      console.group(title);
      try {
        callback();
      } finally {
        console.groupEnd();
      }
    } else {
      callback();
    }
  }
}

/**
 * Feature-specific logger with context (Strategy pattern)
 */
export class FeatureLogger {
  private context: string;
  private config: LoggerConfig;

  constructor(context: string, config: LoggerConfig) {
    this.context = context;
    this.config = config;
  }

  /**
   * Check if log level should be output
   */
  private shouldLog(level: LogLevel): boolean {
    return level >= this.config.level;
  }

  /**
   * Get styled prefix for console with feature context
   */
  private getStyledPrefix(level: LogLevel): [string, string] {
    const styles = {
      [LogLevel.TRACE]: "color: #6B7280; font-weight: bold",
      [LogLevel.DEBUG]: "color: #3B82F6; font-weight: bold",
      [LogLevel.INFO]: "color: #10B981; font-weight: bold",
      [LogLevel.WARN]: "color: #F59E0B; font-weight: bold",
      [LogLevel.ERROR]: "color: #EF4444; font-weight: bold",
      [LogLevel.FATAL]: "color: #DC2626; font-weight: bold",
    };

    const levelNames = {
      [LogLevel.TRACE]: "TRACE",
      [LogLevel.DEBUG]: "DEBUG",
      [LogLevel.INFO]: "INFO",
      [LogLevel.WARN]: "WARN",
      [LogLevel.ERROR]: "ERROR",
      [LogLevel.FATAL]: "FATAL",
    };

    const style = styles[level];
    const levelName = levelNames[level];
    const timestamp = this.config.enableTimestamp
      ? new Date().toISOString().split("T")[1].slice(0, 12)
      : "";

    const prefix = timestamp
      ? `%c[MIRU:FRONTEND] [${timestamp}] [${levelName}] [${this.context}]`
      : `%c[MIRU:FRONTEND] [${levelName}] [${this.context}]`;

    return [prefix, style];
  }

  /**
   * Add emoji icons based on log type
   */
  private getIcon(type: string): string {
    const icons: Record<string, string> = {
      success: "✅",
      error: "❌",
      warning: "⚠️",
      info: "ℹ️",
      debug: "🔍",
      start: "🚀",
      end: "🏁",
      user: "👆",
      cache: "💾",
      api: "🌐",
      mutation: "🔄",
    };
    return icons[type] || "📝";
  }

  public trace(message: string, data?: Record<string, any>): void {
    if (!this.shouldLog(LogLevel.TRACE)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.TRACE);
    const fullMessage =
      data && Object.keys(data).length > 0
        ? `[${this.context}] ${message} ${JSON.stringify(data)}`
        : `[${this.context}] ${message}`;

    // Log to browser console with colors
    if (data && Object.keys(data).length > 0) {
      console.log(prefix, style, message, data);
    } else {
      console.log(prefix, style, message);
    }

    // Forward to Tauri backend for terminal output
    tauriTrace(`[FRONTEND] ${fullMessage}`);
  }

  public debug(message: string, data?: Record<string, any>): void {
    if (!this.shouldLog(LogLevel.DEBUG)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.DEBUG);
    const msg = `${this.getIcon("debug")} ${message}`;
    const fullMessage =
      data && Object.keys(data).length > 0
        ? `[${this.context}] ${msg} ${JSON.stringify(data)}`
        : `[${this.context}] ${msg}`;

    // Log to browser console with colors
    if (data && Object.keys(data).length > 0) {
      console.log(prefix, style, msg, data);
    } else {
      console.log(prefix, style, msg);
    }

    // Forward to Tauri backend for terminal output
    tauriDebug(`[FRONTEND] ${fullMessage}`);
  }

  public info(message: string, data?: Record<string, any>): void {
    if (!this.shouldLog(LogLevel.INFO)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.INFO);
    const msg = `${this.getIcon("info")} ${message}`;
    const fullMessage =
      data && Object.keys(data).length > 0
        ? `[${this.context}] ${msg} ${JSON.stringify(data)}`
        : `[${this.context}] ${msg}`;

    // Log to browser console with colors
    if (data && Object.keys(data).length > 0) {
      console.log(prefix, style, msg, data);
    } else {
      console.log(prefix, style, msg);
    }

    // Forward to Tauri backend for terminal output
    tauriInfo(`[FRONTEND] ${fullMessage}`);
  }

  public warn(message: string, data?: Record<string, any>): void {
    if (!this.shouldLog(LogLevel.WARN)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.WARN);
    const msg = `${this.getIcon("warning")} ${message}`;
    const fullMessage =
      data && Object.keys(data).length > 0
        ? `[${this.context}] ${msg} ${JSON.stringify(data)}`
        : `[${this.context}] ${msg}`;

    // Log to browser console with colors
    if (data && Object.keys(data).length > 0) {
      console.warn(prefix, style, msg, data);
    } else {
      console.warn(prefix, style, msg);
    }

    // Forward to Tauri backend for terminal output
    tauriWarn(`[FRONTEND] ${fullMessage}`);
  }

  public error(
    message: string,
    errorObj?: Error | unknown,
    data?: Record<string, any>,
  ): void {
    if (!this.shouldLog(LogLevel.ERROR)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.ERROR);
    const msg = `${this.getIcon("error")} ${message}`;
    const errorData = this.serializeError(errorObj);
    const allData = { ...errorData, ...data };
    const fullMessage =
      Object.keys(allData).length > 0
        ? `[${this.context}] ${msg} ${JSON.stringify(allData)}`
        : `[${this.context}] ${msg}`;

    // Log to browser console with colors
    if (Object.keys(allData).length > 0) {
      console.error(prefix, style, msg, allData);
    } else {
      console.error(prefix, style, msg);
    }

    // Forward to Tauri backend for terminal output
    tauriError(`[FRONTEND] ${fullMessage}`);
  }

  /**
   * Specialized logging methods
   */
  public success(
    message: string,
    data?: Record<string, any>,
    startTime?: number,
  ): void {
    if (!this.shouldLog(LogLevel.INFO)) return;
    const logData = { ...data };
    if (startTime) {
      logData.duration = `${(Date.now() - startTime).toFixed(2)}ms`;
    }
    const [prefix, style] = this.getStyledPrefix(LogLevel.INFO);
    const msg = `${this.getIcon("success")} ${message}`;
    const fullMessage =
      Object.keys(logData).length > 0
        ? `[${this.context}] ${msg} ${JSON.stringify(logData)}`
        : `[${this.context}] ${msg}`;

    // Log to browser console with colors
    if (Object.keys(logData).length > 0) {
      console.log(prefix, style, msg, logData);
    } else {
      console.log(prefix, style, msg);
    }

    // Forward to Tauri backend for terminal output
    tauriInfo(`[FRONTEND] ${fullMessage}`);
  }

  public userAction(action: string, data?: Record<string, any>): void {
    if (!this.shouldLog(LogLevel.INFO)) return;
    const [prefix, style] = this.getStyledPrefix(LogLevel.INFO);
    const msg = `${this.getIcon("user")} User action: ${action}`;
    if (data && Object.keys(data).length > 0) {
      console.log(prefix, style, msg, data);
    } else {
      console.log(prefix, style, msg);
    }
  }

  public mutation(
    stage: "start" | "success" | "error" | "rollback",
    operation: string,
    data?: Record<string, any>,
  ): void {
    const icons = {
      start: "🚀",
      success: "✅",
      error: "❌",
      rollback: "↩️",
    };
    const message = `${icons[stage]} ${operation} - ${stage}`;
    switch (stage) {
      case "start":
        this.debug(message, data);
        break;
      case "success":
        this.info(message, data);
        break;
      case "error":
      case "rollback":
        this.error(message, undefined, data);
        break;
    }
  }

  public apiCall(
    method: string,
    endpoint: string,
    status: "start" | "success" | "error",
    data?: Record<string, any>,
  ): void {
    const message = `${this.getIcon("api")} ${method} ${endpoint}`;
    switch (status) {
      case "start":
        this.debug(message, data);
        break;
      case "success":
        this.info(message, data);
        break;
      case "error":
        this.error(message, undefined, data);
        break;
    }
  }

  public cache(
    action: "hit" | "miss" | "set" | "invalidate",
    key: string,
    data?: Record<string, any>,
  ): void {
    const icons = {
      hit: "✅",
      miss: "❌",
      set: "💾",
      invalidate: "🗑️",
    };
    this.debug(`${icons[action]} Cache ${action}: ${key}`, data);
  }

  public importProgress(
    current: number,
    total: number,
    currentItem?: string,
  ): void {
    const percentage = ((current / total) * 100).toFixed(1);
    this.info(`📦 Import progress: ${current}/${total} (${percentage}%)`, {
      current,
      total,
      percentage,
      currentItem,
    });
  }

  /**
   * Timed operation helper (Builder pattern)
   */
  public startTimed(operation: string): TimedOperation {
    return new TimedOperation(this, operation);
  }

  /**
   * Group logging
   */
  public group(title: string, callback: () => void): void {
    if (this.config.isDevelopment) {
      console.group(title);
      try {
        callback();
      } finally {
        console.groupEnd();
      }
    } else {
      callback();
    }
  }

  private serializeError(error?: Error | unknown): Record<string, any> {
    if (!error) return {};
    if (error instanceof Error) {
      return {
        errorMessage: error.message,
        errorName: error.name,
        errorStack: error.stack,
      };
    }
    return {
      error: String(error),
    };
  }
}

/**
 * Timed operation helper class (Builder pattern)
 */
export class TimedOperation {
  private logger: FeatureLogger;
  private operation: string;
  private startTime: number;

  constructor(logger: FeatureLogger, operation: string) {
    this.logger = logger;
    this.operation = operation;
    this.startTime = Date.now();
  }

  public success(data?: Record<string, any>): void {
    this.logger.success(this.operation, data, this.startTime);
  }

  public error(error: Error | unknown, data?: Record<string, any>): void {
    const duration = Date.now() - this.startTime;
    this.logger.error(this.operation, error, {
      ...data,
      duration: `${duration.toFixed(2)}ms`,
    });
  }
}

/**
 * Global logger instance
 */
const globalLogger = Logger.getInstance();

/**
 * Feature-specific loggers (exported for convenience)
 */
export const collectionLogger = globalLogger.createChild("collection");
export const animeLogger = globalLogger.createChild("anime");
export const cacheLogger = globalLogger.createChild("cache");
export const apiLogger = globalLogger.createChild("api");
export const uiLogger = globalLogger.createChild("ui");
export const importLogger = globalLogger.createChild("import");
export const errorLogger = globalLogger.createChild("error");

/**
 * Export main logger instance
 */
export default globalLogger;
