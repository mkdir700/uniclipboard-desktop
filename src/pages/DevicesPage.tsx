import React, { useEffect, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { DeviceList } from '@/components'
import PairingDialog from '@/components/PairingDialog'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import { fetchPairedDevices } from '@/store/slices/devicesSlice'

const DevicesPage: React.FC = () => {
  const [showPairingDialog, setShowPairingDialog] = useState(false)
  const [searchParams, setSearchParams] = useSearchParams()
  const dispatch = useAppDispatch()
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
    if (searchParams.get('pairing') !== '1') {
      return
    }

    setShowPairingDialog(true)
    setSearchParams(
      prev => {
        const next = new URLSearchParams(prev)
        next.delete('pairing')
        return next
      },
      {
        replace: true,
      }
    )
  }, [searchParams, setSearchParams])

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
    </div>
  )
}

export default DevicesPage
