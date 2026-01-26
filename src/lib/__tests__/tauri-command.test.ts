import { invoke } from '@tauri-apps/api/core'
import { describe, expect, it, vi } from 'vitest'
import { invokeWithTrace } from '@/lib/tauri-command'
import { redactSensitiveArgs } from '@/observability/redaction'
import { Sentry } from '@/observability/sentry'
import { traceManager } from '@/observability/trace'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@/observability/trace', () => ({
  traceManager: {
    startTrace: vi.fn(),
    endTrace: vi.fn(),
  },
}))

vi.mock('@/observability/sentry', () => ({
  Sentry: {
    addBreadcrumb: vi.fn(),
    captureException: vi.fn(),
  },
}))

describe('invokeWithTrace', () => {
  it('invokes command with trace metadata and args', async () => {
    const trace = { traceId: 'trace-1', startTime: 1234, operation: 'command' }
    const args = { limit: 1, token: 'secret' }
    const safeArgs = redactSensitiveArgs(args)

    vi.mocked(traceManager.startTrace).mockReturnValue(trace)
    vi.mocked(invoke).mockResolvedValueOnce({ ok: true })

    await invokeWithTrace('get_clipboard_entries', args)

    expect(traceManager.startTrace).toHaveBeenCalledWith('get_clipboard_entries')
    expect(Sentry.addBreadcrumb).toHaveBeenCalledWith({
      category: 'tauri_command',
      message: 'get_clipboard_entries',
      level: 'info',
      data: { traceId: trace.traceId, args: safeArgs },
    })
    expect(invoke).toHaveBeenCalledWith('get_clipboard_entries', {
      ...args,
      _trace: {
        trace_id: trace.traceId,
        timestamp: trace.startTime,
      },
    })
    expect(traceManager.endTrace).toHaveBeenCalled()
  })

  it('redacts sensitive args for sentry breadcrumbs and errors', async () => {
    const trace = { traceId: 'trace-2', startTime: 5678, operation: 'command' }
    const args = { password: 'secret', nested: { token: 'value' } }
    const safeArgs = redactSensitiveArgs(args)
    const error = new Error('boom')

    vi.mocked(traceManager.startTrace).mockReturnValue(trace)
    vi.mocked(invoke).mockRejectedValueOnce(error)

    await expect(invokeWithTrace('set_clipboard', args)).rejects.toThrow('boom')

    expect(Sentry.addBreadcrumb).toHaveBeenCalledWith({
      category: 'tauri_command',
      message: 'set_clipboard',
      level: 'info',
      data: { traceId: trace.traceId, args: safeArgs },
    })
    expect(Sentry.captureException).toHaveBeenCalledWith(error, {
      tags: { command: 'set_clipboard', traceId: trace.traceId },
      extra: { args: safeArgs },
    })
    expect(invoke).toHaveBeenCalledWith('set_clipboard', {
      ...args,
      _trace: {
        trace_id: trace.traceId,
        timestamp: trace.startTime,
      },
    })
    expect(traceManager.endTrace).toHaveBeenCalled()
  })
})
