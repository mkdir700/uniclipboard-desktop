import { Laptop, RefreshCw } from 'lucide-react'
import React, { useEffect } from 'react'
import { formatPeerIdForDisplay } from '@/lib/utils'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import { fetchLocalDeviceInfo, clearLocalDeviceError } from '@/store/slices/devicesSlice'

const CurrentDevice: React.FC = () => {
  const dispatch = useAppDispatch()
  const { localDevice, localDeviceLoading, localDeviceError } = useAppSelector(
    state => state.devices
  )

  useEffect(() => {
    // 组件挂载时获取当前设备信息
    dispatch(fetchLocalDeviceInfo())
  }, [dispatch])

  const handleRetry = () => {
    dispatch(clearLocalDeviceError())
    dispatch(fetchLocalDeviceInfo())
  }

  // 加载状态
  if (localDeviceLoading) {
    return (
      <div className="mb-8">
        <div className="border border-border/50 rounded-lg bg-card p-6">
          <div className="animate-pulse flex items-center gap-5">
            <div className="h-14 w-14 bg-muted rounded-md"></div>
            <div className="space-y-2">
              <div className="h-5 bg-muted rounded w-32"></div>
              <div className="h-4 bg-muted rounded w-24"></div>
            </div>
          </div>
        </div>
      </div>
    )
  }

  // 错误状态
  if (localDeviceError || !localDevice) {
    return (
      <div className="mb-8">
        <div className="border border-destructive/50 rounded-lg bg-card p-6">
          <div className="flex items-center gap-3">
            <p className="text-sm text-destructive">{localDeviceError || '无法获取当前设备信息'}</p>
            {localDeviceError && (
              <button
                type="button"
                onClick={handleRetry}
                className="p-1.5 text-destructive hover:bg-destructive/10 rounded-lg transition-colors"
                title="重试"
              >
                <RefreshCw className="h-4 w-4" />
              </button>
            )}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="mb-8">
      <div className="group relative overflow-hidden bg-card/50 hover:bg-card/80 border border-border/50 hover:border-primary/20 rounded-lg transition-all duration-300 shadow-sm hover:shadow-md">
        {/* Abstract Background Gradient */}
        <div className="absolute top-0 right-0 p-12 bg-primary/5 blur-[80px] rounded-full pointer-events-none" />

        <div className="relative z-10 p-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-5">
              {/* Icon Box */}
              <div className="h-14 w-14 bg-primary/10 rounded-md flex items-center justify-center ring-1 ring-primary/20 shadow-inner">
                <Laptop className="h-7 w-7 text-primary" />
              </div>

              {/* Info */}
              <div>
                <div className="flex items-center gap-3">
                  <h4 className="text-lg font-semibold text-foreground tracking-tight">
                    {localDevice.deviceName}
                  </h4>
                  <span className="px-2.5 py-0.5 bg-primary/15 text-primary text-xs font-medium rounded-full border border-primary/10">
                    当前设备
                  </span>
                </div>
                <p className="text-sm text-muted-foreground mt-1">
                  ID: {formatPeerIdForDisplay(localDevice.peerId)}
                </p>
              </div>
            </div>

            {/* Status Indicator */}
            <div className="flex items-center gap-6">
              <div className="flex items-center gap-2 px-3 py-1.5 bg-green-500/10 text-green-500 rounded-full border border-green-500/20">
                <span className="relative flex h-2 w-2">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-500 opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
                </span>
                <span className="text-xs font-medium">在线</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default CurrentDevice
