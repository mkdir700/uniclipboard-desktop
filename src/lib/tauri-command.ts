import { invoke } from '@tauri-apps/api/core'
import { redactSensitiveArgs } from '@/observability/redaction'
import { Sentry } from '@/observability/sentry'
import { traceManager } from '@/observability/trace'

export async function invokeWithTrace<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  const trace = traceManager.startTrace(command)
  const safeArgs = redactSensitiveArgs(args)

  Sentry.addBreadcrumb({
    category: 'tauri_command',
    message: command,
    level: 'info',
    data: { traceId: trace.traceId, args: safeArgs },
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
      extra: { args: safeArgs },
    })
    throw error
  } finally {
    traceManager.endTrace()
  }
}
