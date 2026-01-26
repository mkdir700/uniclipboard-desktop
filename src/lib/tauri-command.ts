import { invoke } from '@tauri-apps/api/core'
import { Sentry } from '@/observability/sentry'
import { traceManager } from '@/observability/trace'

export async function invokeWithTrace<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  const trace = traceManager.startTrace(command)

  Sentry.addBreadcrumb({
    category: 'tauri_command',
    message: command,
    level: 'info',
    data: { traceId: trace.traceId, args },
  })

  try {
    return await invoke<T>(command, {
      ...args,
      _trace: {
        trace_id: trace.traceId,
        timestamp: trace.startTime,
      },
    })
  } catch (error) {
    Sentry.captureException(error, {
      tags: { command, traceId: trace.traceId },
      extra: { args },
    })
    throw error
  } finally {
    traceManager.endTrace()
  }
}
