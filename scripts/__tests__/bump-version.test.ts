import { describe, expect, it } from 'vitest'
import { bumpVersion } from '../bump-version-lib.js'

describe('bumpVersion prerelease patch', () => {
  it('keeps base version when creating first alpha prerelease', () => {
    expect(bumpVersion('0.1.0', 'patch', 'alpha')).toBe('0.1.0-alpha.1')
  })

  it('increments alpha prerelease number on repeat', () => {
    expect(bumpVersion('0.1.0-alpha.1', 'patch', 'alpha')).toBe('0.1.0-alpha.2')
  })
})
