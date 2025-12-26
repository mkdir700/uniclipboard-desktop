import {
  attachConsole,
  debug as pluginDebug,
  error as pluginError,
  info as pluginInfo,
  trace as pluginTrace,
  warn as pluginWarn,
} from "@tauri-apps/plugin-logging";

type LogMethod = (message: string) => Promise<void>;

const stringify = (value: unknown): string => {
  if (value instanceof Error) {
    return value.stack ?? value.message;
  }

  if (typeof value === "object") {
    try {
      return JSON.stringify(value);
    } catch (_) {
      return String(value);
    }
  }

  return String(value);
};

const logWith = async (method: LogMethod, parts: unknown[]): Promise<void> => {
  const message = parts.map(stringify).join(" ");
  await method(message);
};

export const logger = {
  trace: (...parts: unknown[]) => logWith(pluginTrace, parts),
  debug: (...parts: unknown[]) => logWith(pluginDebug, parts),
  info: (...parts: unknown[]) => logWith(pluginInfo, parts),
  warn: (...parts: unknown[]) => logWith(pluginWarn, parts),
  error: (...parts: unknown[]) => logWith(pluginError, parts),
};

export const initLogger = async (): Promise<void> => {
  try {
    await attachConsole();
  } catch (err) {
    await logger.error("Failed to attach console to logging plugin:", err);
  }
};
