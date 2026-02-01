import React, { useEffect, useRef, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { onP2PPairingVerification } from '@/api/p2p'
import { DeviceList } from '@/components'
import PairingDialog from '@/components/PairingDialog'
import PairingPinDialog from '@/components/PairingPinDialog'
import { toast } from '@/components/ui/toast'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import { fetchPairedDevices } from '@/store/slices/devicesSlice'

const DevicesPage: React.FC = () => {
  const [showPairingDialog, setShowPairingDialog] = useState(false)
  const [showPinDialog, setShowPinDialog] = useState(false)
  const [pinCode, setPinCode] = useState('')
  const [pinPhase, setPinPhase] = useState<'display' | 'verifying' | 'success'>('verifying')
  const [pinPeerName, setPinPeerName] = useState<string | undefined>(undefined)
  const [searchParams, setSearchParams] = useSearchParams()
  const dispatch = useAppDispatch()
  const activeSessionRef = useRef<string | null>(null)
  const completedSessionRef = useRef<string | null>(null)
  const { pairedDevices, pairedDevicesLoading, pairedDevicesError } = useAppSelector(
    state => state.devices
  )

  const isEmptyState = !pairedDevicesLoading && !pairedDevicesError && pairedDevices.length === 0

  const handleClosePairingDialog = () => {
    setShowPairingDialog(false)
  }

  const handleAddDevice = () => {
    setShowPairingDialog(true)
  }

  const handlePairingSuccess = () => {
    dispatch(fetchPairedDevices())
    setShowPairingDialog(false)
  }

  useEffect(() => {
    if (searchParams.get('pairingPin') !== '1') {
      return
    }

    const sessionId = searchParams.get('sessionId')
    const deviceName = searchParams.get('deviceName')

    setShowPinDialog(true)
    setPinPhase('verifying')
    setPinCode('')
    setPinPeerName(deviceName ?? undefined)
    activeSessionRef.current = sessionId

    setSearchParams(
      prev => {
        const next = new URLSearchParams(prev)
        next.delete('pairingPin')
        next.delete('sessionId')
        next.delete('deviceName')
        return next
      },
      {
        replace: true,
      }
    )
  }, [searchParams, setSearchParams])

  useEffect(() => {
    const unlistenPromise = onP2PPairingVerification(event => {
      if (!activeSessionRef.current || event.sessionId !== activeSessionRef.current) {
        return
      }

      if (event.kind === 'verification') {
        setPinCode(event.code ?? '')
        setPinPeerName(prev => prev ?? event.deviceName)
        setPinPhase('display')
      }

      if (event.kind === 'complete') {
        if (completedSessionRef.current === event.sessionId) {
          return
        }
        completedSessionRef.current = event.sessionId
        setPinPhase('success')
        toast.success('Pairing Successful')
      }
    })

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [])

  return (
    <div className="flex flex-col h-full relative">
      <div className="flex-1 overflow-hidden relative">
        <div
          className={`h-full overflow-y-auto scrollbar-thin px-8 pt-2 scroll-smooth ${
            isEmptyState ? 'pb-12' : 'pb-32'
          }`}
        >
          <div className="mb-12">
            <DeviceList onAddDevice={handleAddDevice} />
          </div>
        </div>
      </div>

      <PairingDialog
        open={showPairingDialog}
        onClose={handleClosePairingDialog}
        onPairingSuccess={handlePairingSuccess}
      />
      <PairingPinDialog
        open={showPinDialog}
        onClose={() => {
          setShowPinDialog(false)
          setPinCode('')
          setPinPeerName(undefined)
          setPinPhase('verifying')
          activeSessionRef.current = null
          completedSessionRef.current = null
        }}
        pinCode={pinCode}
        peerDeviceName={pinPeerName}
        isInitiator={false}
        phase={pinPhase}
        onConfirm={matches => {
          if (!matches) {
            setShowPinDialog(false)
          }
        }}
      />
    </div>
  )
}

export default DevicesPage
