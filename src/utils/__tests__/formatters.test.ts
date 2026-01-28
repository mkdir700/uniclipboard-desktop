import { describe, expect, it } from 'vitest'
import { formatPeerId } from '@/utils/formatters'

describe('formatPeerId', () => {
  it('returns suffix with ellipsis for long peer ids', () => {
    const peerId = '12D3KooWabcdef1234567890'

    expect(formatPeerId(peerId)).toBe('34567890')
  })

  it('returns original value for short peer ids', () => {
    const peerId = 'peer-1'

    expect(formatPeerId(peerId)).toBe(peerId)
  })
})
