import { motion, AnimatePresence } from 'framer-motion'
import {
  Smartphone,
  Monitor,
  Tablet,
  Trash2,
  Laptop,
  RefreshCw,
  Plus,
  ChevronRight,
} from 'lucide-react'
import React, { useEffect, useState } from 'react'
import DeviceSettingsPanel from './DeviceSettingsPanel'
import { onP2PPeerConnectionChanged, onP2PPeerNameUpdated, unpairP2PDevice } from '@/api/p2p'
import { formatPeerIdForDisplay } from '@/lib/utils'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import {
  fetchPairedDevices,
  clearPairedDevicesError,
  updatePeerConnectionStatus,
  updatePeerDeviceName,
} from '@/store/slices/devicesSlice'

interface OtherDeviceProps {
  onAddDevice: () => void
}

const OtherDevice: React.FC<OtherDeviceProps> = ({ onAddDevice }) => {
  const [expandedDeviceId, setExpandedDeviceId] = useState<string | null>(null)
  const dispatch = useAppDispatch()
  const { pairedDevices, pairedDevicesLoading, pairedDevicesError } = useAppSelector(
    state => state.devices
  )

  useEffect(() => {
    dispatch(fetchPairedDevices())

    let unlistenConnection: (() => void) | undefined
    let unlistenName: (() => void) | undefined

    const setupConnectionListener = async () => {
      unlistenConnection = await onP2PPeerConnectionChanged(event => {
        dispatch(
          updatePeerConnectionStatus({
            peerId: event.peerId,
            connected: event.connected,
            deviceName: event.deviceName ?? undefined,
          })
        )
      })
    }

    const setupNameListener = async () => {
      unlistenName = await onP2PPeerNameUpdated(event => {
        dispatch(
          updatePeerDeviceName({
            peerId: event.peerId,
            deviceName: event.deviceName,
          })
        )
      })
    }

    setupConnectionListener()
    setupNameListener()

    return () => {
      unlistenConnection?.()
      unlistenName?.()
    }
  }, [dispatch])

  const toggleDevice = (id: string) => {
    setExpandedDeviceId(prev => (prev === id ? null : id))
  }

  const handleUnpair = async (e: React.MouseEvent, peerId: string) => {
    e.stopPropagation()
    try {
      await unpairP2PDevice(peerId)
      dispatch(fetchPairedDevices())
      if (expandedDeviceId === peerId) {
        setExpandedDeviceId(null)
      }
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

  if (pairedDevicesLoading) {
    return (
      <div className="space-y-4">
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

  if (pairedDevicesError) {
    return (
      <div className="space-y-4">
        <div className="border border-destructive/50 rounded-lg bg-card p-6">
          <div className="flex items-center gap-3">
            <p className="text-sm text-destructive">{pairedDevicesError}</p>
            <button
              type="button"
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

  if (pairedDevices.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-20 text-center">
        <div className="bg-muted/30 p-6 rounded-full mb-6 ring-1 ring-border/50">
          <Monitor className="h-12 w-12 text-muted-foreground/50" />
        </div>
        <h3 className="text-xl font-semibold text-foreground mb-2">No paired devices</h3>
        <p className="text-muted-foreground max-w-xs mb-8">
          Connect your other devices to start syncing clipboard content instantly.
        </p>
        <button
          type="button"
          onClick={onAddDevice}
          className="inline-flex items-center gap-2 px-6 py-3 bg-primary text-primary-foreground rounded-full font-medium hover:bg-primary/90 transition-all shadow-lg shadow-primary/20 hover:shadow-primary/30 hover:-translate-y-0.5"
        >
          <Plus className="h-5 w-5" />
          Add Device
        </button>
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-col rounded-xl border border-border/50 bg-card/50 overflow-hidden divide-y divide-border/50">
        {pairedDevices.map((device, index) => {
          const Icon = getDeviceIcon(device.deviceName)
          const isExpanded = expandedDeviceId === device.peerId
          const iconColor = getIconColor(index)

          return (
            <div key={device.peerId} className="flex flex-col bg-card/30">
              <div
                role="button"
                tabIndex={0}
                aria-expanded={isExpanded}
                aria-controls={`device-settings-${device.peerId}`}
                onClick={() => toggleDevice(device.peerId)}
                onKeyDown={e => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault()
                    toggleDevice(device.peerId)
                  }
                }}
                className={`
                  relative flex items-center justify-between p-4 cursor-pointer outline-none
                  hover:bg-accent/50 transition-colors duration-200
                  ${isExpanded ? 'bg-accent/50' : ''}
                `}
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`h-10 w-10 rounded-lg flex items-center justify-center ring-1 shadow-sm ${iconColor}`}
                  >
                    <Icon className="h-5 w-5" />
                  </div>

                  <div className="flex flex-col gap-0.5">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-foreground text-sm">
                        {device.deviceName || '未知设备'}
                      </span>
                      {device.connected && (
                        <span className="flex h-2 w-2 rounded-full bg-green-500 animate-pulse" />
                      )}
                    </div>
                    <span className="text-xs text-muted-foreground font-mono">
                      {formatPeerIdForDisplay(device.peerId)}
                    </span>
                  </div>
                </div>

                <div className="flex items-center gap-3">
                  <div
                    className={`text-xs px-2 py-0.5 rounded-full border ${
                      device.connected
                        ? 'bg-green-500/10 text-green-600 border-green-500/20'
                        : 'bg-muted text-muted-foreground border-border'
                    }`}
                  >
                    {device.connected ? '在线' : '离线'}
                  </div>

                  <div className="flex items-center gap-1 pl-2 border-l border-border/50">
                    <button
                      type="button"
                      onClick={e => handleUnpair(e, device.peerId)}
                      className="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-colors"
                      title="取消配对"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                    <ChevronRight
                      className={`h-4 w-4 text-muted-foreground transition-transform duration-200 ${
                        isExpanded ? 'rotate-90' : ''
                      }`}
                    />
                  </div>
                </div>
              </div>

              <AnimatePresence>
                {isExpanded && (
                  <motion.div
                    id={`device-settings-${device.peerId}`}
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: 'auto', opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.2, ease: 'easeInOut' }}
                    className="overflow-hidden bg-accent/20"
                  >
                    <div className="p-4 border-t border-border/50">
                      <DeviceSettingsPanel
                        deviceId={device.peerId}
                        deviceName={device.deviceName || '未知设备'}
                      />
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          )
        })}
      </div>

      <button
        type="button"
        onClick={onAddDevice}
        className="w-full group relative overflow-hidden bg-card/30 hover:bg-card/50 border border-dashed border-border hover:border-primary/50 rounded-lg transition-all duration-300 p-3 flex items-center justify-center gap-2 text-muted-foreground hover:text-primary"
      >
        <Plus className="h-4 w-4" />
        <span className="text-sm font-medium">Add another device</span>
      </button>
    </div>
  )
}

export default OtherDevice
