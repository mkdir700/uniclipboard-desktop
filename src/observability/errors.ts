import { Sentry, sentryEnabled } from '@/observability/sentry'

export class RecoverableError extends Error {
  constructor(
    message: string,
    public context: Record<string, unknown> = {}
  ) {
    super(message)
  }
}

export class CriticalError extends Error {
  constructor(
    message: string,
    public context: Record<string, unknown> = {}
  ) {
    super(message)
  }
}

export class ExpectedError extends Error {}

export function reportError(error: unknown, context?: Record<string, unknown>) {
  if (!sentryEnabled || error instanceof ExpectedError) {
    return
  }
  Sentry.captureException(error, { extra: context })
}
