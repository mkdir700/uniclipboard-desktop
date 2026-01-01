import { motion, AnimatePresence } from 'framer-motion'
import { Laptop, Settings, RefreshCw } from 'lucide-react'
import React, { useEffect, useState } from 'react'
import DeviceSettingsPanel from './DeviceSettingsPanel'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import { fetchLocalDeviceInfo, clearLocalDeviceError } from '@/store/slices/devicesSlice'

const CurrentDevice: React.FC = () => {
  const [isExpanded, setIsExpanded] = useState(false)
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
        <div className="flex items-center gap-4 mb-4">
          <h3 className="text-sm font-medium text-muted-foreground">当前设备</h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
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
        <div className="flex items-center gap-4 mb-4">
          <h3 className="text-sm font-medium text-muted-foreground">当前设备</h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <div className="border border-destructive/50 rounded-lg bg-card p-6">
          <div className="flex items-center gap-3">
            <p className="text-sm text-destructive">{localDeviceError || '无法获取当前设备信息'}</p>
            {localDeviceError && (
              <button
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
      <div className="flex items-center gap-4 mb-4">
        <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">当前设备</h3>
        <div className="h-px flex-1 bg-border/50"></div>
      </div>

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
                  ID: {localDevice.peerId.substring(0, 8)}...
                </p>
              </div>
            </div>

            {/* Actions & Status */}
            <div className="flex items-center gap-6">
              {/* Status Indicator */}
              <div className="flex items-center gap-2 px-3 py-1.5 bg-green-500/10 text-green-500 rounded-full border border-green-500/20">
                <span className="relative flex h-2 w-2">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-500 opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
                </span>
                <span className="text-xs font-medium">在线</span>
              </div>

              {/* Action Buttons */}
              <button
                onClick={() => setIsExpanded(!isExpanded)}
                className={`p-2 rounded-lg transition-all duration-300 ${isExpanded ? 'bg-primary text-primary-foreground shadow-lg shadow-primary/25' : 'text-muted-foreground hover:text-foreground hover:bg-muted'}`}
              >
                <Settings
                  className={`h-5 w-5 transition-transform duration-500 ${isExpanded ? 'rotate-90' : ''}`}
                />
              </button>
            </div>
          </div>

          {/* Expandable Settings Panel */}
          <AnimatePresence>
            {isExpanded && (
              <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: 'auto', opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                transition={{ duration: 0.3, ease: 'easeInOut' }}
                className="overflow-hidden"
              >
                <div className="pt-6 border-t border-border/50 mt-6">
                  <DeviceSettingsPanel deviceId="current" deviceName={localDevice.deviceName} />
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  )
}

export default CurrentDevice
