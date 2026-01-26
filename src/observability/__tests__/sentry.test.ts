import type { ErrorEvent } from '@sentry/core'
import * as Sentry from '@sentry/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { initSentry } from '../sentry'

// Mock Sentry
vi.mock('@sentry/react', async importOriginal => {
  const actual = await importOriginal<typeof Sentry>()
  return {
    ...actual,
    init: vi.fn(),
    browserTracingIntegration: vi.fn(),
    replayIntegration: vi.fn(),
  }
})

describe('initSentry', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('initializes Sentry with correct configuration', () => {
    initSentry()

    expect(Sentry.init).toHaveBeenCalledWith(
      expect.objectContaining({
        dsn: import.meta.env.VITE_SENTRY_DSN,
        environment: import.meta.env.MODE,
        release: import.meta.env.VITE_APP_VERSION,
        beforeSend: expect.any(Function),
        beforeBreadcrumb: expect.any(Function),
      })
    )
  })

  it('scrubs sensitive data from breadcrumbs', () => {
    initSentry()
    const initCall = vi.mocked(Sentry.init).mock.calls[0][0]
    const beforeBreadcrumb = initCall.beforeBreadcrumb!

    const breadcrumb = {
      data: {
        password: 'secret',
        other: 'value',
      },
      message: 'test',
    }

    const result = beforeBreadcrumb(breadcrumb, {})

    expect(result?.data).toEqual({
      password: '[REDACTED]',
      other: 'value',
    })
  })

  it('scrubs sensitive data from event extra', async () => {
    initSentry()
    const initCall = vi.mocked(Sentry.init).mock.calls[0][0]
    const beforeSend = initCall.beforeSend!

    const event = {
      extra: {
        apiKey: '12345',
        safe: 'data',
      },
    } as unknown as ErrorEvent

    const result = await Promise.resolve(beforeSend(event, {}))

    expect(result?.extra).toEqual({
      apiKey: '[REDACTED]',
      safe: 'data',
    })
  })

  it('preserves existing ResizeObserver filter in beforeSend', async () => {
    initSentry()
    const initCall = vi.mocked(Sentry.init).mock.calls[0][0]
    const beforeSend = initCall.beforeSend!

    const resizeEvent = {
      exception: {
        values: [{ type: 'ResizeObserver loop limit exceeded' }],
      },
    } as unknown as ErrorEvent

    const result = await Promise.resolve(beforeSend(resizeEvent, {}))

    expect(result).toBeNull()
  })
})
