import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getSetupState, selectJoinPeer, submitPassphrase } from '@/api/setup'
import { invokeWithTrace } from '@/lib/tauri-command'

vi.mock('@/lib/tauri-command', () => ({
  invokeWithTrace: vi.fn(),
}))

describe('setup api', () => {
  const invokeWithTraceMock = vi.mocked(invokeWithTrace)

  beforeEach(() => {
    invokeWithTraceMock.mockReset()
    invokeWithTraceMock.mockResolvedValue(undefined)
  })

  it('submitPassphrase calls tauri command', async () => {
    await submitPassphrase('a', 'b')

    expect(invokeWithTraceMock).toHaveBeenCalledWith('submit_passphrase', {
      passphrase1: 'a',
      passphrase2: 'b',
    })
  })

  it('selectJoinPeer calls tauri command', async () => {
    await selectJoinPeer('peer-1')

    expect(invokeWithTraceMock).toHaveBeenCalledWith('select_device', {
      peerId: 'peer-1',
    })
  })

  it('getSetupState parses json string response', async () => {
    invokeWithTraceMock.mockResolvedValue(
      JSON.stringify({ CreateSpaceInputPassphrase: { error: null } })
    )

    await expect(getSetupState()).resolves.toEqual({
      CreateSpaceInputPassphrase: { error: null },
    })
  })
})
