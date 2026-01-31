import { beforeEach, describe, expect, it, vi } from 'vitest'
import { dispatchSetupEvent } from '@/api/onboarding'
import { invokeWithTrace } from '@/lib/tauri-command'

vi.mock('@/lib/tauri-command', () => ({
  invokeWithTrace: vi.fn(),
}))

describe('onboarding api', () => {
  const invokeWithTraceMock = vi.mocked(invokeWithTrace)

  beforeEach(() => {
    invokeWithTraceMock.mockReset()
    invokeWithTraceMock.mockResolvedValue(undefined)
  })

  it('dispatchSetupEvent calls new tauri command', async () => {
    const event = {
      SubmitCreatePassphrase: {
        pass1: 'a',
        pass2: 'b',
      },
    }

    await dispatchSetupEvent(event)

    expect(invokeWithTraceMock).toHaveBeenCalledWith('dispatch_setup_event', { event })
  })
})
