import React from 'react'
import OtherDevice from './OtherDevice'

interface DeviceListProps {
  onAddDevice: () => void
}

const DeviceList: React.FC<DeviceListProps> = ({ onAddDevice }) => {
  return (
    <>
      {/* 其他已连接设备 */}
      <OtherDevice onAddDevice={onAddDevice} />
    </>
  )
}

export default DeviceList
