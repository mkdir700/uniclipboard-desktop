import React from 'react'
import OtherDevice from './OtherDevice'

interface DeviceListProps {
  onAddDevice: () => void
}

const DeviceList: React.FC<DeviceListProps> = ({ onAddDevice }) => {
  return (
    <>
      <OtherDevice onAddDevice={onAddDevice} />
    </>
  )
}

export default DeviceList
