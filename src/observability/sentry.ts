import type { ErrorEvent, EventHint } from '@sentry/core'
import * as Sentry from '@sentry/react'

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
    return event
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
    initialScope: {
      tags: {
        platform: getTauriPlatform(),
      },
    },
  })
}

export { Sentry, sentryEnabled }
