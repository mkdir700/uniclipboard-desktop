import React, { ReactNode } from 'react'

interface SettingContentLayoutProps {
  children: ReactNode
}

const SettingContentLayout: React.FC<SettingContentLayoutProps> = ({ children }) => {
  return <div className="space-y-6">{children}</div>
}

export default SettingContentLayout
