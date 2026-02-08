import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import {
  acceptP2PPairing,
  onP2PPairingVerification,
  rejectP2PPairing,
  type P2PPairingVerificationEvent,
} from '@/api/p2p'
import PairingPinDialog from '@/components/PairingPinDialog'

export function PairingNotificationProvider() {
  const { t } = useTranslation()
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null)
  const activeSessionIdRef = useRef(activeSessionId)

  const [dialogState, setDialogState] = useState<{
    open: boolean
    pinCode: string
    peerDeviceName?: string
    peerId?: string
    phase?: 'display' | 'verifying' | 'success'
  }>({
    open: false,
    pinCode: '',
  })

  useEffect(() => {
    activeSessionIdRef.current = activeSessionId
  }, [activeSessionId])

  useEffect(() => {
    const unlistenPromise = onP2PPairingVerification((event: P2PPairingVerificationEvent) => {
      const currentSessionId = activeSessionIdRef.current

      if (event.kind === 'request') {
        toast(
          t('pairing.request.title', {
            defaultValue: 'Pairing Request',
            device: event.deviceName || 'Unknown Device',
          }),
          {
            description: t('pairing.request.description', {
              defaultValue: 'A device wants to pair with you',
            }),
            action: {
              label: t('common.accept', { defaultValue: 'Accept' }),
              onClick: () => {
                setActiveSessionId(event.sessionId)
                acceptP2PPairing(event.sessionId).catch(err => {
                  console.error(err)
                  toast.error(t('pairing.failed', { defaultValue: 'Pairing failed' }))
                  setActiveSessionId(null)
                })
              },
            },
            cancel: {
              label: t('common.reject', { defaultValue: 'Reject' }),
              onClick: () => {
                if (event.peerId) {
                  rejectP2PPairing(event.sessionId, event.peerId).catch(console.error)
                }
              },
            },
            duration: 15000,
          }
        )
        return
      }

      if (currentSessionId && event.sessionId === currentSessionId) {
        if (event.kind === 'verification') {
          if (event.code) {
            setDialogState({
              open: true,
              pinCode: event.code,
              peerDeviceName: event.deviceName,
              peerId: event.peerId,
              phase: 'display',
            })
          }
        } else if (event.kind === 'verifying') {
          setDialogState(prev => ({ ...prev, phase: 'verifying' }))
        } else if (event.kind === 'complete') {
          setDialogState(prev => ({ ...prev, phase: 'success' }))
          setTimeout(() => {
            setDialogState(prev => ({ ...prev, open: false }))
            setActiveSessionId(null)
          }, 2000)
        } else if (event.kind === 'failed') {
          setDialogState(prev => ({ ...prev, open: false }))
          toast.error(event.error || t('pairing.failed', { defaultValue: 'Pairing failed' }))
          setActiveSessionId(null)
        }
      }
    })

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [t])

  const handleClose = () => {
    setDialogState(prev => ({ ...prev, open: false }))
    setActiveSessionId(null)
  }

  return (
    <PairingPinDialog
      open={dialogState.open}
      onClose={handleClose}
      pinCode={dialogState.pinCode}
      peerDeviceName={dialogState.peerDeviceName}
      isInitiator={false}
      onConfirm={matches => {
        if (!matches) {
          handleClose()
        }
      }}
      phase={dialogState.phase}
    />
  )
}
