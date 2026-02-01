import { act, render } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import P2PPairingToastListener from '@/components/P2PPairingToastListener'

const onP2PPairingVerificationMock = vi.hoisted(() => vi.fn())
const toastMock = vi.hoisted(() => vi.fn())
const navigateMock = vi.hoisted(() => vi.fn())

let verificationHandler:
  | ((event: { kind: string; sessionId: string; deviceName?: string }) => void)
  | null = null

vi.mock('@/api/p2p', () => ({
  onP2PPairingVerification: onP2PPairingVerificationMock,
}))

vi.mock('@/components/ui/toast', () => ({
  toast: toastMock,
}))

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')
  return {
    ...actual,
    useNavigate: () => navigateMock,
  }
})

describe('P2PPairingToastListener', () => {
  beforeEach(() => {
    verificationHandler = null
    onP2PPairingVerificationMock.mockImplementation(async callback => {
      verificationHandler = callback
      return vi.fn()
    })
    toastMock.mockReset()
  })

  it('shows a toast for pairing request events', async () => {
    render(<P2PPairingToastListener />)

    await act(async () => {})

    expect(verificationHandler).not.toBeNull()

    act(() => {
      verificationHandler?.({
        kind: 'request',
        sessionId: 'session-1',
        deviceName: 'Peer Device',
      })
    })

    expect(toastMock).toHaveBeenCalledTimes(1)
    const options = toastMock.mock.calls[0]?.[1]
    expect(options).toEqual(
      expect.objectContaining({
        description: expect.stringContaining('Peer Device'),
        action: expect.objectContaining({
          label: expect.any(String),
          onClick: expect.any(Function),
        }),
      })
    )
    options?.action?.onClick()
    expect(navigateMock).toHaveBeenCalledWith('/devices?pairing=1')
  })
})
