import { render, screen, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { verifyP2PPairingPin } from '@/api/p2p'
import PairingDialog from '@/components/PairingDialog'

const getP2PPeersMock = vi.hoisted(() => vi.fn())
const initiateP2PPairingMock = vi.hoisted(() => vi.fn())
const verifyP2PPairingPinMock = vi.hoisted(() => vi.fn())
const onP2PPairingVerificationMock = vi.hoisted(() => vi.fn())

let verificationHandler:
  | ((event: { kind: string; sessionId: string; code?: string }) => void)
  | null = null

vi.mock('@/api/p2p', () => ({
  getP2PPeers: getP2PPeersMock,
  initiateP2PPairing: initiateP2PPairingMock,
  verifyP2PPairingPin: verifyP2PPairingPinMock,
  onP2PPairingVerification: onP2PPairingVerificationMock,
}))

describe('PairingDialog', () => {
  beforeEach(() => {
    verificationHandler = null
    getP2PPeersMock.mockResolvedValue([])
    initiateP2PPairingMock.mockResolvedValue({ success: true, sessionId: 'session-1' })
    verifyP2PPairingPinMock.mockResolvedValue(undefined)
    onP2PPairingVerificationMock.mockImplementation(async callback => {
      verificationHandler = callback
      return vi.fn()
    })
  })

  it('shows loading state after confirming PIN match', async () => {
    const user = userEvent.setup()

    render(<PairingDialog open onClose={vi.fn()} />)

    await act(async () => {})

    expect(verificationHandler).not.toBeNull()

    act(() => {
      verificationHandler?.({
        kind: 'verification',
        sessionId: 'session-1',
        code: '123456',
      })
    })

    const confirmButton = await screen.findByRole('button', {
      name: /确认匹配|Confirm Match/i,
    })

    await user.click(confirmButton)

    expect(verifyP2PPairingPin).toHaveBeenCalledWith({
      sessionId: 'session-1',
      pinMatches: true,
    })
    expect(confirmButton).toBeDisabled()
    expect(confirmButton).toHaveTextContent(/正在验证|Verifying/i)
  })
})
