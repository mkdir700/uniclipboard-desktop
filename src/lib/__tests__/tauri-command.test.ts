import { invoke } from '@tauri-apps/api/core'
import { describe, expect, it, vi } from 'vitest'
import { invokeWithTrace } from '@/lib/tauri-command'
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
    const args = { limit: 1 }

    vi.mocked(traceManager.startTrace).mockReturnValue(trace)
    vi.mocked(invoke).mockResolvedValueOnce({ ok: true })

    await invokeWithTrace('get_clipboard_entries', args)

    expect(traceManager.startTrace).toHaveBeenCalledWith('get_clipboard_entries')
    expect(Sentry.addBreadcrumb).toHaveBeenCalledWith({
      category: 'tauri_command',
      message: 'get_clipboard_entries',
      level: 'info',
      data: { traceId: trace.traceId, args },
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
})
