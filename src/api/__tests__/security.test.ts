import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getEncryptionSessionStatus, unlockEncryptionSession } from '@/api/security'
import { invokeWithTrace } from '@/lib/tauri-command'

vi.mock('@/lib/tauri-command', () => ({
  invokeWithTrace: vi.fn(),
}))

const invokeMock = vi.mocked(invokeWithTrace)

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
