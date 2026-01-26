import { describe, expect, it } from 'vitest'
import { traceManager } from '../trace'

describe('traceManager', () => {
  it('generates unique trace ids', () => {
    const first = traceManager.startTrace('test')
    traceManager.endTrace()
    const second = traceManager.startTrace('test')
    expect(first.traceId).not.toBe(second.traceId)
    traceManager.endTrace()
  })
})
