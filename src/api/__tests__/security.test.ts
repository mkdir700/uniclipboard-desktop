import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getEncryptionSessionStatus, unlockEncryptionSession } from '@/api/security'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

describe('security api', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('gets encryption session status', async () => {
    invokeMock.mockResolvedValueOnce({ initialized: true, session_ready: false })

    const result = await getEncryptionSessionStatus()

    expect(invokeMock).toHaveBeenCalledWith('get_encryption_session_status')
    expect(result).toEqual({ initialized: true, session_ready: false })
  })

  it('unlocks encryption session', async () => {
    invokeMock.mockResolvedValueOnce(true)

    const result = await unlockEncryptionSession()

    expect(invokeMock).toHaveBeenCalledWith('unlock_encryption_session')
    expect(result).toBe(true)
  })
})
