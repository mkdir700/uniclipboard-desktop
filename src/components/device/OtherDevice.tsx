import { motion, AnimatePresence } from 'framer-motion'
import { Smartphone, Monitor, Tablet, Settings, Eye, Trash2, Laptop, RefreshCw } from 'lucide-react'
import React, { useEffect, useState } from 'react'
import DeviceSettingsPanel from './DeviceSettingsPanel'
import { onP2PPeerConnectionChanged, unpairP2PDevice } from '@/api/p2p'
import { formatPeerIdForDisplay } from '@/lib/utils'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import {
  fetchPairedDevices,
  clearPairedDevicesError,
  updatePeerConnectionStatus,
} from '@/store/slices/devicesSlice'

const OtherDevice: React.FC = () => {
  const [expandedDevices, setExpandedDevices] = useState<Record<string, boolean>>({})
  const dispatch = useAppDispatch()
  const { pairedDevices, pairedDevicesLoading, pairedDevicesError } = useAppSelector(
    state => state.devices
  )

  useEffect(() => {
    // 组件挂载时获取已配对设备
    dispatch(fetchPairedDevices())

    // 监听连接状态变化
    let unlistenConnection: (() => void) | undefined

    const setupConnectionListener = async () => {
      unlistenConnection = await onP2PPeerConnectionChanged(event => {
        // 更新 Redux store 中的连接状态
        dispatch(
          updatePeerConnectionStatus({
            peerId: event.peerId,
            connected: event.connected,
          })
        )
      })
    }

    setupConnectionListener()

    return () => {
      // 清理监听器
      unlistenConnection?.()
    }
  }, [dispatch])

  const toggleDevice = (id: string) => {
    setExpandedDevices(prev => ({
      ...prev,
      [id]: !prev[id],
    }))
  }

  const handleUnpair = async (peerId: string) => {
    try {
      await unpairP2PDevice(peerId)
      // 刷新设备列表
      dispatch(fetchPairedDevices())
    } catch (error) {
      console.error('Failed to unpair device:', error)
    }
  }

  const handleRetry = () => {
    dispatch(clearPairedDevicesError())
    dispatch(fetchPairedDevices())
  }

  const getDeviceIcon = (deviceName?: string | null) => {
    const name = deviceName?.toLowerCase() || ''
    if (name.includes('iphone') || name.includes('phone') || name.includes('android'))
      return Smartphone
    if (name.includes('ipad') || name.includes('tablet')) return Tablet
    if (
      name.includes('mac') ||
      name.includes('macbook') ||
      name.includes('pc') ||
      name.includes('windows')
    )
      return Laptop
    return Monitor
  }

  const getIconColor = (index: number) => {
    const colors = [
      'text-blue-500 bg-blue-500/10 border-blue-500/20',
      'text-purple-500 bg-purple-500/10 border-purple-500/20',
      'text-green-500 bg-green-500/10 border-green-500/20',
      'text-orange-500 bg-orange-500/10 border-orange-500/20',
      'text-primary bg-primary/10 border-primary/20',
    ]
    return colors[index % colors.length]
  }

  // 加载状态
  if (pairedDevicesLoading) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-4 mb-4 mt-8">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            其他已连接设备
          </h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        {[1, 2, 3].map(i => (
          <div key={i} className="border border-border/50 rounded-lg bg-card p-6">
            <div className="animate-pulse flex items-center gap-5">
              <div className="h-14 w-14 bg-muted rounded-md"></div>
              <div className="space-y-2 flex-1">
                <div className="h-5 bg-muted rounded w-32"></div>
                <div className="h-4 bg-muted rounded w-24"></div>
              </div>
            </div>
          </div>
        ))}
      </div>
    )
  }

  // 错误状态
  if (pairedDevicesError) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-4 mb-4 mt-8">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            其他已连接设备
          </h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <div className="border border-destructive/50 rounded-lg bg-card p-6">
          <div className="flex items-center gap-3">
            <p className="text-sm text-destructive">{pairedDevicesError}</p>
            <button
              onClick={handleRetry}
              className="p-1.5 text-destructive hover:bg-destructive/10 rounded-lg transition-colors"
              title="重试"
            >
              <RefreshCw className="h-4 w-4" />
            </button>
          </div>
        </div>
      </div>
    )
  }

  // 空状态
  if (pairedDevices.length === 0) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-4 mb-4 mt-8">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            其他已连接设备
          </h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <div className="flex flex-col items-center justify-center p-12 border border-dashed border-border/50 rounded-lg bg-muted/5 text-muted-foreground">
          <p className="text-sm">暂无已配对的设备</p>
          <p className="text-xs mt-2">点击右上角的"添加设备"按钮开始配对</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-4 mb-4 mt-8">
        <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
          其他已连接设备
        </h3>
        <div className="h-px flex-1 bg-border/50"></div>
      </div>

      {pairedDevices.map((device, index) => {
        const Icon = getDeviceIcon(device.deviceName)
        const isExpanded = expandedDevices[device.peerId] || false
        const iconColor = getIconColor(index)

        return (
          <div
            key={device.peerId}
            className="group relative overflow-hidden bg-card/50 hover:bg-card/80 border border-border/50 hover:border-primary/20 rounded-lg transition-all duration-300 shadow-sm hover:shadow-md"
          >
            <div className="relative z-10 p-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-5">
                  {/* Icon Box */}
                  <div
                    className={`h-14 w-14 rounded-md flex items-center justify-center ring-1 shadow-inner ${iconColor}`}
                  >
                    <Icon className="h-7 w-7" />
                  </div>

                  {/* Info */}
                  <div>
                    <div className="flex items-center gap-3">
                      <h4 className="text-lg font-semibold text-foreground tracking-tight">
                        {device.deviceName || '未知设备'}
                      </h4>
                      <span
                        className={`px-2.5 py-0.5 text-xs font-medium rounded-full border ${iconColor}`}
                      >
                        已配对
                      </span>
                    </div>
                    <p className="text-sm text-muted-foreground mt-1">
                      ID: {formatPeerIdForDisplay(device.peerId)}
                    </p>
                  </div>
                </div>

                {/* Actions & Status */}
                <div className="flex items-center gap-6">
                  {/* Status Indicator - 动态显示在线/离线状态 */}
                  <div
                    className={`flex items-center gap-2 px-3 py-1.5 rounded-full border ${
                      device.connected
                        ? 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20'
                        : 'bg-muted-foreground/10 text-muted-foreground border-border'
                    }`}
                  >
                    <span
                      className={`relative inline-flex rounded-full h-2 w-2 ${
                        device.connected ? 'bg-green-500 animate-pulse' : 'bg-gray-400'
                      }`}
                    ></span>
                    <span className="text-xs font-medium">
                      {device.connected ? '在线' : '离线'}
                    </span>
                  </div>

                  {/* Action Buttons */}
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => toggleDevice(device.peerId)}
                      className={`p-2 rounded-xl transition-all duration-300 ${isExpanded ? 'bg-primary text-primary-foreground shadow-lg shadow-primary/25' : 'text-muted-foreground hover:text-foreground hover:bg-muted'}`}
                      title="设置"
                    >
                      <Settings
                        className={`h-5 w-5 transition-transform duration-500 ${isExpanded ? 'rotate-90' : ''}`}
                      />
                    </button>
                    <button
                      className="p-2 text-muted-foreground hover:text-foreground hover:bg-muted rounded-xl transition-colors"
                      title="查看"
                    >
                      <Eye className="h-5 w-5" />
                    </button>
                    <button
                      onClick={() => handleUnpair(device.peerId)}
                      className="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-xl transition-colors"
                      title="取消配对"
                    >
                      <Trash2 className="h-5 w-5" />
                    </button>
                  </div>
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
                      <DeviceSettingsPanel
                        deviceId={device.peerId}
                        deviceName={device.deviceName || '未知设备'}
                      />
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          </div>
        )
      })}
    </div>
  )
}

export default OtherDevice
