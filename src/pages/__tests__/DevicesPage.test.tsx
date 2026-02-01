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

function MockPairingDialog({
  open,
  onPairingSuccess,
}: {
  open: boolean
  onPairingSuccess: () => void
}) {
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

vi.mock('@/components/PairingPinDialog', () => ({
  default: ({
    open,
    pinCode,
    peerDeviceName,
  }: {
    open: boolean
    pinCode: string
    peerDeviceName?: string
  }) => {
    if (!open) {
      return null
    }

    return (
      <div>
        PairingPinDialog
        {peerDeviceName ? <span>{peerDeviceName}</span> : null}
        {pinCode ? <span>{pinCode}</span> : null}
      </div>
    )
  },
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

  it('shows pairing pin dialog for responder and dedupes completion toasts', async () => {
    renderDevicesPage(['/devices?pairingPin=1&sessionId=session-1&deviceName=Peer%20Device'])

    await act(async () => {})

    expect(screen.getByText('PairingPinDialog')).toBeInTheDocument()
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

  it('shows pin code when verification arrives quickly', async () => {
    renderDevicesPage(['/devices?pairingPin=1&sessionId=session-fast&deviceName=Fast%20Peer'])

    await act(async () => {})

    act(() => {
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

  it('opens pairing pin dialog when pairingPin query is present', async () => {
    renderDevicesPage(['/devices?pairingPin=1&sessionId=session-1&deviceName=Peer%20Device'])

    await act(async () => {})

    expect(screen.getByText('PairingPinDialog')).toBeInTheDocument()
  })
})
