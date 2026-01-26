import type { ErrorEvent, EventHint } from '@sentry/core'
import * as Sentry from '@sentry/react'
import { redactSensitiveArgs } from '@/observability/redaction'

const sentryEnabled = Boolean(import.meta.env.VITE_SENTRY_DSN)

const getTauriPlatform = (): string => {
  if (typeof window === 'undefined' || !('__TAURI__' in window)) {
    return 'unknown'
  }

  const tauriWindow = window as typeof window & {
    __TAURI__?: { platform?: string }
  }

  return tauriWindow.__TAURI__?.platform ?? 'unknown'
}

export function initSentry(): void {
  if (!sentryEnabled) {
    return
  }

  const beforeSend: (event: ErrorEvent, hint: EventHint) => ErrorEvent | null = (event, _hint) => {
    const type = event.exception?.values?.[0]?.type
    if (type === 'ResizeObserver loop limit exceeded') {
      return null
    }
    if (event.extra) {
      event.extra = redactSensitiveArgs(event.extra) as Record<string, unknown>
    }
    return event
  }

  const beforeBreadcrumb = (breadcrumb: Sentry.Breadcrumb): Sentry.Breadcrumb | null => {
    if (breadcrumb.data) {
      breadcrumb.data = redactSensitiveArgs(breadcrumb.data) as Record<string, unknown>
    }
    return breadcrumb
  }

  Sentry.init({
    dsn: import.meta.env.VITE_SENTRY_DSN,
    tracesSampleRate: 0.1,
    replaysSessionSampleRate: 0.1,
    replaysOnErrorSampleRate: 1.0,
    environment: import.meta.env.MODE,
    release: import.meta.env.VITE_APP_VERSION,
    integrations: [Sentry.browserTracingIntegration(), Sentry.replayIntegration()],
    beforeSend,
    beforeBreadcrumb,
    initialScope: {
      tags: {
        platform: getTauriPlatform(),
      },
    },
  })
}

export { Sentry, sentryEnabled }
