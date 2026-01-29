import React, { useState, useCallback, useEffect, useRef, ReactNode } from 'react'
import {
  onP2PPairingVerification,
  verifyP2PPairingPin,
  acceptP2PPairing,
  rejectP2PPairing,
  type P2PPairingVerificationEvent,
} from '@/api/p2p'
import { P2PContext, type P2PContextType } from '@/types/p2p'

export const P2PProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [pendingRequest, setPendingRequest] = useState<
    (P2PPairingVerificationEvent & { kind: 'request' }) | null
  >(null)
  const [showRequestDialog, setShowRequestDialog] = useState(false)
  const [showPinDialog, setShowPinDialog] = useState(false)
  const [pinData, setPinData] = useState<
    (P2PPairingVerificationEvent & { kind: 'verification' }) | null
  >(null)

  const cleanupRefs = useRef<(() => void)[]>([])

  // Setup event listeners
  useEffect(() => {
    const setupListeners = async () => {
      const unlistenVerification = await onP2PPairingVerification(event => {
        if (event.kind === 'request') {
          console.log('Received P2P pairing request:', event)
          setPendingRequest(event as P2PPairingVerificationEvent & { kind: 'request' })
          setShowRequestDialog(true)
          return
        }

        if (event.kind === 'verification') {
          console.log('Received P2P verification event:', event)
          setPinData(event as P2PPairingVerificationEvent & { kind: 'verification' })
          setShowPinDialog(true)
          setShowRequestDialog(false)
          return
        }

        if (event.kind === 'complete') {
          console.log('P2P pairing completed')
          setShowPinDialog(false)
          setPendingRequest(null)
          setPinData(null)
          return
        }

        console.error('P2P pairing failed:', event)
        setShowPinDialog(false)
        setShowRequestDialog(false)
        setPendingRequest(null)
        setPinData(null)
      })
      cleanupRefs.current.push(unlistenVerification)
    }

    setupListeners()

    return () => {
      cleanupRefs.current.forEach(cleanup => {
        cleanup()
      })
      cleanupRefs.current = []
    }
  }, [])

  const acceptRequest = useCallback(async () => {
    if (!pendingRequest) return

    try {
      await acceptP2PPairing(pendingRequest.sessionId)
      // After accepting, wait for PIN event - dialog will transition automatically
    } catch (error) {
      console.error('Failed to accept pairing request:', error)
      setShowRequestDialog(false)
      setPendingRequest(null)
    }
  }, [pendingRequest])

  const rejectRequest = useCallback(async () => {
    if (!pendingRequest) return

    try {
      if (pendingRequest.peerId) {
        await rejectP2PPairing(pendingRequest.sessionId, pendingRequest.peerId)
      } else {
        console.warn('Missing peerId for pairing rejection')
      }
    } catch (error) {
      console.error('Failed to reject pairing request:', error)
    } finally {
      setShowRequestDialog(false)
      setPendingRequest(null)
    }
  }, [pendingRequest])

  const verifyPin = useCallback(
    async (matches: boolean) => {
      if (!pinData) return

      try {
        await verifyP2PPairingPin({
          sessionId: pinData.sessionId,
          pinMatches: matches,
        })
        if (!matches) {
          setShowPinDialog(false)
          setPinData(null)
          setPendingRequest(null)
        }
      } catch (error) {
        console.error('Failed to verify PIN:', error)
        setShowPinDialog(false)
        setPinData(null)
        setPendingRequest(null)
      }
    },
    [pinData]
  )

  const closePinDialog = useCallback(() => {
    setShowPinDialog(false)
    setPinData(null)
    setPendingRequest(null)
  }, [])

  const value: P2PContextType = {
    pendingRequest,
    showRequestDialog,
    acceptRequest,
    rejectRequest,
    showPinDialog,
    pinData,
    verifyPin,
    closePinDialog,
  }

  return <P2PContext.Provider value={value}>{children}</P2PContext.Provider>
}
