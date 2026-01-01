import { createAsyncThunk, createSlice } from '@reduxjs/toolkit'
import {
  getLocalDeviceInfo,
  getPairedPeers,
  type LocalDeviceInfo,
  type PairedPeer,
} from '@/api/p2p'

interface DevicesState {
  // 当前设备
  localDevice: LocalDeviceInfo | null
  localDeviceLoading: boolean
  localDeviceError: string | null

  // 已配对的设备
  pairedDevices: PairedPeer[]
  pairedDevicesLoading: boolean
  pairedDevicesError: string | null
}

const initialState: DevicesState = {
  localDevice: null,
  localDeviceLoading: false,
  localDeviceError: null,
  pairedDevices: [],
  pairedDevicesLoading: false,
  pairedDevicesError: null,
}

// 异步 Thunk Actions
export const fetchLocalDeviceInfo = createAsyncThunk(
  'devices/fetchLocalInfo',
  async (_, { rejectWithValue }) => {
    try {
      return await getLocalDeviceInfo()
    } catch {
      return rejectWithValue('获取当前设备信息失败')
    }
  }
)

export const fetchPairedDevices = createAsyncThunk(
  'devices/fetchPaired',
  async (_, { rejectWithValue }) => {
    try {
      return await getPairedPeers()
    } catch {
      return rejectWithValue('获取已配对设备失败')
    }
  }
)

const devicesSlice = createSlice({
  name: 'devices',
  initialState,
  reducers: {
    clearLocalDeviceError: state => {
      state.localDeviceError = null
    },
    clearPairedDevicesError: state => {
      state.pairedDevicesError = null
    },
  },
  extraReducers: builder => {
    // Local device info
    builder
      .addCase(fetchLocalDeviceInfo.pending, state => {
        state.localDeviceLoading = true
        state.localDeviceError = null
      })
      .addCase(fetchLocalDeviceInfo.fulfilled, (state, action) => {
        state.localDeviceLoading = false
        state.localDevice = action.payload
      })
      .addCase(fetchLocalDeviceInfo.rejected, (state, action) => {
        state.localDeviceLoading = false
        state.localDeviceError = action.payload as string
      })

    // Paired devices
    builder
      .addCase(fetchPairedDevices.pending, state => {
        state.pairedDevicesLoading = true
        state.pairedDevicesError = null
      })
      .addCase(fetchPairedDevices.fulfilled, (state, action) => {
        state.pairedDevicesLoading = false
        state.pairedDevices = action.payload
      })
      .addCase(fetchPairedDevices.rejected, (state, action) => {
        state.pairedDevicesLoading = false
        state.pairedDevicesError = action.payload as string
      })
  },
})

export const { clearLocalDeviceError, clearPairedDevicesError } = devicesSlice.actions
export default devicesSlice.reducer
