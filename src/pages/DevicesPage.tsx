import React, { useState } from 'react'
import { DeviceList } from '@/components'
import PairingDialog from '@/components/PairingDialog'
import { useAppDispatch } from '@/store/hooks'
import { fetchPairedDevices } from '@/store/slices/devicesSlice'

const DevicesPage: React.FC = () => {
  const [showPairingDialog, setShowPairingDialog] = useState(false)
  const dispatch = useAppDispatch()

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

  return (
    <div className="flex flex-col h-full relative">
      <div className="flex-1 overflow-hidden relative">
        <div className="h-full overflow-y-auto scrollbar-thin px-4 pb-12 pt-4 scroll-smooth">
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
