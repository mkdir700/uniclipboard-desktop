import React, { useState, useRef, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import {
  acceptP2PPairing,
  onP2PPairingVerification,
  verifyP2PPairingPin,
  rejectP2PPairing,
  type P2PPairingVerificationEvent,
} from '@/api/p2p'
import { DeviceList, DeviceHeader } from '@/components'
import { DeviceTab } from '@/components/device/Header'
import PairingDialog from '@/components/PairingDialog'
import PairingPinDialog from '@/components/PairingPinDialog'
import { toast } from '@/components/ui/toast'
import { captureUserIntent } from '@/observability/breadcrumbs'
import { useAppDispatch } from '@/store/hooks'
import { fetchPairedDevices } from '@/store/slices/devicesSlice'

// P2P配对请求状态
type P2PPairingRequestWithPin = (P2PPairingVerificationEvent & { kind: 'request' }) & {
  pin?: string
  peerDeviceName?: string
}

const DevicesPage: React.FC = () => {
  const { t } = useTranslation()
  const dispatch = useAppDispatch()
  const [showPairingDialog, setShowPairingDialog] = useState(false)
  const [activeTab, setActiveTab] = useState<DeviceTab>('connected')

  // P2P配对请求相关状态
  const [pendingP2PRequest, setPendingP2PRequest] = useState<P2PPairingRequestWithPin | null>(null)
  const [acceptingP2PRequest, setAcceptingP2PRequest] = useState(false)
  const [showPinDialog, setShowPinDialog] = useState(false)
  const [pinCode, setPinCode] = useState('')
  const [pinPeerDeviceName, setPinPeerDeviceName] = useState<string>('')
  const [pairingSessionId, setPairingSessionId] = useState<string>('')
  const cleanupRefs = useRef<(() => void)[]>([])

  // Refs for scrolling
  const connectedRef = useRef<HTMLDivElement>(null)
  const requestsRef = useRef<HTMLDivElement>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  const setupP2PRequestListener = useCallback(async () => {
    try {
      const unlisten = await onP2PPairingVerification(event => {
        if (event.kind === 'request') {
          console.log('Received P2P pairing request:', event)
          const requestEvent = event as P2PPairingVerificationEvent & { kind: 'request' }
          setPendingP2PRequest({
            ...requestEvent,
            pin: undefined,
            peerDeviceName: undefined,
          })
          setAcceptingP2PRequest(false)
          setActiveTab('requests')
          return
        }

        if (event.kind === 'verification') {
          console.log('Received P2P verification event (responder):', event)
          setPinCode(event.code ?? '')
          setPinPeerDeviceName(event.deviceName || t('pairing.discovery.unknownDevice'))
          setPairingSessionId(event.sessionId)
          setShowPinDialog(true)
          return
        }

        if (event.kind === 'complete') {
          console.log('P2P pairing completed')
          setShowPinDialog(false)
          setPendingP2PRequest(null)
          setAcceptingP2PRequest(false)
          toast.success(t('pairing.success.title'))
          dispatch(fetchPairedDevices())
          return
        }

        console.error('P2P pairing failed:', event)
        setShowPinDialog(false)
        setPendingP2PRequest(null)
        setAcceptingP2PRequest(false)
        toast.error(t('pairing.failed.title'), {
          description: event.error || '',
        })
      })
      cleanupRefs.current.push(unlisten)
    } catch (error) {
      console.error('Failed to setup P2P pairing request listener:', error)
    }
  }, [t, dispatch])

  useEffect(() => {
    // 设置P2P配对请求监听
    setupP2PRequestListener()

    return () => {
      // 清理事件监听
      cleanupRefs.current.forEach(cleanup => {
        cleanup()
      })
      cleanupRefs.current = []
    }
  }, [setupP2PRequestListener])

  const handleAddDevice = () => {
    captureUserIntent('pair_device', { source: 'add_device' })
    setShowPairingDialog(true)
  }

  const handleClosePairingDialog = () => {
    setShowPairingDialog(false)
  }

  const handlePairingSuccess = () => {
    // 连接成功后刷新设备列表
    // TODO: 可以添加刷新设备列表的逻辑
  }

  const handleAcceptPairing = async () => {
    if (!pendingP2PRequest) return

    captureUserIntent('pair_device', { source: 'request' })
    setAcceptingP2PRequest(true)
    try {
      await acceptP2PPairing(pendingP2PRequest.sessionId)
      // After accepting, backend will emit `verification` event with PIN.
    } catch (error) {
      console.error('Failed to accept pairing request:', error)
      setAcceptingP2PRequest(false)
      toast.error(t('pairing.failed.title'), {
        description: error instanceof Error ? error.message : String(error),
      })
    }
  }

  const handleRejectPairing = async () => {
    if (pendingP2PRequest) {
      try {
        if (pendingP2PRequest.peerId) {
          await rejectP2PPairing(pendingP2PRequest.sessionId, pendingP2PRequest.peerId)
        } else {
          console.warn('Missing peerId for pairing rejection')
        }
        setPendingP2PRequest(null)
      } catch (error) {
        console.error('Failed to reject pairing:', error)
      }
    }
  }

  // 处理 PIN 验证（接收方确认 PIN 码）
  const handlePinVerify = async (matches: boolean) => {
    if (!pairingSessionId) return

    try {
      await verifyP2PPairingPin({
        sessionId: pairingSessionId,
        pinMatches: matches,
      })
      // 如果不匹配，关闭对话框
      if (!matches) {
        setShowPinDialog(false)
        setPendingP2PRequest(null)
      }
      // 如果匹配，等待配对完成/失败事件
    } catch (error) {
      console.error('Failed to verify PIN:', error)
      setShowPinDialog(false)
      setPendingP2PRequest(null)
    }
  }

  const handleTabChange = (tab: DeviceTab) => {
    setActiveTab(tab)
    let targetRef: React.RefObject<HTMLDivElement> | undefined
    switch (tab) {
      case 'connected':
        targetRef = connectedRef
        break
      case 'requests':
        targetRef = requestsRef
        break
    }

    if (targetRef?.current) {
      targetRef.current.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  }

  // Optional: Update active tab on scroll
  useEffect(() => {
    const container = scrollContainerRef.current
    if (!container) return

    const handleScroll = () => {
      const positions = [
        { id: 'connected' as DeviceTab, ref: connectedRef },
        { id: 'requests' as DeviceTab, ref: requestsRef },
      ]

      // Simple proximity check
      for (const { id, ref } of positions) {
        if (ref.current) {
          const rect = ref.current.getBoundingClientRect()
          // If the element is near the top of the viewport (with some offset for header)
          if (rect.top >= 0 && rect.top < 300) {
            setActiveTab(id)
            break
          }
        }
      }
    }

    container.addEventListener('scroll', handleScroll)
    return () => container.removeEventListener('scroll', handleScroll)
  }, [])

  return (
    <div className="flex flex-col h-full relative">
      {/* 顶部标题栏 */}
      <DeviceHeader
        addDevice={handleAddDevice}
        activeTab={activeTab}
        onTabChange={handleTabChange}
      />

      {/* 内容区域 */}
      <div className="flex-1 overflow-hidden relative">
        <div
          ref={scrollContainerRef}
          className="h-full overflow-y-auto scrollbar-thin px-8 pb-32 pt-2 scroll-smooth"
        >
          {/* Pairing Requests Section */}
          <div id="requests" ref={requestsRef} className="scroll-mt-24 mb-12">
            <div className="flex items-center gap-4 mb-4 mt-8">
              <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
                {t('devices.sections.requests')}
              </h3>
              <div className="h-px flex-1 bg-border/50"></div>
            </div>

            {pendingP2PRequest ? (
              <div className="border border-border/50 rounded-lg bg-card p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center">
                      <svg
                        aria-hidden="true"
                        className="w-5 h-5 text-primary"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
                        />
                      </svg>
                    </div>
                    <div>
                      <h4 className="font-medium text-sm">
                        {pendingP2PRequest.deviceName || t('pairing.discovery.unknownDevice')}
                      </h4>
                      <p className="text-xs text-muted-foreground">
                        ID: {(pendingP2PRequest.peerId ?? '').substring(0, 8)}...
                      </p>
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <button
                      type="button"
                      onClick={handleRejectPairing}
                      disabled={acceptingP2PRequest}
                      className="px-3 py-1.5 text-sm font-medium rounded-md border border-border bg-background hover:bg-muted transition-colors"
                    >
                      {t('pairing.requests.reject')}
                    </button>
                    <button
                      type="button"
                      onClick={handleAcceptPairing}
                      disabled={acceptingP2PRequest}
                      className="px-3 py-1.5 text-sm font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {acceptingP2PRequest
                        ? t('pairing.requests.accepting')
                        : t('pairing.requests.accept')}
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              <div className="flex flex-col items-center justify-center p-8 border border-dashed border-border/50 rounded-lg bg-muted/5 text-muted-foreground">
                <p>{t('devices.sections.noRequests')}</p>
              </div>
            )}
          </div>

          {/* Connected Devices Section */}
          <div id="connected" ref={connectedRef} className="scroll-mt-24 mb-12">
            <DeviceList />
          </div>
        </div>
      </div>

      {/* P2P Pairing Dialog (for initiating pairing) */}
      <PairingDialog
        open={showPairingDialog}
        onClose={handleClosePairingDialog}
        onPairingSuccess={handlePairingSuccess}
      />

      {/* PIN Verification Dialog (for responder) */}
      <PairingPinDialog
        open={showPinDialog}
        onClose={() => {
          setShowPinDialog(false)
          setPendingP2PRequest(null)
        }}
        pinCode={pinCode}
        peerDeviceName={pinPeerDeviceName}
        isInitiator={false}
        onConfirm={handlePinVerify}
      />
    </div>
  )
}

export default DevicesPage
