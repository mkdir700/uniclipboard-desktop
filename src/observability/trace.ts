import { Sentry, sentryEnabled } from '@/observability/sentry'

export interface TraceContext {
  traceId: string
  startTime: number
  operation: string
}

const createTraceId = (): string => {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID()
  }

  return `${Date.now()}_${Math.random().toString(16).slice(2)}`
}

class TraceManager {
  private currentTrace: TraceContext | null = null

  startTrace(operation: string): TraceContext {
    this.currentTrace = {
      traceId: createTraceId(),
      startTime: Date.now(),
      operation,
    }
    return this.currentTrace
  }

  getCurrentTrace(): TraceContext | null {
    return this.currentTrace
  }

  endTrace(): void {
    const trace = this.currentTrace
    if (trace && sentryEnabled && Math.random() < 0.1) {
      Sentry.startSpan(
        {
          name: trace.operation,
          op: 'ui.action',
          startTime: trace.startTime / 1000,
        },
        span => {
          span.end()
        }
      )
    }
    this.currentTrace = null
  }
}

export const traceManager = new TraceManager()
