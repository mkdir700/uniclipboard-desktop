import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import OtherDevice from '../OtherDevice'

const dispatchMock = vi.fn()
const useAppSelectorMock = vi.fn()

vi.mock('@/store/hooks', () => ({
  useAppDispatch: () => dispatchMock,
  useAppSelector: vi.fn(fn => useAppSelectorMock(fn)),
}))

vi.mock('@/api/p2p', () => ({
  onP2PPeerConnectionChanged: vi.fn(() => Promise.resolve(() => {})),
  onP2PPeerNameUpdated: vi.fn(() => Promise.resolve(() => {})),
  unpairP2PDevice: vi.fn(),
}))

vi.mock('@/store/slices/devicesSlice', () => ({
  fetchPairedDevices: vi.fn(() => ({ type: 'devices/fetchPairedDevices' })),
  clearPairedDevicesError: vi.fn(() => ({ type: 'devices/clearPairedDevicesError' })),
  updatePeerConnectionStatus: vi.fn(),
  updatePeerDeviceName: vi.fn(),
}))

describe('OtherDevice', () => {
  it('renders empty state with discovery message when no devices are paired', () => {
    useAppSelectorMock.mockImplementation(selector => {
      const state = {
        devices: {
          pairedDevices: [],
          pairedDevicesLoading: false,
          pairedDevicesError: null,
        },
      }
      return selector(state)
    })

    render(<OtherDevice />)

    expect(screen.getByText('暂无已配对的设备')).toBeInTheDocument()
    expect(screen.getByText(/正在发现设备/)).toBeInTheDocument()
    expect(screen.getByText(/可能需要几秒钟/)).toBeInTheDocument()
  })
})
