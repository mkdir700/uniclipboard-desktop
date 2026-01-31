import {
  Loader2,
  RefreshCw,
  Monitor,
  Smartphone,
  Laptop,
  AlertCircle,
  ShieldCheck,
} from 'lucide-react'
import React, { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  getP2PPeers,
  initiateP2PPairing,
  verifyP2PPairingPin,
  onP2PPairingVerification,
  type P2PPeerInfo,
} from '@/api/p2p'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/toast'
// import { getLocalDeviceName } from '@/api/deviceConnection' // Assuming we'll add this or use existing

// Mock getLocalDeviceName if not available yet, or import if available
// For now, let's assume we might need to fetch it or just display peers.

type PairingStep = 'discovery' | 'connecting' | 'pin-verify' | 'success' | 'failed'

interface PairingDialogProps {
  open: boolean
  onClose: () => void
  onPairingSuccess?: () => void
}

export default function PairingDialog({ open, onClose, onPairingSuccess }: PairingDialogProps) {
  const { t } = useTranslation()
  const [step, setStep] = useState<PairingStep>('discovery')
  const [peers, setPeers] = useState<P2PPeerInfo[]>([])
  const [loading, setLoading] = useState(false)
  const [selectedPeer, setSelectedPeer] = useState<P2PPeerInfo | null>(null)
  const [pairingSessionId, setPairingSessionId] = useState<string | null>(null)
  const [pinCode, setPinCode] = useState<string>('')
  const [errorMsg, setErrorMsg] = useState<string>('')
  const [isPinVerifying, setIsPinVerifying] = useState(false)

  // Cleanup refs
  const cleanupRefs = React.useRef<(() => void)[]>([])
  const listenerRegistered = React.useRef(false)

  // Reset state when dialog opens
  const setupListeners = React.useCallback(async () => {
    // 防止重复注册
    if (listenerRegistered.current) {
      console.log('[PairingDialog] Listener already registered, skipping')
      return
    }

    console.log('[PairingDialog] Setting up listeners')
    try {
      const unlistenVerification = await onP2PPairingVerification(event => {
        console.log('[PairingDialog] Event received:', {
          kind: event.kind,
          sessionId: event.sessionId,
          timestamp: new Date().toISOString(),
        })

        if (event.kind === 'verification') {
          console.log('[PairingDialog] Verification event:', event)
          setPairingSessionId(event.sessionId)
          setPinCode(event.code ?? '')
          setStep('pin-verify')
          setIsPinVerifying(false)
          return
        }

        if (event.kind === 'complete') {
          console.log('[PairingDialog] Complete event:', event)
          console.trace('[PairingDialog] Toast success triggered from:')
          setStep('success')
          setIsPinVerifying(false)
          toast.success(t('pairing.success.title'))
          setTimeout(() => {
            onPairingSuccess?.()
            onClose()
          }, 2000)
          return
        }

        if (event.kind === 'failed') {
          console.error('[PairingDialog] Failed event:', event)
          setErrorMsg(event.error ?? '')
          setStep('failed')
          setIsPinVerifying(false)
          toast.error(t('pairing.failed.title'), {
            description: event.error ?? '',
          })
        }
      })
      cleanupRefs.current.push(unlistenVerification)
      listenerRegistered.current = true
      console.log('[PairingDialog] Listener registered successfully')
    } catch (err) {
      console.error('[PairingDialog] Failed to setup listeners:', err)
    }
  }, [onClose, onPairingSuccess, t])

  const loadPeers = React.useCallback(async () => {
    setLoading(true)
    setErrorMsg('')
    try {
      const list = await getP2PPeers()
      // Filter out already paired devices if needed, or just show status
      // For now show all discovered
      setPeers(list)
    } catch (err) {
      console.error('Failed to load peers:', err)
      setErrorMsg(t('pairing.failed.errors.loadPeers'))
    } finally {
      setLoading(false)
    }
  }, [t])

  useEffect(() => {
    if (open) {
      console.log('[PairingDialog] Dialog opened, initializing...')

      // 先清理旧监听器（防止重复注册）
      if (cleanupRefs.current.length > 0) {
        console.log('[PairingDialog] Cleaning up old listeners')
        cleanupRefs.current.forEach(cleanup => cleanup())
        cleanupRefs.current = []
        listenerRegistered.current = false
      }

      // 重置状态
      setStep('discovery')
      setPeers([])
      setSelectedPeer(null)
      setPairingSessionId(null)
      setPinCode('')
      setErrorMsg('')
      setIsPinVerifying(false)

      // 加载对等设备
      loadPeers()

      // 设置监听器（只注册一次）
      setupListeners()
    } else {
      // Cleanup listeners when closed
      console.log('[PairingDialog] Dialog closed, cleaning up listeners')
      cleanupRefs.current.forEach(cleanup => {
        cleanup()
      })
      cleanupRefs.current = []
      listenerRegistered.current = false
    }
  }, [open, loadPeers, setupListeners])

  const handleConnect = async (peer: P2PPeerInfo) => {
    setSelectedPeer(peer)
    setStep('connecting')
    setErrorMsg('')
    try {
      const response = await initiateP2PPairing({ peerId: peer.peerId })
      if (response.success) {
        setPairingSessionId(response.sessionId)
        // Wait for PIN ready event...
      } else {
        setErrorMsg(response.error || t('pairing.failed.errors.reject'))
        setStep('failed')
      }
    } catch (err) {
      console.error('Failed to initiate pairing:', err)
      setErrorMsg(t('pairing.failed.errors.initiate'))
      setStep('failed')
    }
  }

  const handlePinConfirm = async (matches: boolean) => {
    if (!pairingSessionId) return
    setIsPinVerifying(true)
    try {
      await verifyP2PPairingPin({
        sessionId: pairingSessionId,
        pinMatches: matches,
      })
      if (!matches) {
        setIsPinVerifying(false)
        onClose() // User rejected
      }
      // If matches, wait for 'success' or 'failed' event
    } catch (err) {
      console.error('Failed to verify PIN:', err)
      setErrorMsg(t('pairing.failed.errors.verifyPin'))
      setStep('failed')
      setIsPinVerifying(false)
    }
  }

  const getDeviceIcon = (name?: string | null) => {
    const n = (name || '').toLowerCase()
    if (n.includes('phone') || n.includes('iphone') || n.includes('android'))
      return <Smartphone className="w-5 h-5" />
    if (n.includes('mac') || n.includes('book') || n.includes('laptop'))
      return <Laptop className="w-5 h-5" />
    return <Monitor className="w-5 h-5" />
  }

  return (
    <Dialog open={open} onOpenChange={open => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {step === 'discovery' && t('pairing.steps.discovery')}
            {step === 'connecting' && t('pairing.steps.connecting')}
            {step === 'pin-verify' && t('pairing.steps.pinVerify')}
            {step === 'success' && t('pairing.steps.success')}
            {step === 'failed' && t('pairing.steps.failed')}
          </DialogTitle>
          <DialogDescription>
            {step === 'discovery' && t('pairing.discovery.description')}
            {step === 'pin-verify' && t('pairing.pinVerify.description')}
            {step === 'failed' && errorMsg}
          </DialogDescription>
        </DialogHeader>

        <div className="py-6 min-h-[200px] flex flex-col">
          {step === 'discovery' && (
            <div className="flex-1 space-y-4">
              {loading && peers.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-40 space-y-4 text-muted-foreground">
                  <Loader2 className="w-8 h-8 animate-spin" />
                  <p>{t('pairing.discovery.searching')}</p>
                </div>
              ) : peers.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-40 space-y-4 text-muted-foreground">
                  <Monitor className="w-12 h-12 opacity-20" />
                  <p>{t('pairing.discovery.noDevices')}</p>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={loadPeers}
                    disabled={loading}
                    className="gap-2"
                  >
                    <RefreshCw className={`w-3 h-3 ${loading ? 'animate-spin' : ''}`} />
                    {t('pairing.discovery.reScan')}
                  </Button>
                </div>
              ) : (
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-xs text-muted-foreground px-1 pb-2">
                    <span>{t('pairing.discovery.foundCount', { count: peers.length })}</span>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6"
                      onClick={loadPeers}
                      disabled={loading}
                    >
                      <RefreshCw className={`w-3 h-3 ${loading ? 'animate-spin' : ''}`} />
                    </Button>
                  </div>
                  <div className="space-y-2 overflow-y-auto max-h-[300px]">
                    {peers.map(peer => (
                      <button
                        key={peer.peerId}
                        type="button"
                        className="flex items-center justify-between p-3 rounded-lg border bg-card hover:bg-accent/50 transition-colors cursor-pointer group"
                        onClick={() => handleConnect(peer)}
                        onKeyDown={event => {
                          if (event.key === 'Enter' || event.key === ' ') {
                            event.preventDefault()
                            handleConnect(peer)
                          }
                        }}
                      >
                        <div className="flex items-center gap-3">
                          <div className="p-2 bg-primary/10 rounded-full text-primary">
                            {getDeviceIcon(peer.deviceName)}
                          </div>
                          <div>
                            <h4 className="font-medium text-sm">
                              {peer.deviceName || t('pairing.discovery.unknownDevice')}
                            </h4>
                            <p className="text-xs text-muted-foreground truncate w-40">
                              ID: {peer.peerId.substring(0, 8)}...
                            </p>
                          </div>
                        </div>
                        <Button
                          size="sm"
                          variant="secondary"
                          className="opacity-0 group-hover:opacity-100 transition-opacity"
                        >
                          {t('pairing.discovery.connect')}
                        </Button>
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {step === 'connecting' && (
            <div className="flex flex-col items-center justify-center h-48 space-y-4">
              <div className="relative">
                <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center animate-pulse">
                  <Monitor className="w-8 h-8 text-primary" />
                </div>
                <Loader2 className="absolute -bottom-2 -right-2 w-6 h-6 animate-spin text-primary" />
              </div>
              <div className="text-center space-y-1">
                <h3 className="font-medium">
                  {t('pairing.connecting.title', { deviceName: selectedPeer?.deviceName })}
                </h3>
                <p className="text-sm text-muted-foreground">
                  {t('pairing.connecting.pleaseWait')}
                </p>
              </div>
            </div>
          )}

          {step === 'pin-verify' && (
            <div className="flex flex-col items-center justify-center space-y-6 py-4">
              <div className="p-4 bg-muted/50 rounded-xl border-2 border-dashed border-primary/20 text-center w-full max-w-[240px]">
                <p className="text-sm text-muted-foreground mb-2">
                  {t('pairing.pinVerify.pinLabel')}
                </p>
                <div className="text-4xl font-mono font-bold tracking-wider text-primary">
                  {pinCode}
                </div>
              </div>

              <div className="flex items-center gap-2 text-sm text-muted-foreground bg-amber-500/10 text-amber-600 px-4 py-2 rounded-full">
                <ShieldCheck className="w-4 h-4" />
                {t('pairing.pinVerify.warning')}
              </div>

              <div className="flex gap-4 w-full">
                <Button
                  variant="outline"
                  className="flex-1"
                  onClick={() => handlePinConfirm(false)}
                  disabled={isPinVerifying}
                >
                  {t('pairing.pinVerify.notMatch')}
                </Button>
                <Button
                  className="flex-1"
                  onClick={() => handlePinConfirm(true)}
                  disabled={isPinVerifying}
                >
                  {isPinVerifying ? (
                    <span className="flex items-center gap-2">
                      <Loader2 className="w-4 h-4 animate-spin" />
                      {t('pairing.pinVerify.verifying')}
                    </span>
                  ) : (
                    t('pairing.pinVerify.match')
                  )}
                </Button>
              </div>
            </div>
          )}

          {step === 'success' && (
            <div className="flex flex-col items-center justify-center h-48 space-y-4 text-green-600">
              <div className="w-16 h-16 rounded-full bg-green-100 flex items-center justify-center">
                <ShieldCheck className="w-8 h-8" />
              </div>
              <h3 className="text-lg font-medium">{t('pairing.success.title')}</h3>
            </div>
          )}

          {step === 'failed' && (
            <div className="flex flex-col items-center justify-center space-y-6 py-4">
              <div className="w-16 h-16 rounded-full bg-destructive/10 flex items-center justify-center text-destructive">
                <AlertCircle className="w-8 h-8" />
              </div>
              <div className="text-center space-y-2">
                <h3 className="font-medium text-destructive">{t('pairing.failed.title')}</h3>
                <p className="text-sm text-muted-foreground">{errorMsg}</p>
              </div>
              <Button onClick={() => setStep('discovery')} className="w-full">
                {t('pairing.failed.retry')}
              </Button>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
