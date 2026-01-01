import React, { useState, useCallback, useEffect, useRef, ReactNode } from 'react'
import {
  onP2PPairingRequest,
  onP2PPinReady,
  onP2PPairingComplete,
  onP2PPairingFailed,
  verifyP2PPairingPin,
  acceptP2PPairing,
  rejectP2PPairing,
  type P2PPairingRequestEvent,
  type P2PPinReadyEvent,
} from '@/api/p2p'
import { P2PContext, type P2PContextType } from '@/types/p2p'

export const P2PProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [pendingRequest, setPendingRequest] = useState<P2PPairingRequestEvent | null>(null)
  const [showRequestDialog, setShowRequestDialog] = useState(false)
  const [showPinDialog, setShowPinDialog] = useState(false)
  const [pinData, setPinData] = useState<P2PPinReadyEvent | null>(null)

  const cleanupRefs = useRef<(() => void)[]>([])

  // Setup event listeners
  useEffect(() => {
    const setupListeners = async () => {
      // Listen for pairing requests
      const unlistenRequest = await onP2PPairingRequest(request => {
        console.log('Received P2P pairing request:', request)
        setPendingRequest(request)
        setShowRequestDialog(true)
      })
      cleanupRefs.current.push(unlistenRequest)

      // Listen for PIN ready
      const unlistenPin = await onP2PPinReady(event => {
        console.log('Received P2P PIN ready event:', event)
        setPinData(event)
        setShowPinDialog(true)
        setShowRequestDialog(false) // Close request dialog
      })
      cleanupRefs.current.push(unlistenPin)

      // Listen for pairing complete
      const unlistenComplete = await onP2PPairingComplete(() => {
        console.log('P2P pairing completed')
        setShowPinDialog(false)
        setPendingRequest(null)
        setPinData(null)
      })
      cleanupRefs.current.push(unlistenComplete)

      // Listen for pairing failed
      const unlistenFailed = await onP2PPairingFailed(event => {
        console.error('P2P pairing failed:', event)
        setShowPinDialog(false)
        setShowRequestDialog(false)
        setPendingRequest(null)
        setPinData(null)
      })
      cleanupRefs.current.push(unlistenFailed)
    }

    setupListeners()

    return () => {
      cleanupRefs.current.forEach(cleanup => cleanup())
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
      await rejectP2PPairing(pendingRequest.sessionId, pendingRequest.peerId)
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
