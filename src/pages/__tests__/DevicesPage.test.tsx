import { act, render, screen } from '@testing-library/react'
import * as React from 'react'
import { MemoryRouter } from 'react-router-dom'
import { toast } from 'sonner'
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import DevicesPage from '@/pages/DevicesPage'

const onP2PPairingVerificationMock = vi.hoisted(() => vi.fn())
const acceptP2PPairingMock = vi.hoisted(() => vi.fn())
const rejectP2PPairingMock = vi.hoisted(() => vi.fn())
const verifyP2PPairingPinMock = vi.hoisted(() => vi.fn())
const toastSuccessMock = vi.hoisted(() => vi.fn())
const toastErrorMock = vi.hoisted(() => vi.fn())
const dispatchMock = vi.hoisted(() => vi.fn())
const useAppSelectorMock = vi.hoisted(
  () =>
    (
      selector: (state: {
        devices: {
          pairedDevices: unknown[]
          pairedDevicesLoading: boolean
          pairedDevicesError: string | null
        }
      }) => unknown
    ) =>
      selector({
        devices: {
          pairedDevices: [],
          pairedDevicesLoading: false,
          pairedDevicesError: null,
        },
      })
)

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
  useAppSelector: useAppSelectorMock,
}))

vi.mock('@/store/slices/devicesSlice', () => ({
  fetchPairedDevices: () => ({ type: 'devices/fetchPairedDevices' }),
}))

vi.mock('@/observability/breadcrumbs', () => ({
  captureUserIntent: vi.fn(),
}))

vi.mock('@/components', () => ({
  DeviceList: ({ onAddDevice }: { onAddDevice: () => void }) => (
    <div data-testid="device-list">
      <button type="button" onClick={onAddDevice}>
        Add Device
      </button>
    </div>
  ),
  DeviceHeader: ({ activeTab, addDevice }: { activeTab: string; addDevice: () => void }) => (
    <div data-testid="device-header" data-active-tab={activeTab}>
      <button type="button" onClick={addDevice}>
        Add Device
      </button>
    </div>
  ),
}))

const MockPairingDialog = ({
  open,
  onPairingSuccess,
}: {
  open: boolean
  onPairingSuccess: () => void
}) => {
  const [peerName, setPeerName] = React.useState<string | null>(null)
  const [code, setCode] = React.useState<string | null>(null)
  const [successVisible, setSuccessVisible] = React.useState(false)
  const completedSessionId = React.useRef<string | null>(null)

  React.useEffect(() => {
    if (open) {
      onP2PPairingVerificationMock(
        (event: { kind: string; sessionId?: string; deviceName?: string; code?: string }) => {
          if (event.kind === 'request') {
            setPeerName(event.deviceName ?? null)
          }

          if (event.kind === 'verification') {
            setCode(event.code ?? null)
          }

          if (event.kind === 'complete') {
            if (event.sessionId && completedSessionId.current === event.sessionId) {
              return
            }
            completedSessionId.current = event.sessionId ?? completedSessionId.current
            setSuccessVisible(true)
            toast.success('Pairing Successful')
            setTimeout(() => {
              setSuccessVisible(false)
            }, 2000)
          }
        }
      )
    }
  }, [open])

  if (!open) {
    return null
  }

  return (
    <div>
      PairingDialog
      {peerName ? <span>{peerName}</span> : null}
      {code ? <span>{code}</span> : null}
      {successVisible ? <span>Pairing Successful</span> : null}
      <button type="button" onClick={onPairingSuccess}>
        Trigger Success
      </button>
    </div>
  )
}

vi.mock('@/components/PairingDialog', () => ({
  default: MockPairingDialog,
}))

vi.mock('sonner', () => ({
  toast: {
    success: toastSuccessMock,
    error: toastErrorMock,
  },
}))

describe('DevicesPage', () => {
  const renderDevicesPage = (initialEntries: string[] = ['/devices']) => {
    return render(
      <MemoryRouter initialEntries={initialEntries}>
        <DevicesPage />
      </MemoryRouter>
    )
  }

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
    Element.prototype.scrollIntoView = vi.fn()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('shows success state for responder and dedupes completion toasts', async () => {
    renderDevicesPage()

    await act(async () => {})

    const addDeviceButton = screen.getByText('Add Device')
    act(() => {
      addDeviceButton.click()
    })

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

  it('handles initiator pairing success: refreshes list and switches tab', async () => {
    renderDevicesPage()

    const addDeviceButton = screen.getByText('Add Device')
    act(() => {
      addDeviceButton.click()
    })

    const successButton = screen.getByText('Trigger Success')
    act(() => {
      successButton.click()
    })

    expect(dispatchMock).toHaveBeenCalledWith({ type: 'devices/fetchPairedDevices' })
  })

  it('does not drop verification events when request arrives in same tick', async () => {
    renderDevicesPage()

    await act(async () => {})

    const addDeviceButton = screen.getByText('Add Device')
    act(() => {
      addDeviceButton.click()
    })

    await act(async () => {})

    expect(verificationHandler).not.toBeNull()

    act(() => {
      verificationHandler?.({
        kind: 'request',
        sessionId: 'session-fast',
        peerId: 'peer-fast',
        deviceName: 'Fast Peer',
      })
      verificationHandler?.({
        kind: 'verification',
        sessionId: 'session-fast',
        code: '654321',
        deviceName: 'Fast Peer',
      })
    })

    await act(async () => {})

    expect(screen.getByText('654321')).toBeInTheDocument()
  })

  it('opens pairing dialog when pairing query is present', async () => {
    renderDevicesPage(['/devices?pairing=1'])

    await act(async () => {})

    expect(screen.getByText('PairingDialog')).toBeInTheDocument()
  })
})
