import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import DevicesPage from '@/pages/DevicesPage'

const dispatchMock = vi.hoisted(() => vi.fn())

vi.mock('@/store/hooks', () => ({
  useAppDispatch: () => dispatchMock,
}))

vi.mock('@/store/slices/devicesSlice', () => ({
  fetchPairedDevices: () => ({ type: 'devices/fetchPairedDevices' }),
}))

vi.mock('@/components', () => ({
  DeviceList: ({ onAddDevice }: { onAddDevice: () => void }) => (
    <div>
      <div data-testid="device-list">DeviceList</div>
      <button type="button" onClick={onAddDevice}>
        Open Pairing
      </button>
    </div>
  ),
}))

vi.mock('@/components/PairingDialog', () => ({
  default: ({ open, onPairingSuccess }: { open: boolean; onPairingSuccess: () => void }) =>
    open ? (
      <div>
        PairingDialog
        <button type="button" onClick={onPairingSuccess}>
          Trigger Success
        </button>
      </div>
    ) : null,
}))

describe('DevicesPage', () => {
  it('does not render legacy header and pairing requests sections', () => {
    render(<DevicesPage />)

    expect(screen.getByTestId('device-list')).toBeInTheDocument()
    expect(screen.queryByText('Device Management')).not.toBeInTheDocument()
    expect(screen.queryByText('Pairing Requests')).not.toBeInTheDocument()
    expect(screen.queryByText('当前设备')).not.toBeInTheDocument()
  })

  it('opens pairing dialog and refreshes devices on pairing success', () => {
    render(<DevicesPage />)

    fireEvent.click(screen.getByText('Open Pairing'))
    expect(screen.getByText('PairingDialog')).toBeInTheDocument()

    fireEvent.click(screen.getByText('Trigger Success'))
    expect(dispatchMock).toHaveBeenCalledWith({ type: 'devices/fetchPairedDevices' })
  })
})
