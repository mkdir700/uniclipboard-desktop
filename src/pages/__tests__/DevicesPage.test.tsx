import { act, render, screen } from '@testing-library/react'
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import DevicesPage from '@/pages/DevicesPage'

const onP2PPairingVerificationMock = vi.hoisted(() => vi.fn())
const acceptP2PPairingMock = vi.hoisted(() => vi.fn())
const rejectP2PPairingMock = vi.hoisted(() => vi.fn())
const verifyP2PPairingPinMock = vi.hoisted(() => vi.fn())
const toastSuccessMock = vi.hoisted(() => vi.fn())
const toastErrorMock = vi.hoisted(() => vi.fn())
const dispatchMock = vi.hoisted(() => vi.fn())

let verificationHandler:
  | ((event: {
      kind: string
      sessionId: string
      peerId?: string
      deviceName?: string
      code?: string
    }) => void)
  | null = null

vi.mock('@/api/p2p', () => ({
  onP2PPairingVerification: onP2PPairingVerificationMock,
  acceptP2PPairing: acceptP2PPairingMock,
  rejectP2PPairing: rejectP2PPairingMock,
  verifyP2PPairingPin: verifyP2PPairingPinMock,
}))

vi.mock('@/store/hooks', () => ({
  useAppDispatch: () => dispatchMock,
}))

vi.mock('@/store/slices/devicesSlice', () => ({
  fetchPairedDevices: () => ({ type: 'devices/fetchPairedDevices' }),
}))

vi.mock('@/observability/breadcrumbs', () => ({
  captureUserIntent: vi.fn(),
}))

vi.mock('@/components', () => ({
  DeviceList: () => <div data-testid="device-list" />,
  DeviceHeader: () => <div data-testid="device-header" />,
}))

vi.mock('@/components/PairingDialog', () => ({
  default: ({ open }: { open: boolean }) => (open ? <div>PairingDialog</div> : null),
}))

vi.mock('sonner', () => ({
  toast: {
    success: toastSuccessMock,
    error: toastErrorMock,
  },
}))

describe('DevicesPage', () => {
  beforeEach(() => {
    verificationHandler = null
    onP2PPairingVerificationMock.mockImplementation(async callback => {
      verificationHandler = callback
      return vi.fn()
    })
    toastSuccessMock.mockReset()
    toastErrorMock.mockReset()
    dispatchMock.mockReset()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('shows success state for responder and dedupes completion toasts', async () => {
    render(<DevicesPage />)

    await act(async () => {})

    expect(verificationHandler).not.toBeNull()

    act(() => {
      verificationHandler?.({
        kind: 'request',
        sessionId: 'session-1',
        peerId: 'peer-1',
        deviceName: 'Peer Device',
      })
    })

    await act(async () => {})

    expect(screen.getByText('Peer Device')).toBeInTheDocument()

    act(() => {
      verificationHandler?.({
        kind: 'verification',
        sessionId: 'session-1',
        code: '123456',
        deviceName: 'Peer Device',
      })
    })

    await act(async () => {})

    expect(screen.getByText('123456')).toBeInTheDocument()

    act(() => {
      verificationHandler?.({
        kind: 'complete',
        sessionId: 'session-1',
        peerId: 'peer-1',
        deviceName: 'Peer Device',
      })
    })

    await act(async () => {})

    expect(screen.getAllByText(/配对成功|Pairing Successful/i).length).toBeGreaterThan(0)
    expect(toastSuccessMock).toHaveBeenCalledTimes(1)

    act(() => {
      verificationHandler?.({
        kind: 'complete',
        sessionId: 'session-1',
        peerId: 'peer-1',
        deviceName: 'Peer Device',
      })
    })

    expect(toastSuccessMock).toHaveBeenCalledTimes(1)

    act(() => {
      vi.advanceTimersByTime(2000)
    })

    await act(async () => {})

    expect(screen.queryAllByText(/配对成功|Pairing Successful/i).length).toBe(0)
  })
})
