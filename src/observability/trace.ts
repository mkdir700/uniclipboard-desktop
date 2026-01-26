import { Sentry, sentryEnabled } from '@/observability/sentry'

export interface TraceContext {
  traceId: string
  startTime: number
  operation: string
}

const createTraceId = (): string => {
  if (typeof crypto !== 'undefined') {
    if ('randomUUID' in crypto) {
      return crypto.randomUUID()
    }
    if ('getRandomValues' in crypto) {
      const bytes = new Uint8Array(16)
      ;(crypto as Crypto).getRandomValues(bytes)

      // Set version to 4 (0100xxxx)
      bytes[6] = (bytes[6] & 0x0f) | 0x40
      // Set variant to 10xxxxxx (RFC4122)
      bytes[8] = (bytes[8] & 0x3f) | 0x80

      const hex = (b: number) => b.toString(16).padStart(2, '0')

      return (
        hex(bytes[0]) +
        hex(bytes[1]) +
        hex(bytes[2]) +
        hex(bytes[3]) +
        '-' +
        hex(bytes[4]) +
        hex(bytes[5]) +
        '-' +
        hex(bytes[6]) +
        hex(bytes[7]) +
        '-' +
        hex(bytes[8]) +
        hex(bytes[9]) +
        '-' +
        hex(bytes[10]) +
        hex(bytes[11]) +
        hex(bytes[12]) +
        hex(bytes[13]) +
        hex(bytes[14]) +
        hex(bytes[15])
      )
    }
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
