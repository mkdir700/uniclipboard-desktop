import React from 'react'
import CurrentDevice from './CurrentDevice'
import OtherDevice from './OtherDevice'

interface DeviceListProps {
  onAddDevice: () => void
}

const DeviceList: React.FC<DeviceListProps> = ({ onAddDevice }) => {
  return (
    <>
      {/* 当前设备 */}
      <CurrentDevice />

      {/* 其他已连接设备 */}
      <OtherDevice onAddDevice={onAddDevice} />
    </>
  )
}

export default DeviceList
