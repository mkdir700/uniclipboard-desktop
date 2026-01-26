import { describe, expect, it, vi } from 'vitest'
import { traceManager } from '../trace'

describe('traceManager', () => {
  it('generates unique trace ids', () => {
    const first = traceManager.startTrace('test')
    traceManager.endTrace()
    const second = traceManager.startTrace('test')
    expect(first.traceId).not.toBe(second.traceId)
    traceManager.endTrace()
  })

  it('generates RFC4122 v4 trace ids when randomUUID is unavailable', () => {
    const originalCrypto = globalThis.crypto
    const bytes = new Uint8Array(16)
    // Mock bytes for deterministic UUID generation
    // These values will be modified by the v4 algorithm (version and variant bits)
    bytes.set([
      0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
      0xf0,
    ])

    vi.stubGlobal('crypto', {
      getRandomValues: (value: Uint8Array) => {
        value.set(bytes)
        return value
      },
    })

    const trace = traceManager.startTrace('test')
    // RFC4122 v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    // where y is 8, 9, a, or b
    expect(trace.traceId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i
    )

    vi.stubGlobal('crypto', originalCrypto)
    traceManager.endTrace()
  })
})
